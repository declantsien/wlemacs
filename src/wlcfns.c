/* Functions for the Wayland Window System.

Copyright (C) 2024 Free Software Foundation, Inc.

This file is part of GNU Emacs.

GNU Emacs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or (at
your option) any later version.

GNU Emacs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.  */

#include <config.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <unistd.h>

#include "lisp.h"
#include "character.h"
#include "wlcterm.h"
#include "frame.h"
#include "window.h"
#include "buffer.h"
#include "dispextern.h"
#include "keyboard.h"
#include "blockinput.h"
#include "charset.h"
#include "coding.h"
#include "termhooks.h"
#include "font.h"

#include <errno.h>
#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdbool.h>
#include <string.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>
#include "xdg-shell-client-protocol.h"

static void
wlc_set_background_color (struct frame *f, Lisp_Object arg, Lisp_Object oldval)
{
  unsigned long bg;

  block_input ();
  bg = wlc_decode_color (f, arg, WHITE_PIX_DEFAULT (f));
  FRAME_BACKGROUND_PIXEL (f) = bg;

  /* Clear the frame.  */
  if (FRAME_VISIBLE_P (f))
    clear_frame(f);

  /* FRAME_X_OUTPUT (f)->background_color = bg; */
  /* FRAME_X_OUTPUT (f)->cursor_xgcv.foreground = bg; */

  update_face_from_frame_parameter (f, Qbackground_color, arg);

  if (FRAME_VISIBLE_P (f))
    SET_FRAME_GARBAGED (f);
  unblock_input ();
}

static void
wlc_set_border_color (struct frame *f, Lisp_Object arg, Lisp_Object oldval)
{
  int pix;

  CHECK_STRING (arg);
  pix = wlc_decode_color (f, arg, BLACK_PIX_DEFAULT (f));
  FRAME_OUTPUT_DATA (f)->border_pixel = pix;
  /* TODO */
  /* wlc_frame_rehighlight (FRAME_DISPLAY_INFO (f)); */
}

static void
wlc_set_cursor_color (struct frame *f, Lisp_Object arg, Lisp_Object oldval)
{
  unsigned long fore_pixel, pixel;
  struct wlc_output *x = FRAME_OUTPUT_DATA (f);

  if (!NILP (Vx_cursor_fore_pixel))
    {
      fore_pixel = wlc_decode_color (f, Vx_cursor_fore_pixel,
				   WHITE_PIX_DEFAULT (f));
    }
  else
    fore_pixel = FRAME_BACKGROUND_PIXEL (f);

  pixel = wlc_decode_color (f, arg, BLACK_PIX_DEFAULT (f));

  /* Make sure that the cursor color differs from the background color.  */
  if (pixel == FRAME_BACKGROUND_PIXEL (f))
    {
      pixel = x->mouse_color;
      if (pixel == fore_pixel)
	{
	  fore_pixel = FRAME_BACKGROUND_PIXEL (f);
	}
    }

  x->cursor_foreground_color = fore_pixel;
  x->cursor_color = pixel;

  /* if (FRAME_X_WINDOW (f) != 0) */
  /*   { */
      x->cursor_gc.background = x->cursor_color;
      x->cursor_gc.foreground = fore_pixel;

      if (FRAME_VISIBLE_P (f))
	{
	  gui_update_cursor (f, false);
	  gui_update_cursor (f, true);
	}
    /* } */

  update_face_from_frame_parameter (f, Qcursor_color, arg);
}

/* Keep this list in the same order as frame_parms in frame.c.
   Use 0 for unsupported frame parameters.  */

frame_parm_handler wlc_frame_parm_handlers[] =
{
  gui_set_autoraise,
  gui_set_autolower,
  wlc_set_background_color,
  wlc_set_border_color,
  gui_set_border_width,
  wlc_set_cursor_color,
  NULL, /* x_set_cursor_type, */
  gui_set_font,
  NULL, /* x_set_foreground_color, */
  NULL, /* x_set_icon_name, */
  NULL, /* x_set_icon_type, */
  NULL, /* x_set_child_frame_border_width, */
  NULL, /* x_set_internal_border_width, */
  gui_set_right_divider_width,
  gui_set_bottom_divider_width,
  NULL, /* x_set_menu_bar_lines, */
  NULL, /* x_set_mouse_color, */
  NULL, /* x_explicitly_set_name, */
  gui_set_scroll_bar_width,
  gui_set_scroll_bar_height,
  NULL, /* x_set_title, */
  gui_set_unsplittable,
  gui_set_vertical_scroll_bars,
  gui_set_horizontal_scroll_bars,
  gui_set_visibility,
  NULL, /* x_set_tab_bar_lines, */
  NULL, /* x_set_tool_bar_lines, */
  NULL, /* x_set_scroll_bar_foreground, */
  NULL, /* x_set_scroll_bar_background, */
  gui_set_screen_gamma,
  gui_set_line_spacing,
  gui_set_left_fringe,
  gui_set_right_fringe,
  NULL, /* x_set_wait_for_wm, */
  gui_set_fullscreen,
  gui_set_font_backend,
  NULL, /* x_set_alpha, */
  NULL, /* x_set_sticky, */
  NULL, /* x_set_tool_bar_position, */
#ifdef HAVE_XDBE
  NULL, /* x_set_inhibit_double_buffering, */
#else
  NULL,
#endif
  NULL, /* x_set_undecorated, */
  NULL, /* x_set_parent_frame, */
  NULL, /* x_set_skip_taskbar, */
  NULL, /* x_set_no_focus_on_map, */
  NULL, /* x_set_no_accept_focus, */
  NULL, /* x_set_z_group, */
  NULL, /* x_set_override_redirect, */
  gui_set_no_special_glyphs,
  NULL, /* x_set_alpha_background, */
  NULL, /* x_set_use_frame_synchronization, */
  NULL, /* x_set_shaded, */
};

/* Called from frame.c.  */
struct wlc_display_info *
check_x_display_info (Lisp_Object frame)
{
  return check_wlc_display_info (frame);
}

unsigned long
wlc_get_pixel (Emacs_Pix_Context bitmap, int x, int y)
{
  return 0.0;
}

void
wlc_put_pixel (Emacs_Pix_Context bitmap, int x, int y, unsigned long pixel)
{
}

DEFUN ("x-hide-tip", Fx_hide_tip, Sx_hide_tip, 0, 0, 0,
       doc: /* TODO: Hide the current tooltip window, if there is any.
Value is t if tooltip was open, nil otherwise.  */)
  (void)
{
  return Qnil;
}

DEFUN ("xw-color-defined-p", Fxw_color_defined_p, Sxw_color_defined_p, 1, 2, 0,
       doc: /* Internal function called by `color-defined-p', which see.  */)
  (Lisp_Object color, Lisp_Object frame)
{
  Emacs_Color col;
  struct frame *f = decode_window_system_frame (frame);

  CHECK_STRING (color);

  if (wr_defined_color (f, SSDATA (color), &col, false, false))
    return Qt;
  else
    return Qnil;
}


DEFUN ("xw-color-values", Fxw_color_values, Sxw_color_values, 1, 2, 0,
       doc: /* Internal function called by `color-values', which see.  */)
  (Lisp_Object color, Lisp_Object frame)
{
  Emacs_Color col;
  struct frame *f = decode_window_system_frame (frame);

  CHECK_STRING (color);

  if (wr_defined_color (f, SSDATA (color), &col, false, false))
    return list3i (col.red, col.green, col.blue);
  else
    return Qnil;
}

DEFUN ("xw-display-color-p", Fxw_display_color_p, Sxw_display_color_p, 0, 1, 0,
       doc: /* TODO: Internal function called by `display-color-p'.  */)
  (Lisp_Object terminal)
{
  check_wlc_display_info (terminal);
  return Qt;
}

DEFUN ("x-display-grayscale-p", Fx_display_grayscale_p, Sx_display_grayscale_p,
       0, 1, 0,
       doc: /* TODO: Return t if the Wayland display supports shades of gray.
Note that color displays do support shades of gray.
The optional argument TERMINAL specifies which display to ask about.
TERMINAL should be a terminal object, a frame or a display name (a string).
If omitted or nil, that stands for the selected frame's display.  */)
  (Lisp_Object terminal)
{
  return Qnil;
}

DEFUN ("x-display-planes", Fx_display_planes, Sx_display_planes, 0, 1, 0,
       doc: /* Return the number of bitplanes of the display TERMINAL.
The optional argument TERMINAL specifies which display to ask about.
TERMINAL should be a terminal object, a frame or a display name (a string).
If omitted or nil, that stands for the selected frame's display.  */)
  (Lisp_Object terminal)
{
  check_wlc_display_info (terminal);
  return make_fixnum (32);
}

DEFUN ("x-display-color-cells", Fx_display_color_cells, Sx_display_color_cells, 0, 1, 0,
       doc: /* Returns the number of color cells of the display TERMINAL.
The optional argument TERMINAL specifies which display to ask about.
TERMINAL should be a terminal object, a frame or a display name (a string).
If omitted or nil, that stands for the selected frame's display.  */)
  (Lisp_Object terminal)
{
  struct wlc_display_info *dpyinfo = check_wlc_display_info (terminal);
  /* We force 24+ bit depths to 24-bit to prevent an overflow.  */
  return make_fixnum (1 << min (dpyinfo->n_planes, 24));
}

DEFUN ("wlc-open-connection", Fwlc_open_connection, Swlc_open_connection, 1, 3, 0,
       doc: /* Open a connection to Wayland display server.
DISPLAY is the name of the display to connect to.
Optional second arg XRM-STRING is a string of resources in xrdb format.
If the optional third arg MUST-SUCCEED is non-nil,
terminate Emacs if we can't open the connection.  */)
  (Lisp_Object display, Lisp_Object resource_string, Lisp_Object must_succeed)
{
  struct wlc_display_info *dpyinfo;

  dpyinfo = wlc_term_init (display);
  if (dpyinfo == 0)
    {
      if (!NILP (must_succeed))
	fatal ("Display on %s not responding.\n", SSDATA (display));
      else
	error ("Display on %s not responding.\n", SSDATA (display));
    }

  return Qnil;
}

/* Handler for signals raised during x_create_frame and
   x_create_tip_frame.  FRAME is the frame which is partially
   constructed.  */

static Lisp_Object
unwind_create_frame (Lisp_Object frame)
{
  struct frame *f = XFRAME (frame);

  /* If frame is already dead, nothing to do.  This can happen if the
     display is disconnected after the frame has become official, but
     before x_create_frame removes the unwind protect.  */
  if (!FRAME_LIVE_P (f))
    return Qnil;

  /* If frame is ``official'', nothing to do.  */
  if (NILP (Fmemq (frame, Vframe_list)))
    {
      /* pgtk_free_frame_resources (f); */
      /* free_glyphs (f); */
      return Qt;
    }

  return Qnil;
}

static void
do_unwind_create_frame (Lisp_Object frame)
{
  unwind_create_frame (frame);
}

/* Return the Wayland display structure for the display named NAME.
   Open a new connection if necessary.  */

static struct wlc_display_info *
wlc_display_info_for_name (Lisp_Object name)
{
  struct wlc_display_info *dpyinfo;

  CHECK_STRING (name);

  for (dpyinfo = x_display_list; dpyinfo; dpyinfo = dpyinfo->next)
    if (!NILP (Fstring_equal (XCAR (dpyinfo->name_list_element), name)))
      return dpyinfo;

  dpyinfo = wlc_term_init (name);

  if (dpyinfo == 0)
    error ("Cannot connect to Wayland server %s", SDATA (name));

  return dpyinfo;
}

/* Let the user specify an Wayland display with a Lisp object.
   OBJECT may be nil, a frame or a terminal object.
   nil stands for the selected frame--or, if that is not an Wayland frame,
   the first Wayland display on the list.  */

struct wlc_display_info *
check_wlc_display_info (Lisp_Object object)
{
  struct wlc_display_info *dpyinfo = NULL;

  if (NILP (object))
    {
      struct frame *sf = XFRAME (selected_frame);

      if (FRAME_WLC_P (sf) && FRAME_LIVE_P (sf))
	dpyinfo = FRAME_DISPLAY_INFO (sf);
      else if (x_display_list != 0)
	dpyinfo = x_display_list;
      else
	error ("Wayland windows are not in use or not initialized");
    }
  else if (TERMINALP (object))
    {
      struct terminal *t = decode_live_terminal (object);

      if (t->type != output_wlc)
        error ("Terminal %d is not an Wayland display", t->id);

      dpyinfo = t->display_info.wlc;
    }
  else if (STRINGP (object))
    dpyinfo = wlc_display_info_for_name (object);
  else
    {
      struct frame *f = decode_window_system_frame (object);
      dpyinfo = FRAME_DISPLAY_INFO (f);
    }

  return dpyinfo;
}

/* Shared memory support code */
static void
randname(char *buf)
{
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    long r = ts.tv_nsec;
    for (int i = 0; i < 6; ++i) {
        buf[i] = 'A'+(r&15)+(r&16)*2;
        r >>= 5;
    }
}

static int
create_shm_file(void)
{
    int retries = 100;
    do {
        char name[] = "/wl_shm-XXXXXX";
        randname(name + sizeof(name) - 7);
        --retries;
        int fd = shm_open(name, O_RDWR | O_CREAT | O_EXCL, 0600);
        if (fd >= 0) {
            shm_unlink(name);
            return fd;
        }
    } while (retries > 0 && errno == EEXIST);
    return -1;
}

static int
allocate_shm_file(size_t size)
{
    int fd = create_shm_file();
    if (fd < 0)
        return -1;
    int ret;
    do {
        ret = ftruncate(fd, size);
    } while (ret < 0 && errno == EINTR);
    if (ret < 0) {
        close(fd);
        return -1;
    }
    return fd;
}

static void
wl_buffer_release(void *data, struct wl_buffer *wl_buffer)
{
    /* Sent by the compositor when it's no longer using this buffer */
    wl_buffer_destroy(wl_buffer);
}

static const struct wl_buffer_listener wl_buffer_listener = {
    .release = wl_buffer_release,
};

static struct wl_buffer *
draw_frame(struct frame *f)
{
  const int width = FRAME_PIXEL_WIDTH (f), height = FRAME_PIXEL_HEIGHT (f);
  int stride = width * 4;
  int size = stride * height;
  /* fprintf(stderr, "draw frame %d, %d ", width, height); */

  int fd = allocate_shm_file(size);
  if (fd == -1) {
    return NULL;
  }

  uint32_t *data = mmap(NULL, size,
			PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
  if (data == MAP_FAILED) {
    close(fd);
    return NULL;
  }

  struct wl_shm_pool *pool = wl_shm_create_pool(FRAME_DISPLAY_INFO(f)->shm, fd, size);
  struct wl_buffer *buffer = wl_shm_pool_create_buffer(pool, 0,
						       width, height, stride, WL_SHM_FORMAT_XRGB8888);
  wl_shm_pool_destroy(pool);
  close(fd);

  /* Draw checkerboxed background */
  for (int y = 0; y < height; ++y) {
    for (int x = 0; x < width; ++x) {
      if ((x + y / 8 * 8) % 16 < 8)
	data[y * width + x] = 0xFF666666;
      else
	data[y * width + x] = 0xFFEEEEEE;
    }
  }

  munmap(data, size);
  wl_buffer_add_listener(buffer, &wl_buffer_listener, NULL);
  return buffer;
}

static void
xdg_toplevel_configure(void *data,
		struct xdg_toplevel *xdg_toplevel, int32_t width, int32_t height,
		struct wl_array *states)
{
  struct frame *f = data;
  if (width == 0 || height == 0)
    {
    fprintf(stderr, "TODO: set xdg top level size");
    /* Compositor is deferring to us */
    return;
    }
  fprintf(stderr, "new size %d, %d ", width, height);
  fprintf(stderr, "emacs size %d, %d ", FRAME_PIXEL_WIDTH (f), FRAME_PIXEL_HEIGHT (f));



  if (width > 0 && height > 0) {
    FRAME_PIXEL_WIDTH (f) = width;
    FRAME_PIXEL_HEIGHT (f) = height;

    if (FRAME_OUTPUT_DATA (f)->wait_for_configure) {
      if (FRAME_OUTPUT_DATA (f)->enable_compositor) {
        wp_viewport_set_destination(FRAME_OUTPUT_DATA (f)->viewport, FRAME_PIXEL_WIDTH (f),
				    FRAME_PIXEL_HEIGHT (f));
      } else {
	gl_renderer_fit_context(f);
	wl_surface_commit(FRAME_OUTPUT_DATA (f)->surface);
      }
    }
  }
  FRAME_OUTPUT_DATA (f)->wait_for_configure = false;
}

static void
xdg_toplevel_close(void *data, struct xdg_toplevel *toplevel)
{
  struct frame *f = data;
  fprintf(stderr, "close frame");
  wlc_handle_xdg_toplevel_close(f);
}

const static struct xdg_toplevel_listener xdg_toplevel_listener = {
	.configure = xdg_toplevel_configure,
	.close = xdg_toplevel_close,
};

static void
xdg_surface_configure(void *data,
        struct xdg_surface *xdg_surface, uint32_t serial)
{
    struct frame *f = data;
    gl_renderer_fit_context(f);
    xdg_surface_ack_configure(xdg_surface, serial);
    /* struct wl_buffer *buffer = draw_frame(f); */
    /* wl_surface_attach(FRAME_OUTPUT_DATA(f)->surface, buffer, 0, 0); */
    /* wl_surface_commit(FRAME_OUTPUT_DATA(f)->surface); */
    // Emacs redisplay check FRAME_REDISPLAY_P before
    f->visible = true;
    int width = FRAME_PIXEL_WIDTH (f);
    int height = FRAME_PIXEL_HEIGHT (f) ;
    xdg_surface_set_window_geometry(FRAME_OUTPUT_DATA (f)->xdg_surface, 0, 0, width, height);

    if (FRAME_OUTPUT_DATA (f)->wait_for_configure) {
      if (FRAME_OUTPUT_DATA (f)->enable_compositor) {

	/* window->egl_window = wl_egl_window_create(window->surface, 1, 1); */
	/* window->egl_surface = eglCreateWindowSurface( */
	/* 					     window->eglDisplay, window->config, window->egl_window, NULL); */
	/* assert(window->egl_surface != EGL_NO_SURFACE); */

	/* EGLBoolean ok = eglMakeCurrent(window->eglDisplay, window->egl_surface, */
	/* 			       window->egl_surface, window->eglContext); */
	/* assert(ok); */

	/* glClearColor(1.0, 1.0, 1.0, 1.0); */
	/* glClear(GL_COLOR_BUFFER_BIT); */

	FRAME_OUTPUT_DATA (f)->viewport = wp_viewporter_get_viewport(FRAME_DISPLAY_INFO (f)->viewporter,
						      FRAME_OUTPUT_DATA (f)->surface);
	wp_viewport_set_destination(FRAME_OUTPUT_DATA (f)->viewport, width, height);

	/* eglSwapBuffers(window->eglDisplay, window->egl_surface); */
      } else {
	/* window->egl_window = wl_egl_window_create( */
	/* 					  window->surface, window->geometry.width, window->geometry.height); */
	/* window->egl_surface = eglCreateWindowSurface( */
	/* 					     window->eglDisplay, window->config, window->egl_window, NULL); */
	/* assert(window->egl_surface != EGL_NO_SURFACE); */

	/* EGLBoolean ok = eglMakeCurrent(window->eglDisplay, window->egl_surface, */
	/* 			       window->egl_surface, window->eglContext); */
	/* assert(ok); */
      }
    }

    FRAME_OUTPUT_DATA (f)->wait_for_configure = false;
}

static const struct xdg_surface_listener xdg_surface_listener = {
  .configure = xdg_surface_configure,
};

static void
zxdg_toplevel_decoration_v1_handle_configure(void *data, struct zxdg_toplevel_decoration_v1 *deco, uint32_t mode)
{
  struct frame *wl = data;
  int csd = mode == ZXDG_TOPLEVEL_DECORATION_V1_MODE_CLIENT_SIDE;
  if (csd)
    fprintf (stderr, "Using xdg toplevel decoration client mode");
  else
    fprintf (stderr, "Using xdg toplevel decoration server mode");
  /* if(csd == wl->client_side_deco){ */
  /*   return; */
  /* } */

  /* wl->client_side_deco = csd; */
}

static const struct zxdg_toplevel_decoration_v1_listener zxdg_toplevel_decoration_v1_listener = {
	.configure = zxdg_toplevel_decoration_v1_handle_configure,
};


static void
init_xdg_window (struct frame *f, long window_prompting)
{
  FRAME_OUTPUT_DATA(f)->xdg_surface =
    xdg_wm_base_get_xdg_surface(FRAME_DISPLAY_INFO(f)->wm_base,
				FRAME_OUTPUT_DATA(f)->surface);
  assert(FRAME_OUTPUT_DATA(f)->xdg_surface);

  xdg_surface_add_listener(FRAME_OUTPUT_DATA(f)->xdg_surface, &xdg_surface_listener, f);
  FRAME_OUTPUT_DATA(f)->xdg_toplevel = xdg_surface_get_toplevel(FRAME_OUTPUT_DATA(f)->xdg_surface);

  xdg_toplevel_add_listener(FRAME_OUTPUT_DATA(f)->xdg_toplevel,
			    &xdg_toplevel_listener, f);
  assert(FRAME_OUTPUT_DATA(f)->xdg_toplevel);

  xdg_toplevel_set_title (FRAME_OUTPUT_DATA (f)->xdg_toplevel,
			  "Emacs");
  xdg_toplevel_set_app_id(FRAME_OUTPUT_DATA (f)->xdg_toplevel,
                            "org.gnu.emacs");

  xdg_surface_set_window_geometry(FRAME_OUTPUT_DATA(f)->xdg_surface, f->left_pos, f->top_pos,
				  FRAME_PIXEL_WIDTH (f), FRAME_PIXEL_HEIGHT (f));

  FRAME_OUTPUT_DATA (f)->wait_for_configure = true;
  wl_surface_commit (FRAME_OUTPUT_DATA (f)->surface);

  if (FRAME_DISPLAY_INFO (f)->decoration_manager)
    {

    FRAME_OUTPUT_DATA (f)->decoration
	= zxdg_decoration_manager_v1_get_toplevel_decoration (
	  FRAME_DISPLAY_INFO (f)->decoration_manager,
	  FRAME_OUTPUT_DATA (f)->xdg_toplevel);
    zxdg_toplevel_decoration_v1_add_listener(FRAME_OUTPUT_DATA (f)->decoration, &zxdg_toplevel_decoration_v1_listener, f);
    zxdg_toplevel_decoration_v1_set_mode(FRAME_OUTPUT_DATA (f)->decoration, ZXDG_TOPLEVEL_DECORATION_V1_MODE_SERVER_SIDE);
    wl_display_roundtrip(FRAME_DISPLAY_INFO (f)->display);
  }
}

const static struct wl_callback_listener wl_surface_frame_listener;

static void
wl_surface_frame_done(void *data, struct wl_callback *cb, uint32_t time)
{
  /* Destroy this callback */
  wl_callback_destroy(cb);

  /* Request another frame */
  struct frame *f = data;
  cb = wl_surface_frame(FRAME_OUTPUT_DATA(f)->surface);
  wl_callback_add_listener(cb, &wl_surface_frame_listener, f);

  /* Update scroll amount at 24 pixels per second */
  if (FRAME_OUTPUT_DATA(f)->last_surface_frame != 0) {
    int elapsed = time - FRAME_OUTPUT_DATA(f)->last_surface_frame;
    FRAME_OUTPUT_DATA(f)->offset += elapsed / 1000.0 * 24;
  }

  /* /\* Submit a frame for this event *\/ */
  /* struct wl_buffer *buffer = draw_frame(f); */
  /* wl_surface_attach(FRAME_OUTPUT_DATA(f)->surface, buffer, 0, 0); */
  /* wl_surface_damage_buffer(FRAME_OUTPUT_DATA(f)->surface, 0, 0, INT32_MAX, INT32_MAX); */
  /* wl_surface_commit(FRAME_OUTPUT_DATA(f)->surface); */
  /* redraw_frame (f); */
  /* update_frame (f, true); */
  /* wr_flush_display(f); */
  /* redisplay() */

  FRAME_OUTPUT_DATA(f)->last_surface_frame = time;
}

const static struct wl_callback_listener wl_surface_frame_listener = {
	.done = wl_surface_frame_done,
};

DEFUN ("x-create-frame", Fx_create_frame, Sx_create_frame,
       1, 1, 0,
       doc: /* Make a new X window, which is called a "frame" in Emacs terms.
Return an Emacs frame object.  PARMS is an alist of frame parameters.
If the parameters specify that the frame should not have a minibuffer,
and do not specify a specific minibuffer window to use, then
`default-minibuffer-frame' must be a frame whose minibuffer can be
shared by the new frame.

This function is an internal primitive--use `make-frame' instead.  */)
  (Lisp_Object parms)
{
  struct frame *f;
  Lisp_Object frame, tem;
  Lisp_Object name;
  bool minibuffer_only = false;
  bool undecorated = false, override_redirect = false;
  long window_prompting = 0;
  specpdl_ref count = SPECPDL_INDEX ();
  Lisp_Object display;
  struct wlc_display_info *dpyinfo = NULL;
  Lisp_Object parent, parent_frame;
  struct kboard *kb;

  parms = Fcopy_alist (parms);

  /* Use this general default value to start with
     until we know if this frame has a specified name.  */
  /* FIXME: Do we need to set it under Wayland */
  Vx_resource_name = Vinvocation_name;

  display = gui_display_get_arg (dpyinfo, parms, Qterminal, 0, 0,
                                 RES_TYPE_NUMBER);
  if (BASE_EQ (display, Qunbound))
    display = gui_display_get_arg (dpyinfo, parms, Qdisplay, 0, 0,
                                   RES_TYPE_STRING);
  if (BASE_EQ (display, Qunbound))
    display = Qnil;
  dpyinfo = check_wlc_display_info (display);
  kb = dpyinfo->terminal->kboard;

  if (!dpyinfo->terminal->name)
    error ("Terminal is not live, can't create new frames on it");

  name = gui_display_get_arg (dpyinfo, parms, Qname, "name", "Name",
                              RES_TYPE_STRING);
  if (!STRINGP (name)
      && ! BASE_EQ (name, Qunbound)
      && ! NILP (name))
    error ("Invalid frame name--not a string or nil");

  if (STRINGP (name))
    Vx_resource_name = name;

  /* See if parent window is specified.  */
  parent = gui_display_get_arg (dpyinfo, parms, Qparent_id, NULL, NULL,
                                RES_TYPE_NUMBER);
  if (BASE_EQ (parent, Qunbound))
    parent = Qnil;
  if (! NILP (parent))
    CHECK_FIXNUM (parent);

  frame = Qnil;
  tem = gui_display_get_arg (dpyinfo,
                             parms, Qminibuffer, "minibuffer", "Minibuffer",
                             RES_TYPE_SYMBOL);
  if (EQ (tem, Qnone) || NILP (tem))
    f = make_frame_without_minibuffer (Qnil, kb, display);
  else if (EQ (tem, Qonly))
    {
      f = make_minibuffer_frame ();
      minibuffer_only = true;
    }
  else if (WINDOWP (tem))
    f = make_frame_without_minibuffer (tem, kb, display);
  else
    f = make_frame (true);

  /* parent_frame = gui_display_get_arg (dpyinfo, */
  /*                                     parms, */
  /*                                     Qparent_frame, */
  /*                                     NULL, */
  /*                                     NULL, */
  /*                                     RES_TYPE_SYMBOL); */
  /* /\* Accept parent-frame iff parent-id was not specified.  *\/ */
  /* if (!NILP (parent) */
  /*     || BASE_EQ (parent_frame, Qunbound) */
  /*     || NILP (parent_frame) */
  /*     || !FRAMEP (parent_frame) */
  /*     || !FRAME_LIVE_P (XFRAME (parent_frame)) */
  /*     || !FRAME_X_P (XFRAME (parent_frame))) */
  /*   parent_frame = Qnil; */

  /* fset_parent_frame (f, parent_frame); */
  /* store_frame_param (f, Qparent_frame, parent_frame); */

  /* if (!NILP (tem = (gui_display_get_arg (dpyinfo, */
  /*                                        parms, */
  /*                                        Qundecorated, */
  /*                                        NULL, */
  /*                                        NULL, */
  /*                                        RES_TYPE_BOOLEAN))) */
  /*     && !(BASE_EQ (tem, Qunbound))) */
  /*   undecorated = true; */

  /* FRAME_UNDECORATED (f) = undecorated; */
  /* store_frame_param (f, Qundecorated, undecorated ? Qt : Qnil); */

  /* if (!NILP (tem = (gui_display_get_arg (dpyinfo, */
  /*                                        parms, */
  /*                                        Qoverride_redirect, */
  /*                                        NULL, */
  /*                                        NULL, */
  /*                                        RES_TYPE_BOOLEAN))) */
  /*     && !(BASE_EQ (tem, Qunbound))) */
  /*   override_redirect = true; */

  /* FRAME_OVERRIDE_REDIRECT (f) = override_redirect; */
  /* store_frame_param (f, Qoverride_redirect, override_redirect ? Qt : Qnil); */

  XSETFRAME (frame, f);

  f->terminal = dpyinfo->terminal;

  /* Init output data */
  f->output_method = output_wlc;
  FRAME_OUTPUT_DATA (f) = xzalloc (sizeof *FRAME_OUTPUT_DATA (f));
  FRAME_FONTSET (f) = -1;
  /* FRAME_OUTPUT_DATA (f)->white_relief.pixel = -1; */
  /* FRAME_OUTPUT_DATA (f)->black_relief.pixel = -1; */

  /* fset_icon_name (f, gui_display_get_arg (dpyinfo, */
  /*                                         parms, */
  /*                                         Qicon_name, */
  /*                                         "iconName", */
  /*                                         "Title", */
  /*                                         RES_TYPE_STRING)); */
  /* if (! STRINGP (f->icon_name)) */
  /*   fset_icon_name (f, Qnil); */

  FRAME_DISPLAY_INFO (f) = dpyinfo;

  /* With FRAME_DISPLAY_INFO set up, this unwind-protect is safe.  */
  record_unwind_protect (do_unwind_create_frame, frame);

  /* /\* These colors will be set anyway later, but it's important */
  /*    to get the color reference counts right, so initialize them!  *\/ */
  /* { */
  /*   Lisp_Object black; */

  /*   /\* Function x_decode_color can signal an error.  Make */
  /*      sure to initialize color slots so that we won't try */
  /*      to free colors we haven't allocated.  *\/ */
  /*   FRAME_FOREGROUND_PIXEL (f) = -1; */
  /*   FRAME_BACKGROUND_PIXEL (f) = -1; */
  /*   f->output_data.x->cursor_pixel = -1; */
  /*   f->output_data.x->cursor_foreground_pixel = -1; */
  /*   f->output_data.x->border_pixel = -1; */
  /*   f->output_data.x->mouse_pixel = -1; */

  /*   black = build_string ("black"); */
  /*   FRAME_FOREGROUND_PIXEL (f) */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /*   FRAME_BACKGROUND_PIXEL (f) */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /*   f->output_data.x->cursor_pixel */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /*   f->output_data.x->cursor_foreground_pixel */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /*   f->output_data.x->border_pixel */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /*   f->output_data.x->mouse_pixel */
  /*     = x_decode_color (f, black, BLACK_PIX_DEFAULT (f)); */
  /* } */

  /* /\* Specify the parent under which to make this X window.  *\/ */
  /* if (!NILP (parent)) */
  /*   { */
  /*     FRAME_OUTPUT_DATA (f)->parent_desc = (Window) XFIXNAT (parent); */
  /*     FRAME_OUTPUT_DATA (f)->explicit_parent = true; */
  /*   } */
  /* else */
  /*   { */
  /*     FRAME_OUTPUT_DATA (f)->parent_desc = FRAME_DISPLAY_INFO (f)->root_window; */
  /*     FRAME_OUTPUT_DATA (f)->explicit_parent = false; */
  /*   } */

  /* /\* Set the name; the functions to which we pass f expect the name to */
  /*    be set.  *\/ */
  /* if (BASE_EQ (name, Qunbound) || NILP (name)) */
  /*   { */
  /*     /\* fset_name (f, build_string (dpyinfo->x_id_name)); *\/ */
  /*     f->explicit_name = false; */
  /*   } */
  /* else */
  /*   { */
  /*     fset_name (f, name); */
  /*     f->explicit_name = true; */
  /*     /\* Use the frame's title when getting resources for this frame.  *\/ */
  /*     specbind (Qx_resource_name, name); */
  /*   } */

/*   register_font_driver (&ftwrfont_driver, f); */
/* #ifdef HAVE_HARFBUZZ */
/*   register_font_driver (&ftwrhbfont_driver, f); */
/* #endif	/\* HAVE_HARFBUZZ *\/ */
  register_swash_font_driver(f);

/* #ifdef GLYPH_DEBUG */
/*   dpyinfo_refcount = dpyinfo->reference_count; */
/* #endif /\* GLYPH_DEBUG *\/ */

  gui_default_parameter (f, parms, Qfont_backend, Qnil,
                         "fontBackend", "FontBackend", RES_TYPE_STRING);

  /* Extract the window parameters from the supplied values
     that are needed to determine window geometry.  */
  FRAME_RIF (f)->default_font_parameter (f, parms);
  /* /\* // We rely on Rust font-index crate to choose a generic Monospace font *\/ */
  /* /\* gui_default_parameter (f, parms, Qfont, Qnil, *\/ */
  /* /\*                        "font", "Monospace", RES_TYPE_STRING); *\/ */

  /* /\* FIXME font is invalid here *\/ */
  /* if (!FRAME_FONT (f)) */
  /*   { */
  /*     int height = FONT_HEIGHT (FRAME_FONT (f)); */
  /*     delete_frame (frame, Qnoelisp); */
  /*     error ("Invalid frame font"); */
  /*   } */

  /* gui_default_parameter (f, parms, Qborder_width, make_fixnum (0), */
  /*                        "borderWidth", "BorderWidth", RES_TYPE_NUMBER); */

  /* /\* This defaults to 1 in order to match xterm.  We recognize either */
  /*    internalBorderWidth or internalBorder (which is what xterm calls */
  /*    it).  *\/ */
  /* if (NILP (Fassq (Qinternal_border_width, parms))) */
  /*   { */
  /*     Lisp_Object value; */

  /*     value = gui_display_get_arg (dpyinfo, parms, Qinternal_border_width, */
  /*                                  "internalBorder", "internalBorder", */
  /*                                  RES_TYPE_NUMBER); */
  /*     if (! BASE_EQ (value, Qunbound)) */
  /* 	parms = Fcons (Fcons (Qinternal_border_width, value), */
  /* 		       parms); */
  /*   } */

  /* gui_default_parameter (f, parms, Qinternal_border_width, make_fixnum (0), */
  /*                        "internalBorderWidth", "internalBorderWidth", */
  /*                        RES_TYPE_NUMBER); */

  /* /\* Same for child frames.  *\/ */
  /* if (NILP (Fassq (Qchild_frame_border_width, parms))) */
  /*   { */
  /*     Lisp_Object value; */

  /*     value = gui_display_get_arg (dpyinfo, parms, Qchild_frame_border_width, */
  /*                                  "childFrameBorder", "childFrameBorder", */
  /*                                  RES_TYPE_NUMBER); */
  /*     if (! BASE_EQ (value, Qunbound)) */
  /* 	parms = Fcons (Fcons (Qchild_frame_border_width, value), */
  /* 		       parms); */
  /*   } */

  /* gui_default_parameter (f, parms, Qchild_frame_border_width, Qnil, */
  /* 			 "childFrameBorderWidth", "childFrameBorderWidth", */
  /* 			 RES_TYPE_NUMBER); */

  /* gui_default_parameter (f, parms, Qright_divider_width, make_fixnum (0), */
  /*                        NULL, NULL, RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qbottom_divider_width, make_fixnum (0), */
  /*                        NULL, NULL, RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qvertical_scroll_bars, */
  /*                        Qright, */
  /*                        "verticalScrollBars", "ScrollBars", */
  /*                        RES_TYPE_SYMBOL); */
  /* gui_default_parameter (f, parms, Qhorizontal_scroll_bars, Qnil, */
  /*                        "horizontalScrollBars", "ScrollBars", */
  /*                        RES_TYPE_SYMBOL); */

  /* Also do the stuff which must be set before the window exists.  */
  gui_default_parameter (f, parms, Qforeground_color, build_string ("black"),
                         "foreground", "Foreground", RES_TYPE_STRING);
  gui_default_parameter (f, parms, Qbackground_color, build_string ("white"),
                         "background", "Background", RES_TYPE_STRING);
  /* gui_default_parameter (f, parms, Qmouse_color, build_string ("black"), */
  /*                        "pointerColor", "Foreground", RES_TYPE_STRING); */
  /* gui_default_parameter (f, parms, Qborder_color, build_string ("black"), */
  /*                        "borderColor", "BorderColor", RES_TYPE_STRING); */
  /* gui_default_parameter (f, parms, Qno_special_glyphs, Qnil, */
  /*                        NULL, NULL, RES_TYPE_BOOLEAN); */
  /* gui_default_parameter (f, parms, Qscreen_gamma, Qnil, */
  /*                        "screenGamma", "ScreenGamma", RES_TYPE_FLOAT); */
  /* gui_default_parameter (f, parms, Qline_spacing, Qnil, */
  /*                        "lineSpacing", "LineSpacing", RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qleft_fringe, Qnil, */
  /*                        "leftFringe", "LeftFringe", RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qright_fringe, Qnil, */
  /*                        "rightFringe", "RightFringe", RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qno_special_glyphs, Qnil, */
  /*                        NULL, NULL, RES_TYPE_BOOLEAN); */
  /* gui_default_parameter (f, parms, Qscroll_bar_foreground, Qnil, */
  /* 			 "scrollBarForeground", "ScrollBarForeground", */
  /* 			 RES_TYPE_STRING); */
  /* gui_default_parameter (f, parms, Qscroll_bar_background, Qnil, */
  /* 			 "scrollBarBackground", "ScrollBarBackground", */
  /* 			 RES_TYPE_STRING); */

  /* Init faces before gui_default_parameter is called for the
     scroll-bar-width parameter because otherwise we end up in
     init_iterator with a null face cache, which should not happen.  */
  init_frame_faces (f);

  /* tem = gui_display_get_arg (dpyinfo, parms, Qmin_width, NULL, NULL, */
  /*                            RES_TYPE_NUMBER); */
  /* if (FIXNUMP (tem)) */
  /*   store_frame_param (f, Qmin_width, tem); */
  /* tem = gui_display_get_arg (dpyinfo, parms, Qmin_height, NULL, NULL, */
  /*                            RES_TYPE_NUMBER); */
  /* if (FIXNUMP (tem)) */
  /*   store_frame_param (f, Qmin_height, tem); */

  adjust_frame_size (f, FRAME_COLS (f) * FRAME_COLUMN_WIDTH (f),
		     FRAME_LINES (f) * FRAME_LINE_HEIGHT (f), 5, true,
		     Qx_create_frame_1);

  /* /\* Set the menu-bar-lines and tool-bar-lines parameters.  We don't */
  /*    look up the X resources controlling the menu-bar and tool-bar */
  /*    here; they are processed specially at startup, and reflected in */
  /*    the values of the mode variables.  *\/ */

  /* gui_default_parameter (f, parms, Qmenu_bar_lines, */
  /*                        NILP (Vmenu_bar_mode) */
  /*                        ? make_fixnum (0) : make_fixnum (1), */
  /*                        NULL, NULL, RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qtab_bar_lines, */
  /*                        NILP (Vtab_bar_mode) */
  /*                        ? make_fixnum (0) : make_fixnum (1), */
  /*                        NULL, NULL, RES_TYPE_NUMBER); */
  /* gui_default_parameter (f, parms, Qtool_bar_lines, */
  /*                        NILP (Vtool_bar_mode) */
  /*                        ? make_fixnum (0) : make_fixnum (1), */
  /*                        NULL, NULL, RES_TYPE_NUMBER); */

  /* gui_default_parameter (f, parms, Qbuffer_predicate, Qnil, */
  /*                        "bufferPredicate", "BufferPredicate", */
  /*                        RES_TYPE_SYMBOL); */
  /* gui_default_parameter (f, parms, Qtitle, Qnil, */
  /*                        "title", "Title", RES_TYPE_STRING); */
  /* gui_default_parameter (f, parms, Qwait_for_wm, Qt, */
  /*                        "waitForWM", "WaitForWM", RES_TYPE_BOOLEAN); */
  /* gui_default_parameter (f, parms, Qtool_bar_position, */
  /*                        FRAME_TOOL_BAR_POSITION (f), 0, 0, RES_TYPE_SYMBOL); */
  /* gui_default_parameter (f, parms, Qinhibit_double_buffering, Qnil, */
  /*                        "inhibitDoubleBuffering", "InhibitDoubleBuffering", */
  /*                        RES_TYPE_BOOLEAN); */

  /* /\* Compute the size of the X window.  *\/ */
  /* window_prompting = gui_figure_window_size (f, parms, true, true); */

  /* tem = gui_display_get_arg (dpyinfo, parms, Qunsplittable, 0, 0, */
  /*                            RES_TYPE_BOOLEAN); */
  /* f->no_split = minibuffer_only || EQ (tem, Qt); */

  {
    /* setup wayland xdg-toplevel/surface and event listeners here */
    FRAME_OUTPUT_DATA (f)->surface = wl_compositor_create_surface (
      FRAME_DISPLAY_INFO (f)->compositor);
    wl_surface_set_user_data(FRAME_OUTPUT_DATA (f)->surface, f);
    /* FRAME_OUTPUT_DATA (f)->viewport */
    /*   = wp_viewporter_get_viewport (FRAME_DISPLAY_INFO (f) */
    /* 				      ->viewporter, */
    /* 				    FRAME_OUTPUT_DATA (f)->surface); */
    /* wp_viewport_set_source(FRAME_OUTPUT_DATA(f)->viewport, 0, 0, f->pixel_width, f->pixel_height);     */
    /* wp_viewport_set_destination(FRAME_OUTPUT_DATA(f)->viewport, f->pixel_width, f->pixel_height); */
    init_xdg_window (f, window_prompting);

    struct wl_region* region =
      wl_compositor_create_region(FRAME_DISPLAY_INFO (f)->compositor);
    wl_region_add(region, 0, 0, INT32_MAX, INT32_MAX);
    wl_surface_set_opaque_region(FRAME_OUTPUT_DATA (f)->surface, region);
    wl_region_destroy(region);

    FRAME_OUTPUT_DATA (f)->wait_for_configure = true;
    wl_surface_commit(FRAME_OUTPUT_DATA (f)->surface);

    /* struct wl_callback *cb */
    /*   = wl_surface_frame (FRAME_OUTPUT_DATA (f)->surface); */
    /* wl_callback_add_listener(cb, &wl_surface_frame_listener, f); */
  }

    /* Now consider the frame official.  */
  f->terminal->reference_count++;
  FRAME_DISPLAY_INFO (f)->reference_count++;
  Vframe_list = Fcons (frame, Vframe_list);

  /* Make sure windows on this frame appear in calls to next-window
     and similar functions.  */
  Vwindow_list = Qnil;

 return unbind_to (count, frame);

}

void
syms_of_wlcfns (void)
{
  defsubr (&Sx_hide_tip);
  defsubr (&Sxw_color_defined_p);
  defsubr (&Sxw_color_values);
  defsubr (&Sxw_display_color_p);
  defsubr (&Sx_display_grayscale_p);
  defsubr (&Swlc_open_connection);
  defsubr (&Sx_create_frame);
  defsubr (&Sx_display_planes);
  defsubr (&Sx_display_color_cells);

  DEFVAR_LISP ("x-cursor-fore-pixel", Vx_cursor_fore_pixel,
	       doc: /* SKIP: real doc in xfns.c.  */);
  Vx_cursor_fore_pixel = Qnil;
}
