/* Wayland Communication module for terminals which understand
   the Wayland protocol.

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

/* Wayland window system support for GNU Emacs

   This file is part of the Wayland window system support for GNU Emacs.
   It contains subroutines comprising the redisplay interface, setting
   up scroll bars and widgets, and handling input.

   WAYLAND WINDOW SYSTEM

   Wayland is the next-generation display server for Unix-like systems,
   designed and built by the alumni of the venerable Xorg server, and is
   the best way to get your application windows onto your user's
   screens. Readers who have worked with X11 in the past will be
   pleasantly surprised by Wayland's improvements, and those who are new
   to graphics on Unix will find it a flexible and powerful system for
   building graphical applications and desktops.*/

#include <config.h>

#include "lisp.h"
#include "blockinput.h"

#include <stdio.h>
#include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>
#include "xdg-shell-client-protocol.h"
#include <stdbool.h>
#include <assert.h>
#include <signal.h>
#include <sys/mman.h>

#include "wlcterm.h"
#include "termhooks.h"
#include "keyboard.h"
#include "frame.h"

/* This is a chain of structures for all the Wayland displays currently in
   use.  */

struct wlc_display_info *x_display_list;

struct event_queue_t
{
  union buffered_input_event *q;
  int nr, cap;
};

/* A queue of events that will be read by the read_socket_hook.  */
static struct event_queue_t event_q;

static void
evq_enqueue (union buffered_input_event *ev)
{
  struct event_queue_t *evq = &event_q;
  struct frame *frame;
  struct wlc_display_info *dpyinfo;

  if (evq->cap == 0)
    {
      evq->cap = 4;
      evq->q = xmalloc (sizeof *evq->q * evq->cap);
    }

  if (evq->nr >= evq->cap)
    {
      evq->cap += evq->cap / 2;
      evq->q = xrealloc (evq->q, sizeof *evq->q * evq->cap);
    }

  evq->q[evq->nr++] = *ev;

  if (ev->ie.kind != SELECTION_REQUEST_EVENT
      && ev->ie.kind != SELECTION_CLEAR_EVENT)
    {
      frame = NULL;

      if (WINDOWP (ev->ie.frame_or_window))
	frame = WINDOW_XFRAME (XWINDOW (ev->ie.frame_or_window));

      if (FRAMEP (ev->ie.frame_or_window))
	frame = XFRAME (ev->ie.frame_or_window);

      if (frame)
	{
	  dpyinfo = FRAME_DISPLAY_INFO (frame);

	  if (dpyinfo->last_user_time < ev->ie.timestamp)
	    dpyinfo->last_user_time = ev->ie.timestamp;
	}
    }

  raise (SIGIO);
}

static int
evq_flush (struct input_event *hold_quit)
{
  struct event_queue_t *evq = &event_q;
  int n = 0;

  while (evq->nr > 0)
    {
      /* kbd_buffer_store_buffered_event may do longjmp, so
	 we need to shift event queue first and pass the event
	 to kbd_buffer_store_buffered_event so that events in
	 queue are not processed twice.  Bug#52941 */
      union buffered_input_event ev = evq->q[0];
      int i;
      for (i = 1; i < evq->nr; i++)
	evq->q[i - 1] = evq->q[i];
      evq->nr--;

      kbd_buffer_store_buffered_event (&ev, hold_quit);
      n++;
    }

  return n;
}

/* void */
/* xkb__print_keycode_state(const char *prefix, */
/*                           struct xkb_state *state, */
/*                           struct xkb_compose_state *compose_state, */
/*                           xkb_keycode_t keycode, */
/*                           enum xkb_consumed_mode consumed_mode, */
/*                           print_state_fields_mask_t fields) */
/* { */
/*     struct xkb_keymap *keymap; */

/*     xkb_keysym_t sym; */
/*     const xkb_keysym_t *syms; */
/*     int nsyms; */
/* #define BUFFER_SIZE MAX(XKB_COMPOSE_MAX_STRING_SIZE, XKB_KEYSYM_NAME_MAX_SIZE) */
/*     assert(XKB_KEYSYM_UTF8_MAX_SIZE <= BUFFER_SIZE); */
/*     char s[BUFFER_SIZE]; */
/* #undef BUFFER_SIZE */
/*     xkb_layout_index_t layout; */
/*     enum xkb_compose_status status; */

/*     keymap = xkb_state_get_keymap(state); */

/*     nsyms = xkb_state_key_get_syms(state, keycode, &syms); */

/*     if (nsyms <= 0) */
/*         return; */

/*     status = XKB_COMPOSE_NOTHING; */
/*     if (compose_state) */
/*         status = xkb_compose_state_get_status(compose_state); */

/*     if (status == XKB_COMPOSE_COMPOSING || status == XKB_COMPOSE_CANCELLED) */
/*         return; */

/*     if (status == XKB_COMPOSE_COMPOSED) { */
/*         sym = xkb_compose_state_get_one_sym(compose_state); */
/*         syms = &sym; */
/*         nsyms = 1; */
/*     } */
/*     else if (nsyms == 1) { */
/*         sym = xkb_state_key_get_one_sym(state, keycode); */
/*         syms = &sym; */
/*     } */

/*     if (prefix) */
/*         printf("%s", prefix); */

/*     print_keycode(keymap, "keycode [ ", keycode, " ] "); */

/* #ifdef ENABLE_PRIVATE_APIS */
/*     if (fields & PRINT_MODMAPS) { */
/*         print_key_modmaps(keymap, keycode); */
/*     } */
/* #endif */

/*     printf("keysyms [ "); */
/*     for (int i = 0; i < nsyms; i++) { */
/*         xkb_keysym_get_name(syms[i], s, sizeof(s)); */
/*         printf("%-*s ", XKB_KEYSYM_NAME_MAX_SIZE, s); */
/*     } */
/*     printf("] "); */

/*     if (fields & PRINT_UNICODE) { */
/*         if (status == XKB_COMPOSE_COMPOSED) */
/*             xkb_compose_state_get_utf8(compose_state, s, sizeof(s)); */
/*         else */
/*             xkb_state_key_get_utf8(state, keycode, s, sizeof(s)); */
/*         /\* HACK: escape single control characters from C0 set using the */
/*         * Unicode codepoint convention. Ideally we would like to escape */
/*         * any non-printable character in the string. */
/*         *\/ */
/*         if (!*s) { */
/*             printf("unicode [   ] "); */
/*         } else if (strlen(s) == 1 && (*s <= 0x1F || *s == 0x7F)) { */
/*             printf("unicode [ U+%04hX ] ", *s); */
/*         } else { */
/*             printf("unicode [ %s ] ", s); */
/*         } */
/*     } */

/*     layout = xkb_state_key_get_layout(state, keycode); */
/*     if (fields & PRINT_LAYOUT) { */
/*         printf("layout [ %s (%d) ] ", */
/*                xkb_keymap_layout_get_name(keymap, layout), layout); */
/*     } */

/*     printf("level [ %d ] ", */
/*            xkb_state_key_get_level(state, keycode, layout)); */

/*     printf("mods [ "); */
/*     for (xkb_mod_index_t mod = 0; mod < xkb_keymap_num_mods(keymap); mod++) { */
/*         if (xkb_state_mod_index_is_active(state, mod, */
/*                                           XKB_STATE_MODS_EFFECTIVE) <= 0) */
/*             continue; */
/*         if (xkb_state_mod_index_is_consumed2(state, keycode, mod, */
/*                                              consumed_mode)) */
/*             printf("-%s ", xkb_keymap_mod_get_name(keymap, mod)); */
/*         else */
/*             printf("%s ", xkb_keymap_mod_get_name(keymap, mod)); */
/*     } */
/*     printf("] "); */

/*     printf("leds [ "); */
/*     for (xkb_led_index_t led = 0; led < xkb_keymap_num_leds(keymap); led++) { */
/*         if (xkb_state_led_index_is_active(state, led) <= 0) */
/*             continue; */
/*         printf("%s ", xkb_keymap_led_get_name(keymap, led)); */
/*     } */
/*     printf("] "); */

/*     printf("\n"); */
/* } */

/* void */
/* xkb__print_state_changes(enum xkb_state_component changed) */
/* { */
/*     if (changed == 0) */
/*         return; */

/*     printf("changed [ "); */
/*     if (changed & XKB_STATE_LAYOUT_EFFECTIVE) */
/*         printf("effective-layout "); */
/*     if (changed & XKB_STATE_LAYOUT_DEPRESSED) */
/*         printf("depressed-layout "); */
/*     if (changed & XKB_STATE_LAYOUT_LATCHED) */
/*         printf("latched-layout "); */
/*     if (changed & XKB_STATE_LAYOUT_LOCKED) */
/*         printf("locked-layout "); */
/*     if (changed & XKB_STATE_MODS_EFFECTIVE) */
/*         printf("effective-mods "); */
/*     if (changed & XKB_STATE_MODS_DEPRESSED) */
/*         printf("depressed-mods "); */
/*     if (changed & XKB_STATE_MODS_LATCHED) */
/*         printf("latched-mods "); */
/*     if (changed & XKB_STATE_MODS_LOCKED) */
/*         printf("locked-mods "); */
/*     if (changed & XKB_STATE_LEDS) */
/*         printf("leds "); */
/*     printf("]\n"); */
/* } */

bool
wlc_handle_xdg_toplevel_close (struct frame *f)
{
  union buffered_input_event inev;

  EVENT_INIT (inev.ie);
  inev.ie.kind = NO_EVENT;
  inev.ie.arg = Qnil;

  if (f)
    {
      inev.ie.kind = DELETE_WINDOW_EVENT;
      XSETFRAME (inev.ie.frame_or_window, f);
    }

  if (inev.ie.kind != NO_EVENT)
    evq_enqueue (&inev);
  return true;
}

/* Move the mouse to position pixel PIX_X, PIX_Y relative to frame F.  */

void
frame_set_mouse_pixel_position (struct frame *f, int pix_x, int pix_y)
{
  /* TODO: Move the mouse to position pixel PIX_X, PIX_Y relative to frame F. */
  /* pgtk's impl is empty, seems not nessary*/
}

/* Convert a keysym to its name.  */

char *
get_keysym_name (int keysym)
{
  static char value[64];

  block_input ();
  if (xkb_keysym_get_name (keysym, value, sizeof value) == -1)
    value[0] = '\0';
  unblock_input ();

  return value;
}

static int
wlc_read_socket (struct terminal *terminal, struct input_event *hold_quit)
{
  /* printf("wlc_read_socket\n"); */

  int count;

  count = evq_flush (hold_quit);
  if (count > 0)
    {
      /* printf("event count %d", count); */
      return count;
    }

  struct wlc_display_info *dpyinfo = terminal->display_info.wlc;
  /* block_input (); */
  /* wl_display_dispatch (dpyinfo->display); */
  /* unblock_input (); */

  count = evq_flush (hold_quit);
  if (count > 0)
    {
      /* printf("event count %d", count); */
      return count;
    }

  /* printf("event count %d\n", 0); */
  return 0;
};

/* Change window state from mapped to iconified.  */

void
wlc_iconify_frame (struct frame *f)
{
  /* TODO */
}

/* This tries to wait until the frame is really visible, depending on
   the value of Vx_wait_for_event_timeout.
   However, if the window manager asks the user where to position
   the frame, this will return before the user finishes doing that.
   The frame will not actually be visible at that time,
   but it will become visible later when the window manager
   finishes with it.  */

void
wlc_make_frame_visible (struct frame *f)
{
}

/* `wl_select' is a `pselect' replacement.  To announce the intention
   to read from the fd, we need to call wl_display_prepare_read before
   pselect, and then actully read events with wl_display_read_events.

   Note that the fd should already be set in RFDS via
   add_keyboard_wait_descriptor.  */

int
wlc_select (int fds_lim, fd_set *rfds, fd_set *wfds, fd_set *efds,
	    struct timespec const *timeout, sigset_t const *sigmask)
{
  int retval;

  if (x_display_list && x_display_list->display)
    {
      while (wl_display_prepare_read (x_display_list->display) != 0)
	wl_display_dispatch_pending (x_display_list->display);
      wl_display_flush (x_display_list->display);
    }

  retval = pselect (fds_lim, rfds, wfds, efds, timeout, sigmask);

  if (x_display_list->display)
    {
      wl_display_read_events (x_display_list->display);
      wl_display_flush (x_display_list->display);
    }

  return retval;
}

/* Defined in wlcfns.c */
extern frame_parm_handler wlc_frame_parm_handlers[];

static void
wlc_default_font_parameter (struct frame *f, Lisp_Object parms)
{
  struct wlc_display_info *dpyinfo = FRAME_DISPLAY_INFO (f);
  Lisp_Object font_param = gui_display_get_arg (dpyinfo, parms, Qfont, NULL, NULL,
                                                RES_TYPE_STRING);
  Lisp_Object font = Qnil;
  if (BASE_EQ (font_param, Qunbound))
    font_param = Qnil;

  /* if (NILP (font_param)) */
  /*   /\* System font should take precedence over X resources.  We */
  /*      suggest this regardless of font-use-system-font because .emacs */
  /*      may not have been read yet.  Returning a font-spec is Haiku */
  /*      specific behavior.  *\/ */
  /*   font = font_open_by_spec (f, Ffont_get_system_font ()); */

  if (NILP (font))
    font = (!NILP (font_param)
	    ? font_param
	    : gui_display_get_arg (dpyinfo, parms, Qfont,
				   "font", "Font",
				   RES_TYPE_STRING));

  if (! FONTP (font) && ! STRINGP (font))
    {
      const char **names = (const char *[]) { "monospace-12",
					      "Noto Sans Mono-12",
					      "Source Code Pro-12",
					      NULL };
      int i;

      for (i = 0; names[i]; i++)
        {
          font
            = font_open_by_name (f, build_unibyte_string (names[i]));
          if (!NILP (font))
            break;
        }
      if (NILP (font))
        error ("No suitable font was found");
    }

  gui_default_parameter (f, parms, Qfont, font, "font", "Font",
                         RES_TYPE_STRING);
}

void
wlc_free_pixmap (struct frame *f, Emacs_Pixmap pixmap)
{
}

/* Return the pixel color value for color COLOR_NAME on frame F.  If F
   is a monochrome frame, return MONO_COLOR regardless of what ARG says.
   Signal an error if color can't be allocated.  */

int
wlc_decode_color (struct frame *f, Lisp_Object color_name, int mono_color)
{
  Emacs_Color cdef;

  CHECK_STRING (color_name);

  /* Return MONO_COLOR for monochrome frames.  */
  if (FRAME_DISPLAY_INFO (f)->n_planes == 1)
    return mono_color;

  /* wr_defined_color is responsible for coping with failures
     by looking for a near-miss.  */
  if (wr_defined_color (f, SSDATA (color_name), &cdef, true, 0))
    return cdef.pixel;

  signal_error ("Undefined color", color_name);
}

static void
wr_row_clip_bounds (struct window *w, struct glyph_row *row,
		  enum glyph_row_area area, Emacs_Rectangle *rect)
{
  int window_x, window_y, window_width;

  window_box (w, area, &window_x, &window_y, &window_width, 0);

  rect->x = window_x;
  rect->y = WINDOW_TO_FRAME_PIXEL_Y (w, max (0, row->y));
  rect->y = max (rect->y, window_y);
  rect->width = window_width;
  rect->height = row->visible_height;

}

/* Draw a hollow box cursor on window W in glyph row ROW.  */

static void
wlc_draw_hollow_cursor (struct window *w, struct glyph_row *row)
{
  struct frame *f = XFRAME (WINDOW_FRAME (w));
  int x, y, wd, h;
  struct glyph *cursor_glyph;

  /* Get the glyph the cursor is on.  If we can't tell because
     the current matrix is invalid or such, give up.  */
  cursor_glyph = get_phys_cursor_glyph (w);
  if (cursor_glyph == NULL)
    return;

  /* Compute frame-relative coordinates for phys cursor.  */
  get_phys_cursor_geometry (w, row, cursor_glyph, &x, &y, &h);
  wd = w->phys_cursor_width - 1;

  /* The foreground of cursor_gc is typically the same as the normal
     background color, which can cause the cursor box to be invisible.  */
  const Emacs_Rectangle clip_bounds;
  wr_row_clip_bounds (w, row, TEXT_AREA, &clip_bounds);

  /* When on R2L character, show cursor at the right edge of the
     glyph, unless the cursor box is as wide as the glyph or wider
     (the latter happens when x-stretch-cursor is non-nil).  */
  if ((cursor_glyph->resolved_level & 1) != 0
      && cursor_glyph->pixel_width > wd)
    {
      x += cursor_glyph->pixel_width - wd;
      if (wd > 0)
	wd -= 1;
    }

  wr_push_border_with_clip (FRAME_WR_DATA (f), FRAME_X_OUTPUT (f)->cursor_color,
			  x, y, wd, h - 1,
			  clip_bounds.x, clip_bounds.y,
			  clip_bounds.width, clip_bounds.height);
}

/* Draw a bar cursor on window W in glyph row ROW.

   Implementation note: One would like to draw a bar cursor with an
   angle equal to the one given by the font property XA_ITALIC_ANGLE.
   Unfortunately, I didn't find a font yet that has this property set.
   --gerd.  */

static void
wlc_draw_bar_cursor (struct window *w, struct glyph_row *row, int width,
		   enum text_cursor_kinds kind)
{
  struct frame *f = XFRAME (w->frame);
  struct glyph *cursor_glyph;

  /* If cursor is out of bounds, don't draw garbage.  This can happen
     in mini-buffer windows when switching between echo area glyphs
     and mini-buffer.  */
  cursor_glyph = get_phys_cursor_glyph (w);
  if (cursor_glyph == NULL)
    return;

  /* Experimental avoidance of cursor on xwidget.  */
  if (cursor_glyph->type == XWIDGET_GLYPH)
    return;

  /* If on an image, draw like a normal cursor.  That's usually better
     visible than drawing a bar, esp. if the image is large so that
     the bar might not be in the window.  */
  if (cursor_glyph->type == IMAGE_GLYPH)
    {
      struct glyph_row *r;
      r = MATRIX_ROW (w->current_matrix, w->phys_cursor.vpos);
      draw_phys_cursor_glyph (w, r, DRAW_CURSOR);
    }
  else
    {
      struct face *face = FACE_FROM_ID (f, cursor_glyph->face_id);
      unsigned long color;

      /* cairo_t *cr = pgtk_begin_cr_clip (f); */

      /* If the glyph's background equals the color we normally draw
         the bars cursor in, the bar cursor in its normal color is
         invisible.  Use the glyph's foreground color instead in this
         case, on the assumption that the glyph's colors are chosen so
         that the glyph is legible.  */
      if (face->background == FRAME_X_OUTPUT (f)->cursor_color)
	color = face->foreground;
      else
	color = FRAME_X_OUTPUT (f)->cursor_color;

      const Emacs_Rectangle clip_bounds;
      wr_row_clip_bounds (w, row, TEXT_AREA, &clip_bounds);

      if (kind == BAR_CURSOR)
	{
	  int x = WINDOW_TEXT_TO_FRAME_PIXEL_X (w, w->phys_cursor.x);

	  if (width < 0)
	    width = FRAME_CURSOR_WIDTH (f);
	  width = min (cursor_glyph->pixel_width, width);

	  w->phys_cursor_width = width;

	  /* If the character under cursor is R2L, draw the bar cursor
	     on the right of its glyph, rather than on the left.  */
	  if ((cursor_glyph->resolved_level & 1) != 0)
	    x += cursor_glyph->pixel_width - width;

	  wr_push_rect_with_clip (FRAME_WR_DATA (f), color, x,
				  WINDOW_TO_FRAME_PIXEL_Y (w, w->phys_cursor.y),
				  width, row->height,
				  clip_bounds.x, clip_bounds.y,
				  clip_bounds.width, clip_bounds.height);
	}
      else			/* HBAR_CURSOR */
	{
	  int dummy_x, dummy_y, dummy_h;
	  int x = WINDOW_TEXT_TO_FRAME_PIXEL_X (w, w->phys_cursor.x);

	  if (width < 0)
	    width = row->height;

	  width = min (row->height, width);

	  get_phys_cursor_geometry (w, row, cursor_glyph, &dummy_x,
				    &dummy_y, &dummy_h);

	  if ((cursor_glyph->resolved_level & 1) != 0
	      && cursor_glyph->pixel_width > w->phys_cursor_width - 1)
	    x += cursor_glyph->pixel_width - w->phys_cursor_width + 1;
	  wr_push_rect_with_clip (FRAME_WR_DATA (f), color, x,
				  WINDOW_TO_FRAME_PIXEL_Y (w, w->phys_cursor.y +
							   row->height - width),
				  w->phys_cursor_width - 1, width,
				  clip_bounds.x, clip_bounds.y,
				  clip_bounds.width, clip_bounds.height);
	}

    }
}

/* RIF: Draw cursor on window W.  */

static void
wlc_draw_window_cursor (struct window *w, struct glyph_row *glyph_row, int x,
			 int y, enum text_cursor_kinds cursor_type,
			 int cursor_width, bool on_p, bool active_p)
{
  struct frame *f = XFRAME (w->frame);

  if (on_p)
    {
      w->phys_cursor_type = cursor_type;
      w->phys_cursor_on_p = true;

      if (glyph_row->exact_window_width_line_p
	  && (glyph_row->reversed_p
	      ? (w->phys_cursor.hpos < 0)
	      : (w->phys_cursor.hpos >= glyph_row->used[TEXT_AREA])))
	{
	  glyph_row->cursor_in_fringe_p = true;
	  draw_fringe_bitmap (w, glyph_row, glyph_row->reversed_p);
	}
      else
	{
	  switch (cursor_type)
	    {
	    case HOLLOW_BOX_CURSOR:
	      wlc_draw_hollow_cursor (w, glyph_row);
	      break;

	    case FILLED_BOX_CURSOR:
	      draw_phys_cursor_glyph (w, glyph_row, DRAW_CURSOR);
	      break;

	    case BAR_CURSOR:
	      wlc_draw_bar_cursor (w, glyph_row, cursor_width, BAR_CURSOR);
	      break;

	    case HBAR_CURSOR:
	      wlc_draw_bar_cursor (w, glyph_row, cursor_width, HBAR_CURSOR);
	      break;

	    case NO_CURSOR:
	      w->phys_cursor_width = 0;
	      break;

	    default:
	      emacs_abort ();
	    }
	}

      if (w == XWINDOW (f->selected_window))
	{
	  int frame_x = (WINDOW_TO_FRAME_PIXEL_X (w, x)
			 + WINDOW_LEFT_FRINGE_WIDTH (w)
			 + WINDOW_LEFT_MARGIN_WIDTH (w));
	  int frame_y = WINDOW_TO_FRAME_PIXEL_Y (w, y);
	  /* pgtk_im_set_cursor_location (f, frame_x, frame_y, */
	  /* 			       w->phys_cursor_width, */
	  /* 			       w->phys_cursor_height); */
	}
    }

}


/* Set up use of Wayland before we make the first connection.  */

static struct redisplay_interface wlc_redisplay_interface = {
  wlc_frame_parm_handlers,
  gui_produce_glyphs,
  gui_write_glyphs,
  gui_insert_glyphs,
  gui_clear_end_of_line,
  wr_scroll_run,
  wr_after_update_window_line,
  NULL, /* update_window_begin */
  NULL, /* update_window_end   */
  wr_flush_display,
  gui_clear_window_mouse_face,
  gui_get_glyph_overhangs,
  gui_fix_overlapping_area,
  wr_draw_fringe_bitmap,
  0, /* define_fringe_bitmap */
  0, /* destroy_fringe_bitmap */
  0, /* compute_glyph_string_overhangs */
  wr_draw_glyph_string,
  0, /* wl_define_frame_cursor, */
  wr_clear_frame_area,
  wr_clear_under_internal_border,
  wlc_draw_window_cursor,
  wr_draw_vertical_window_border,
  wr_draw_window_divider,
  0, /* wl_shift_glyphs_for_insert, /\* Never called; see comment in
	function.  *\/   */
  0, /* wl_show_hourglass, */
  0, /* wl_hide_hourglass, */
  wlc_default_font_parameter,
};

static Lisp_Object
wlc_new_font (struct frame *f, Lisp_Object font_object, int fontset)
{
  struct font *font = XFONT_OBJECT (font_object);
  int font_ascent, font_descent;

  if (fontset < 0)
    fontset = fontset_from_font (font_object);
  FRAME_FONTSET (f) = fontset;

  if (FRAME_FONT (f) == font)
    {
      /* This font is already set in frame F.  There's nothing more to
         do.  */
      return font_object;
    }

  FRAME_FONT (f) = font;

  FRAME_BASELINE_OFFSET (f) = font->baseline_offset;
  FRAME_COLUMN_WIDTH (f) = font->average_width;
  get_font_ascent_descent (font, &font_ascent, &font_descent);
  FRAME_LINE_HEIGHT (f) = font_ascent + font_descent;

  /* We could use a more elaborate calculation here.  */
  FRAME_TAB_BAR_HEIGHT (f) = FRAME_TAB_BAR_LINES (f) * FRAME_LINE_HEIGHT (f);

  /* Compute the scroll bar width in character columns.  */
  if (FRAME_CONFIG_SCROLL_BAR_WIDTH (f) > 0)
    {
      int wid = FRAME_COLUMN_WIDTH (f);
      FRAME_CONFIG_SCROLL_BAR_COLS (f)
	= (FRAME_CONFIG_SCROLL_BAR_WIDTH (f) + wid - 1) / wid;
    }
  else
    {
      int wid = FRAME_COLUMN_WIDTH (f);
      FRAME_CONFIG_SCROLL_BAR_COLS (f) = (14 + wid - 1) / wid;
    }

  /* Compute the scroll bar height in character lines.  */
  if (FRAME_CONFIG_SCROLL_BAR_HEIGHT (f) > 0)
    {
      int height = FRAME_LINE_HEIGHT (f);
      FRAME_CONFIG_SCROLL_BAR_LINES (f)
	= (FRAME_CONFIG_SCROLL_BAR_HEIGHT (f) + height - 1) / height;
    }
  else
    {
      int height = FRAME_LINE_HEIGHT (f);
      FRAME_CONFIG_SCROLL_BAR_LINES (f) = (14 + height - 1) / height;
    }

  /* Now make the frame display the given font.  */
  if (FRAME_XDG_TOPLEVEL (f) != NULL)
    adjust_frame_size (f, FRAME_COLS (f) * FRAME_COLUMN_WIDTH (f),
		       FRAME_LINES (f) * FRAME_LINE_HEIGHT (f), 3,
		       false, Qfont);

  return font_object;
}

static void
wlc_update_begin (struct frame *f)
{
  wr_clear_under_internal_border (f);
}

/* End update of frame F.  This function is installed as a hook in
   update_end.  */

static void
wlc_update_end (struct frame *f)
{
  /* Mouse highlight may be displayed again.  */
  MOUSE_HL_INFO (f)->mouse_face_defer = false;
}


/* Destroy the X window of frame F.  */

static void
wlc_delete_frame (struct frame *f)
{
  struct wlc_display_info *dpyinfo = FRAME_DISPLAY_INFO (f);

  check_window_system (f);

  wr_free_frame_resources(f);

  /* /\* If a display connection is dead, don't try sending more */
  /*    commands to the X server.  *\/ */
  /* if (dpyinfo->display != 0) */
  /*   x_free_frame_resources (f); */

  /* xfree (f->output_data.x->saved_menu_event); */

/* #ifdef HAVE_X_I18N */
/*   if (f->output_data.x->preedit_chars) */
/*     xfree (f->output_data.x->preedit_chars); */
/* #endif */

/* #ifdef HAVE_XINPUT2 */
/* #ifdef HAVE_XINPUT2_1 */
/*   if (f->output_data.x->xi_masks) */
/*     XFree (f->output_data.x->xi_masks); */
/* #else */
/*   /\* This is allocated by us under very old versions of libXi; see */
/*      `setup_xi_event_mask'.  *\/ */
/*   if (f->output_data.x->xi_masks) */
/*     xfree (f->output_data.x->xi_masks); */
/* #endif */
  /* #endif */

  fprintf(stderr, "delete frame");

  xfree (f->output_data.wlc);
  f->output_data.wlc = NULL;

  dpyinfo->reference_count--;
}

/* Check if we need to resize the frame due to a fullscreen request.
   If so needed, resize the frame. */
static void
wlc_check_fullscreen (struct frame *f)
{
  Lisp_Object lval = Qnil;

  /* Setting fullscreen to nil doesn't do anything.  We could save the
     last non-fullscreen size and restore it, but it seems like a
     lot of work for this unusual case (no window manager running).  */

  if (f->want_fullscreen != FULLSCREEN_NONE)
    {
      int width = FRAME_PIXEL_WIDTH (f), height = FRAME_PIXEL_HEIGHT (f);
      struct wlc_display_info *dpyinfo = FRAME_DISPLAY_INFO (f);

      /* xdg_surface_unset_fullscreen(FRAME_OUTPUT_DATA (f)->xdg_toplevel); */
      /* xdg_surface_unset_maximized(window->xdg_surface); */

      switch (f->want_fullscreen)
        {
          /* No difference between these two when there is no WM */
        case FULLSCREEN_MAXIMIZED:
          lval = Qmaximized;
	  xdg_toplevel_set_maximized (FRAME_OUTPUT_DATA (f)->xdg_toplevel);
          break;
        case FULLSCREEN_BOTH:
          lval = Qfullboth;
	  xdg_toplevel_set_fullscreen (FRAME_OUTPUT_DATA (f)->xdg_toplevel, NULL);
          break;
        case FULLSCREEN_WIDTH:
          /* lval = Qfullwidth; */
          /* width = x_display_pixel_width (dpyinfo); */
	  /* height = height + FRAME_MENUBAR_HEIGHT (f); */
	  break;
        case FULLSCREEN_HEIGHT:
          /* lval = Qfullheight; */
          /* height = x_display_pixel_height (dpyinfo); */
	  break;
	default:
	  emacs_abort ();
        }

      /* XResizeWindow (FRAME_X_DISPLAY (f), FRAME_OUTER_WINDOW (f), */
      /* 		     width, height); */

      /* if (FRAME_VISIBLE_P (f)) */
      /* 	x_wait_for_event (f, ConfigureNotify); */
      /* else */
      /* 	change_frame_size (f, width, height, false, true, false); */
    }

  /* `x_net_wm_state' might have reset the fullscreen frame parameter,
     restore it. */
  store_frame_param (f, Qfullscreen, lval);
}

static void
wlc_handle_fullscreen (struct frame *f)
{
  block_input ();
  wlc_check_fullscreen (f);
  unblock_input ();
}

/* This function is called when the last frame on a display is deleted. */
void
wlc_delete_terminal (struct terminal *terminal)
{
  struct pgtk_display_info *dpyinfo = terminal->display_info.pgtk;

  /* Protect against recursive calls.  delete_frame in
     delete_terminal calls us back when it deletes our last frame.  */
  if (!terminal->name)
    return;

  block_input ();

  //TODO more to cleanup

  unblock_input ();
}

/* On frame F, translate pixel colors to RGB values for the NCOLORS
   colors in COLORS.  On W32, we no longer try to map colors to
   a palette.  */
void
wlc_query_colors (struct frame *f, Emacs_Color * colors, int ncolors)
{
  int i;

  for (i = 0; i < ncolors; i++)
    {
      unsigned long pixel = colors[i].pixel;
      /* Convert to a 16 bit value in range 0 - 0xffff. */
#define GetRValue(p) (((p) >> 16) & 0xff)
#define GetGValue(p) (((p) >> 8) & 0xff)
#define GetBValue(p) (((p) >> 0) & 0xff)
      colors[i].red = GetRValue (pixel) * 257;
      colors[i].green = GetGValue (pixel) * 257;
      colors[i].blue = GetBValue (pixel) * 257;
    }
}

/* Create a struct terminal, initialize it with the Wayland specific
   functions and make DISPLAY->TERMINAL point to it.  */

static struct terminal *
wlc_create_terminal (struct wlc_display_info *dpyinfo)
{
  struct terminal *terminal;

  terminal = create_terminal (output_wlc, &wlc_redisplay_interface);

  terminal->display_info.wlc = dpyinfo;
  dpyinfo->terminal = terminal;

  /* kboard is initialized in wl_term_init. */

  terminal->clear_frame_hook = wr_clear_frame;
/*   terminal->ins_del_lines_hook = x_ins_del_lines; */
/*   terminal->delete_glyphs_hook = x_delete_glyphs; */
/*   terminal->ring_bell_hook = XTring_bell; */
/*   terminal->toggle_invisible_pointer_hook = XTtoggle_invisible_pointer; */
  terminal->update_begin_hook = wlc_update_begin;
  terminal->update_end_hook = wlc_update_end;
  terminal->read_socket_hook = wlc_read_socket;
/*   terminal->frame_up_to_date_hook = XTframe_up_to_date; */
/* #ifdef HAVE_XDBE */
/*   terminal->buffer_flipping_unblocked_hook = XTbuffer_flipping_unblocked_hook; */
/* #endif */
  terminal->defined_color_hook = wr_defined_color;
/*   terminal->query_frame_background_color = x_query_frame_background_color; */
  terminal->query_colors = wlc_query_colors;
/*   terminal->mouse_position_hook = XTmouse_position; */
/*   terminal->get_focus_frame = x_get_focus_frame; */
/*   terminal->focus_frame_hook = x_focus_frame; */
/*   terminal->frame_rehighlight_hook = XTframe_rehighlight; */
/*   terminal->frame_raise_lower_hook = XTframe_raise_lower; */

  // Not possible on Wayland.
  /* terminal->frame_visible_invisible_hook =  */

  terminal->fullscreen_hook = wlc_handle_fullscreen;
/*   terminal->iconify_frame_hook = x_iconify_frame; */
/*   terminal->set_window_size_hook = x_set_window_size; */
/*   terminal->set_frame_offset_hook = x_set_offset; */
/*   terminal->set_frame_alpha_hook = x_set_frame_alpha; */
  terminal->set_new_font_hook = wlc_new_font;
/*   terminal->set_bitmap_icon_hook = x_bitmap_icon; */
  terminal->implicit_set_name_hook = wlc_implicitly_set_name;
/*   terminal->menu_show_hook = x_menu_show; */
/* #ifdef HAVE_EXT_MENU_BAR */
/*   terminal->activate_menubar_hook = x_activate_menubar; */
/* #endif */
/* #if defined (USE_X_TOOLKIT) || defined (USE_GTK) */
/*   terminal->popup_dialog_hook = xw_popup_dialog; */
/* #endif */
/*   terminal->change_tab_bar_height_hook = x_change_tab_bar_height; */
/* #ifndef HAVE_EXT_TOOL_BAR */
/*   terminal->change_tool_bar_height_hook = x_change_tool_bar_height; */
/* #endif */
/*   terminal->set_vertical_scroll_bar_hook = XTset_vertical_scroll_bar; */
/*   terminal->set_horizontal_scroll_bar_hook = XTset_horizontal_scroll_bar; */
/*   terminal->set_scroll_bar_default_width_hook = x_set_scroll_bar_default_width; */
/*   terminal->set_scroll_bar_default_height_hook = x_set_scroll_bar_default_height; */
/*   terminal->condemn_scroll_bars_hook = XTcondemn_scroll_bars; */
/*   terminal->redeem_scroll_bar_hook = XTredeem_scroll_bar; */
/*   terminal->judge_scroll_bars_hook = XTjudge_scroll_bars; */
/*   terminal->get_string_resource_hook = x_get_string_resource; */
  terminal->free_pixmap = wr_free_pixmap;
  terminal->delete_frame_hook = wlc_delete_frame;
  terminal->delete_terminal_hook = wlc_delete_terminal;
/*   terminal->toolkit_position_hook = x_toolkit_position; */
/* #ifdef HAVE_XINPUT2 */
/*   terminal->any_grab_hook = x_have_any_grab; */
/* #endif */
  /* Other hooks are NULL by default.  */

  return terminal;
}

/* Find Emacs frame with wl_surface surface */

static void
wl_pointer_enter(void *data, struct wl_pointer *wl_pointer,
		uint32_t serial, struct wl_surface *surface,
		wl_fixed_t surface_x, wl_fixed_t surface_y)
{
  struct frame *f = wl_surface_get_user_data(surface);
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->x_focus_frame = f;
  dpyinfo->highlight_frame = f;

  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_ENTER;
  dpyinfo->pointer_event.serial = serial;
  dpyinfo->pointer_event.surface_x = surface_x,
    dpyinfo->pointer_event.surface_y = surface_y;
}

static void
wl_pointer_leave(void *data, struct wl_pointer *wl_pointer,
		uint32_t serial, struct wl_surface *surface)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.serial = serial;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_LEAVE;
}

static void
wl_pointer_motion(void *data, struct wl_pointer *wl_pointer, uint32_t time,
		wl_fixed_t surface_x, wl_fixed_t surface_y)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_MOTION;
  dpyinfo->pointer_event.time = time;
  dpyinfo->pointer_event.surface_x = surface_x,
    dpyinfo->pointer_event.surface_y = surface_y;
}

static void
wl_pointer_button(void *data, struct wl_pointer *wl_pointer, uint32_t serial,
		uint32_t time, uint32_t button, uint32_t state)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_BUTTON;
  dpyinfo->pointer_event.time = time;
  dpyinfo->pointer_event.serial = serial;
  dpyinfo->pointer_event.button = button,
    dpyinfo->pointer_event.state = state;
}

static void
wl_pointer_axis(void *data, struct wl_pointer *wl_pointer, uint32_t time,
		uint32_t axis, wl_fixed_t value)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_AXIS;
  dpyinfo->pointer_event.time = time;
  dpyinfo->pointer_event.axes[axis].valid = true;
  dpyinfo->pointer_event.axes[axis].value = value;
}

/* One or more pointer events are available.

Multiple related events may be grouped together in a single frame.  Some examples:

- A drag that terminates outside the surface may send the Release and Leave events as one frame
- Movement from one surface to another may send the Enter and Leave events in one frame */
static void
wl_pointer_frame(void *data, struct wl_pointer *wl_pointer)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_pointer_event *event = &dpyinfo->pointer_event;
  /* fprintf(stderr, "pointer frame @ %d: ", event->time); */

  if (event->event_mask & POINTER_EVENT_ENTER) {
    /* fprintf(stderr, "entered %f, %f ", */
    /* 	    wl_fixed_to_double(event->surface_x), */
    /* 	    wl_fixed_to_double(event->surface_y)); */
  }

  if (event->event_mask & POINTER_EVENT_LEAVE) {
    /* fprintf(stderr, "leave"); */
  }

  if (event->event_mask & POINTER_EVENT_MOTION) {
    /* fprintf(stderr, "motion %f, %f ", */
    /* 	    wl_fixed_to_double(event->surface_x), */
    /* 	    wl_fixed_to_double(event->surface_y)); */
  }

  if (event->event_mask & POINTER_EVENT_BUTTON) {
    char *state = event->state == WL_POINTER_BUTTON_STATE_RELEASED ?
      "released" : "pressed";
    /* fprintf(stderr, "button %d %s ", event->button, state); */
  }

  uint32_t axis_events = POINTER_EVENT_AXIS
    | POINTER_EVENT_AXIS_SOURCE
    | POINTER_EVENT_AXIS_STOP
    | POINTER_EVENT_AXIS_DISCRETE;
  char *axis_name[2] = {
    [WL_POINTER_AXIS_VERTICAL_SCROLL] = "vertical",
    [WL_POINTER_AXIS_HORIZONTAL_SCROLL] = "horizontal",
  };
  char *axis_source[4] = {
    [WL_POINTER_AXIS_SOURCE_WHEEL] = "wheel",
    [WL_POINTER_AXIS_SOURCE_FINGER] = "finger",
    [WL_POINTER_AXIS_SOURCE_CONTINUOUS] = "continuous",
    [WL_POINTER_AXIS_SOURCE_WHEEL_TILT] = "wheel tilt",
  };
  if (event->event_mask & axis_events) {
    for (size_t i = 0; i < 2; ++i) {
      if (!event->axes[i].valid) {
	continue;
      }
      /* fprintf(stderr, "%s axis ", axis_name[i]); */
      if (event->event_mask & POINTER_EVENT_AXIS) {
	/* fprintf(stderr, "value %d ", wl_fixed_to_double( */
							/* event->axes[i].value)); */
      }
      if (event->event_mask & POINTER_EVENT_AXIS_DISCRETE) {
	/* fprintf(stderr, "discrete %d ", */
		/* event->axes[i].discrete); */
      }
      if (event->event_mask & POINTER_EVENT_AXIS_SOURCE) {
	/* fprintf(stderr, "via %s ", */
		/* axis_source[event->axis_source]); */
      }
      if (event->event_mask & POINTER_EVENT_AXIS_STOP) {
	/* fprintf(stderr, "(stopped) "); */
      }
    }
  }

  /* fprintf(stderr, "\n"); */
  memset(event, 0, sizeof(*event));
}

static void
wl_pointer_axis_source(void *data, struct wl_pointer *wl_pointer,
		       uint32_t axis_source)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_AXIS_SOURCE;
  dpyinfo->pointer_event.axis_source = axis_source;
}

static void
wl_pointer_axis_stop(void *data, struct wl_pointer *wl_pointer,
		     uint32_t time, uint32_t axis)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.time = time;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_AXIS_STOP;
  dpyinfo->pointer_event.axes[axis].valid = true;
}

static void
wl_pointer_axis_discrete(void *data, struct wl_pointer *wl_pointer,
			 uint32_t axis, int32_t discrete)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->pointer_event.event_mask |= POINTER_EVENT_AXIS_STOP;
  dpyinfo->pointer_event.axes[axis].valid = true;
  dpyinfo->pointer_event.axes[axis].discrete = discrete;
}

const static struct wl_pointer_listener wl_pointer_listener = {
  .enter = wl_pointer_enter,
  .leave = wl_pointer_leave,
  .motion = wl_pointer_motion,
  .button = wl_pointer_button,
  .axis = wl_pointer_axis,
  .frame = wl_pointer_frame,
  .axis_source = wl_pointer_axis_source,
  .axis_stop = wl_pointer_axis_stop,
  .axis_discrete = wl_pointer_axis_discrete,
};

/* Convert between the modifier bits X uses and the modifier bits
   Emacs uses.  */

int
xkb_to_emacs_modifiers (struct wlc_display_info *dpyinfo, int state)
{
  int mod_meta = meta_modifier;
  int mod_alt  = alt_modifier;
  int mod_hyper = hyper_modifier;
  int mod_super = super_modifier;
  Lisp_Object tem;

  tem = Fget (Vx_alt_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_alt = XFIXNUM (tem) & INT_MAX;
  tem = Fget (Vx_meta_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_meta = XFIXNUM (tem) & INT_MAX;
  tem = Fget (Vx_hyper_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_hyper = XFIXNUM (tem) & INT_MAX;
  tem = Fget (Vx_super_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_super = XFIXNUM (tem) & INT_MAX;

  return (  ((state & (dpyinfo->shift_mod_mask | dpyinfo->shift_lock_mask)) ? shift_modifier : 0)
            | ((state & dpyinfo->control_mod_mask)	? ctrl_modifier	: 0)
            | ((state & dpyinfo->meta_mod_mask)		? mod_meta	: 0)
            | ((state & dpyinfo->alt_mod_mask)		? mod_alt	: 0)
            | ((state & dpyinfo->super_mod_mask)	? mod_super	: 0)
            | ((state & dpyinfo->hyper_mod_mask)	? mod_hyper	: 0));
}


int
emacs_to_xkb_modifiers (struct wlc_display_info *dpyinfo, EMACS_INT state)
{
  EMACS_INT mod_meta = meta_modifier;
  EMACS_INT mod_alt  = alt_modifier;
  EMACS_INT mod_hyper = hyper_modifier;
  EMACS_INT mod_super = super_modifier;

  Lisp_Object tem;

  tem = Fget (Vx_alt_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_alt = XFIXNUM (tem);
  tem = Fget (Vx_meta_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_meta = XFIXNUM (tem);
  tem = Fget (Vx_hyper_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_hyper = XFIXNUM (tem);
  tem = Fget (Vx_super_keysym, Qmodifier_value);
  if (INTEGERP (tem)) mod_super = XFIXNUM (tem);


  return (  ((state & mod_alt)		? dpyinfo->alt_mod_mask   : 0)
            | ((state & mod_super)	? dpyinfo->super_mod_mask : 0)
            | ((state & mod_hyper)	? dpyinfo->hyper_mod_mask : 0)
            | ((state & shift_modifier)	? dpyinfo->shift_mod_mask : 0)
            | ((state & ctrl_modifier)	? dpyinfo->control_mod_mask : 0)
            | ((state & mod_meta)	? dpyinfo->meta_mod_mask  : 0));
}

static void
wl_keyboard_keymap(void *data, struct wl_keyboard *wl_keyboard,
		   uint32_t format, int32_t fd, uint32_t size)
{
  struct wlc_display_info *dpyinfo = data;

  /* assert(format == WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1); */
  switch (format)
    {
    case WL_KEYBOARD_KEYMAP_FORMAT_NO_KEYMAP:
      fprintf(stderr, "wlc: non-xkb compatible keymap");
      break;

    case WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1:
      // - TODO custom rules, models, layouts, variants and options (RMLVO) from elisp

      // - no custom from elisp

      char *map_shm = mmap(NULL, size, PROT_READ, MAP_SHARED, fd, 0);
      struct xkb_keymap *xkb_keymap =
	xkb_keymap_new_from_string(dpyinfo->xkb_context, map_shm,
				   XKB_KEYMAP_FORMAT_TEXT_V1, XKB_KEYMAP_COMPILE_NO_FLAGS);
      munmap(map_shm, size);
      close(fd);
      if (xkb_keymap == NULL) {
	fprintf(stderr, "wlc: invalid keymap");
      }

      struct xkb_state *xkb_state = xkb_state_new(xkb_keymap);

      xkb_keymap_unref(dpyinfo->xkb_keymap);
      xkb_state_unref(dpyinfo->xkb_state);
      dpyinfo->xkb_keymap = xkb_keymap;
      dpyinfo->xkb_state = xkb_state;

      dpyinfo->control_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, XKB_MOD_NAME_CTRL);
      dpyinfo->alt_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, "Meta");
      /* XKB_VMOD_NAME_META since 1.8.0(unreleased at this time of writing 2025/02/02) */
      dpyinfo->meta_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, XKB_MOD_NAME_ALT);
      dpyinfo->shift_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, XKB_MOD_NAME_SHIFT);
      dpyinfo->shift_lock_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, XKB_MOD_NAME_CAPS);
      dpyinfo->super_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, XKB_MOD_NAME_LOGO);
      /* XKB_VMOD_NAME_SUPER since 1.8.0(unreleased at this time of writing 2025/02/02) */
      dpyinfo->hyper_mod_mask =
	1 << xkb_map_mod_get_index (dpyinfo->xkb_keymap, "Super");

      /* If we couldn't find any meta keys, accept any alt keys as meta keys.  */
      if (!dpyinfo->meta_mod_mask)
	{
	  dpyinfo->meta_mod_mask = dpyinfo->alt_mod_mask;
	  dpyinfo->alt_mod_mask = 0;
	}

      /* If some keys are both alt and meta,
	 make them just meta, not alt.  */
      if (dpyinfo->alt_mod_mask & dpyinfo->meta_mod_mask)
	{
	  dpyinfo->alt_mod_mask &= ~dpyinfo->meta_mod_mask;
	}

      /* If some keys are both super and hyper, make them just super.
	 Many X servers are misconfigured so that super and hyper are both
	 Mod4, but most users have no hyper key.  */
      if (dpyinfo->hyper_mod_mask & dpyinfo->super_mod_mask)
	dpyinfo->hyper_mod_mask &= ~dpyinfo->super_mod_mask;

      break;

    default:
      emacs_abort ();
    }
}

static void
wl_keyboard_enter(void *data, struct wl_keyboard *wl_keyboard,
		  uint32_t serial, struct wl_surface *surface,
		  struct wl_array *keys)
{
  struct frame *f = wl_surface_get_user_data(surface);
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->x_focus_frame = f;
  dpyinfo->highlight_frame = f;
  /* fprintf(stderr, "keyboard enter; keys pressed are:\n"); */
  /* uint32_t *key; */
  /* wl_array_for_each(key, keys) { */
  /*   char buf[128]; */
  /*   xkb_keysym_t sym = xkb_state_key_get_one_sym( */
  /* 						 dpyinfo->xkb_state, *key + 8); */
  /*   xkb_keysym_get_name(sym, buf, sizeof(buf)); */
  /*   fprintf(stderr, "sym: %-12s (%d), ", buf, sym); */
  /*   xkb_state_key_get_utf8(dpyinfo->xkb_state, */
  /* 			   *key + 8, buf, sizeof(buf)); */
  /*   fprintf(stderr, "utf8: '%s'\n", buf); */
  /* } */
}

static void
wl_keyboard_leave(void *data, struct wl_keyboard *wl_keyboard,
		  uint32_t serial, struct wl_surface *surface)
{
  struct frame *f = wl_surface_get_user_data(surface);
  struct wlc_display_info *dpyinfo = data;
  if (dpyinfo->x_focus_frame == f) {
    dpyinfo->x_focus_frame = NULL;
  }
}

static void
wl_keyboard_key(void *data, struct wl_keyboard *wl_keyboard,
		uint32_t serial, uint32_t time, uint32_t key, uint32_t state)
{
  struct wlc_display_info *dpyinfo = data;
  uint32_t keycode = key + 8;
  union buffered_input_event inev;

  if (state == WL_KEYBOARD_KEY_STATE_RELEASED)
    return;
  if (!dpyinfo->x_focus_frame)
    return;

  EVENT_INIT (inev.ie);
  inev.ie.kind = NO_EVENT;
  inev.ie.arg = Qnil;

  xkb_keysym_t keysym, orig_keysym;
  uint32_t modifiers;

  keysym = xkb_state_key_get_one_sym (dpyinfo->xkb_state, keycode);
  modifiers = xkb_state_serialize_mods (dpyinfo->xkb_state,
					XKB_STATE_DEPRESSED
					| XKB_STATE_LATCHED);
  modifiers
    |= emacs_to_xkb_modifiers (dpyinfo, extra_keyboard_modifiers);

  /* Common for all keysym input events.  */
  XSETFRAME (inev.ie.frame_or_window, dpyinfo->x_focus_frame);
  inev.ie.modifiers = xkb_to_emacs_modifiers (dpyinfo, modifiers);
  inev.ie.timestamp = time;

  /* First deal with keysyms which have defined
     translations to characters.  */
  if (keysym >= 32 && keysym < 128)
    /* Avoid explicitly decoding each ASCII character.  */
    {
      inev.ie.kind = ASCII_KEYSTROKE_EVENT;
      inev.ie.code = keysym;
      goto done;
    }

  /* Keysyms directly mapped to Unicode characters.  */
  if (keysym >= 0x01000000 && keysym <= 0x0110FFFF)
    {
      if (keysym < 0x01000080)
	inev.ie.kind = ASCII_KEYSTROKE_EVENT;
      else
	inev.ie.kind = MULTIBYTE_CHAR_KEYSTROKE_EVENT;
      inev.ie.code = keysym & 0xFFFFFF;
      goto done;
    }

  /* Random non-modifier sorts of keysyms.  */
  if (((keysym >= XKB_KEY_BackSpace && keysym <= XKB_KEY_Escape)
       || keysym == XKB_KEY_Delete
       || (keysym >= XKB_KEY_ISO_Left_Tab
	   && keysym <= XKB_KEY_ISO_Enter)
       || (0xff50 <= keysym && keysym < 0xff60)
#if 0
       || 0xff60 <= keysym && keysym < VARIES
#endif
       || orig_keysym == XKB_KEY_dead_circumflex
       || orig_keysym == XKB_KEY_dead_grave
       || orig_keysym == XKB_KEY_dead_tilde
       || orig_keysym == XKB_KEY_dead_diaeresis
       || orig_keysym == XKB_KEY_dead_macron
       || orig_keysym == XKB_KEY_dead_acute
       || orig_keysym == XKB_KEY_dead_cedilla
       || orig_keysym == XKB_KEY_dead_breve
       || orig_keysym == XKB_KEY_dead_ogonek
       || orig_keysym == XKB_KEY_dead_caron
       || orig_keysym == XKB_KEY_dead_doubleacute
       || orig_keysym == XKB_KEY_dead_abovedot
       || (0xff80 <= keysym && keysym < 0xffbe)
       || (0xffbe <= keysym && keysym < 0xffe1)
       /* Any "vendor-specific" key is ok.  */
       || (orig_keysym & (1 << 28)))
#if 0
      || (keysym != XKB_KEY_NoSymbol && nbytes == 0))
    && ! (IsModifierKey (orig_keysym)
	  /* The symbols from XKB_KEY_ISO_Lock
	     to XKB_KEY_ISO_Last_Group_Lock
	     don't have real modifiers but
	     should be treated similarly to
	     Mode_switch by Emacs. */
	  || (XKB_KEY_ISO_Lock <= orig_keysym
	      && orig_keysym <= XKB_KEY_ISO_Last_Group_Lock)
	  )
#endif
      )
	{
	  /* make_lispy_event will convert this to a symbolic
	     key.  */
	  inev.ie.kind = NON_ASCII_KEYSTROKE_EVENT;
	  inev.ie.code = keysym;
	  goto done;
	}

 done:
   if (inev.ie.kind != NO_EVENT)
     evq_enqueue (&inev);

  char buf[128];

  /* xkb_keysym_get_name(keysym, buf, sizeof(buf)); */
  /* const char *action = */
  /*   state == WL_KEYBOARD_KEY_STATE_PRESSED ? "press" : "release"; */
  /* fprintf(stderr, "key %s: sym: %-12s (%d), ", action, buf, keysym); */
  /* xkb_state_key_get_utf8(dpyinfo->xkb_state, keycode, */
  /* 			 buf, sizeof(buf)); */
  /* fprintf(stderr, "utf8: '%s'\n", buf); */
}

static void
wl_keyboard_modifiers(void *data, struct wl_keyboard *wl_keyboard,
		      uint32_t serial, uint32_t mods_depressed,
		      uint32_t mods_latched, uint32_t mods_locked,
		      uint32_t group)
{
  struct wlc_display_info *dpyinfo = data;
  xkb_state_update_mask(dpyinfo->xkb_state,
			mods_depressed, mods_latched, mods_locked, 0, 0, group);
}

static void
wl_keyboard_repeat_info(void *data, struct wl_keyboard *wl_keyboard,
			int32_t rate, int32_t delay)
{
  /* TODO: check winit impl */
}

const static struct wl_keyboard_listener wl_keyboard_listener =
  {
  .keymap = wl_keyboard_keymap,
  .enter = wl_keyboard_enter,
  .leave = wl_keyboard_leave,
  .key = wl_keyboard_key,
  .modifiers = wl_keyboard_modifiers,
  .repeat_info = wl_keyboard_repeat_info,
};

static struct wlc_touch_point *
get_touch_point(struct wlc_display_info *dpyinfo, int32_t id)
{
  struct wlc_touch_event *touch = &dpyinfo->touch_event;
  const size_t nmemb = sizeof(touch->points) / sizeof(struct wlc_touch_point);
  int invalid = -1;
  for (size_t i = 0; i < nmemb; ++i) {
    if (touch->points[i].id == id) {
      return &touch->points[i];
    }
    if (invalid == -1 && !touch->points[i].valid) {
      invalid = i;
    }
  }
  if (invalid == -1) {
    return NULL;
  }
  touch->points[invalid].valid = true;
  touch->points[invalid].id = id;
  return &touch->points[invalid];
}

static void
wl_touch_down(void *data, struct wl_touch *wl_touch, uint32_t serial,
	      uint32_t time, struct wl_surface *surface, int32_t id,
	      wl_fixed_t x, wl_fixed_t y)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_point *point = get_touch_point(dpyinfo, id);
  if (point == NULL) {
    return;
  }
  point->event_mask |= TOUCH_EVENT_UP;
  point->surface_x = wl_fixed_to_double(x),
    point->surface_y = wl_fixed_to_double(y);
  dpyinfo->touch_event.time = time;
  dpyinfo->touch_event.serial = serial;
}

static void
wl_touch_up(void *data, struct wl_touch *wl_touch, uint32_t serial,
	    uint32_t time, int32_t id)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_point *point = get_touch_point(dpyinfo, id);
  if (point == NULL) {
    return;
  }
  point->event_mask |= TOUCH_EVENT_UP;
}

static void
wl_touch_motion(void *data, struct wl_touch *wl_touch, uint32_t time,
		int32_t id, wl_fixed_t x, wl_fixed_t y)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_point *point = get_touch_point(dpyinfo, id);
  if (point == NULL) {
    return;
  }
  point->event_mask |= TOUCH_EVENT_MOTION;
  point->surface_x = x, point->surface_y = y;
  dpyinfo->touch_event.time = time;
}

static void
wl_touch_frame(void *data, struct wl_touch *wl_touch)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_event *touch = &dpyinfo->touch_event;
  const size_t nmemb = sizeof(touch->points) / sizeof(struct wlc_touch_point);
  fprintf(stderr, "touch event @ %d:\n", touch->time);

  for (size_t i = 0; i < nmemb; ++i) {
    struct wlc_touch_point *point = &touch->points[i];
    if (!point->valid) {
      continue;
    }
    fprintf(stderr, "point %d: ", touch->points[i].id);

    if (point->event_mask & TOUCH_EVENT_DOWN) {
      fprintf(stderr, "down %f,%f ",
	      wl_fixed_to_double(point->surface_x),
	      wl_fixed_to_double(point->surface_y));
    }

    if (point->event_mask & TOUCH_EVENT_UP) {
      fprintf(stderr, "up ");
    }

    if (point->event_mask & TOUCH_EVENT_MOTION) {
      fprintf(stderr, "motion %f,%f ",
	      wl_fixed_to_double(point->surface_x),
	      wl_fixed_to_double(point->surface_y));
    }

    if (point->event_mask & TOUCH_EVENT_SHAPE) {
      fprintf(stderr, "shape %fx%f ",
	      wl_fixed_to_double(point->major),
	      wl_fixed_to_double(point->minor));
    }

    if (point->event_mask & TOUCH_EVENT_ORIENTATION) {
      fprintf(stderr, "orientation %f ",
	      wl_fixed_to_double(point->orientation));
    }

    point->valid = false;
    fprintf(stderr, "\n");
  }

  if (touch->event_mask & TOUCH_EVENT_CANCEL) {
    fprintf(stderr, "touch cancelled\n");
  }
}

static void
wl_touch_cancel(void *data, struct wl_touch *wl_touch)
{
  struct wlc_display_info *dpyinfo = data;
  dpyinfo->touch_event.event_mask |= TOUCH_EVENT_CANCEL;
}

static void
wl_touch_shape(void *data, struct wl_touch *wl_touch,
	       int32_t id, wl_fixed_t major, wl_fixed_t minor)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_point *point = get_touch_point(dpyinfo, id);
  if (point == NULL) {
    return;
  }
  point->event_mask |= TOUCH_EVENT_SHAPE;
  point->major = major, point->minor = minor;
}

static void
wl_touch_orientation(void *data, struct wl_touch *wl_touch,
		     int32_t id, wl_fixed_t orientation)
{
  struct wlc_display_info *dpyinfo = data;
  struct wlc_touch_point *point = get_touch_point(dpyinfo, id);
  if (point == NULL) {
    return;
  }
  point->event_mask |= TOUCH_EVENT_ORIENTATION;
  point->orientation = orientation;
}

const static struct wl_touch_listener wl_touch_listener = {
  .down = wl_touch_down,
  .up = wl_touch_up,
  .motion = wl_touch_motion,
  .frame = wl_touch_frame,
  .cancel = wl_touch_cancel,
  .shape = wl_touch_shape,
  .orientation = wl_touch_orientation,
};

static void
wl_seat_capabilities(void *data, struct wl_seat *wl_seat, uint32_t capabilities)
{
       /* struct client_state *state = data; */
       /* TODO */
  struct wlc_display_info *dpyinfo = data;

  bool have_pointer = capabilities & WL_SEAT_CAPABILITY_POINTER,
    have_keyboard = capabilities & WL_SEAT_CAPABILITY_KEYBOARD,
    have_touch = capabilities & WL_SEAT_CAPABILITY_TOUCH;

  if (have_pointer && dpyinfo->pointer == NULL) {
    dpyinfo->pointer = wl_seat_get_pointer(dpyinfo->seat);
    wl_pointer_add_listener(dpyinfo->pointer,
			    &wl_pointer_listener, dpyinfo);
  } else if (!have_pointer && dpyinfo->pointer != NULL) {
    wl_pointer_release(dpyinfo->pointer);
    dpyinfo->pointer = NULL;
  }

  if (have_keyboard && dpyinfo->keyboard == NULL) {
    dpyinfo->keyboard = wl_seat_get_keyboard(dpyinfo->seat);
    wl_keyboard_add_listener(dpyinfo->keyboard,
			     &wl_keyboard_listener, dpyinfo);
  } else if (!have_keyboard && dpyinfo->keyboard != NULL) {
    wl_keyboard_release(dpyinfo->keyboard);
    dpyinfo->keyboard = NULL;
  }

  if (have_touch && dpyinfo->touch == NULL) {
    dpyinfo->touch = wl_seat_get_touch(dpyinfo->seat);
    wl_touch_add_listener(dpyinfo->touch,
			  &wl_touch_listener, dpyinfo);
  } else if (!have_touch && dpyinfo->touch != NULL) {
    wl_touch_release(dpyinfo->touch);
    dpyinfo->touch = NULL;
  }
}

static void
wl_seat_name(void *data, struct wl_seat *wl_seat, const char *name)
{
       fprintf(stderr, "seat name: %s\n", name);
}

static const struct wl_seat_listener wl_seat_listener = {
       .capabilities = wl_seat_capabilities,
       .name = wl_seat_name,
};

static void
xdg_wm_base_ping(void *data, struct xdg_wm_base *xdg_wm_base, uint32_t serial)
{
    xdg_wm_base_pong(xdg_wm_base, serial);
}

static const struct xdg_wm_base_listener xdg_wm_base_listener = {
    .ping = xdg_wm_base_ping,
};

static void
registry_handle_global(void *data, struct wl_registry *registry,
		uint32_t name, const char *interface, uint32_t version)
{
  struct wlc_display_info *dpyinfo = data;
  if (strcmp(interface, wl_shm_interface.name) == 0) {
    dpyinfo->shm = wl_registry_bind(
				    registry, name, &wl_shm_interface, 1);
  } else if (strcmp(interface, wl_compositor_interface.name) == 0) {
    dpyinfo->compositor = wl_registry_bind(
					   registry, name, &wl_compositor_interface, 4);
  } else if (strcmp(interface, wp_viewporter_interface.name) == 0) {
    dpyinfo->viewporter = wl_registry_bind(
					   registry, name, &wp_viewporter_interface, 1);
  } else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
    dpyinfo->wm_base = wl_registry_bind(
					registry, name, &xdg_wm_base_interface, 1);
    xdg_wm_base_add_listener(dpyinfo->wm_base,
			     &xdg_wm_base_listener, dpyinfo);
  } else if (strcmp(interface, wl_seat_interface.name) == 0) {
    dpyinfo->seat = wl_registry_bind(
				     registry, name, &wl_seat_interface, 7);
    wl_seat_add_listener(dpyinfo->seat,
			 &wl_seat_listener, dpyinfo);
  } else if (strcmp(interface, zxdg_decoration_manager_v1_interface.name) == 0) {
    dpyinfo->decoration_manager = wl_registry_bind(
						   registry, name, &zxdg_decoration_manager_v1_interface, 1);
  }
}

static void
registry_handle_global_remove(void *data, struct wl_registry *registry,
		uint32_t name)
{
	// This space deliberately left blank
}

static const struct wl_registry_listener
registry_listener = {
	.global = registry_handle_global,
	.global_remove = registry_handle_global_remove,
};

/* Test whether two display-name strings agree up to the dot that separates
   the screen number from the server number.  */
static bool
same_x_server (const char *name1, const char *name2)
{
  bool seen_colon = false;
  Lisp_Object sysname = Fsystem_name ();
  const char *system_name = SSDATA (sysname);
  ptrdiff_t system_name_length = SBYTES (sysname);
  ptrdiff_t length_until_period = 0;

  while (system_name[length_until_period] != 0
	 && system_name[length_until_period] != '.')
    length_until_period++;

  /* Treat `unix' like an empty host name.  */
  if (!strncmp (name1, "unix:", 5))
    name1 += 4;
  if (!strncmp (name2, "unix:", 5))
    name2 += 4;
  /* Treat this host's name like an empty host name.  */
  if (!strncmp (name1, system_name, system_name_length)
      && name1[system_name_length] == ':')
    name1 += system_name_length;
  if (!strncmp (name2, system_name, system_name_length)
      && name2[system_name_length] == ':')
    name2 += system_name_length;
  /* Treat this host's domainless name like an empty host name.  */
  if (!strncmp (name1, system_name, length_until_period)
      && name1[length_until_period] == ':')
    name1 += length_until_period;
  if (!strncmp (name2, system_name, length_until_period)
      && name2[length_until_period] == ':')
    name2 += length_until_period;

  for (; *name1 != '\0' && *name1 == *name2; name1++, name2++)
    {
      if (*name1 == ':')
	seen_colon = true;
      if (seen_colon && *name1 == '.')
	return true;
    }
  return (seen_colon
	  && (*name1 == '.' || *name1 == '\0')
	  && (*name2 == '.' || *name2 == '\0'));
}

static void
wlc_initialize_display_info (struct wlc_display_info *dpyinfo)
/* --------------------------------------------------------------------------
      Initialize global info and storage for display.
   -------------------------------------------------------------------------- */
{
  dpyinfo->resx = 96;
  dpyinfo->resy = 96;
  /* dpyinfo->color_p = 1; */
  dpyinfo->n_planes = 32;
  dpyinfo->root_window = 42;	/* a placeholder.. */
  dpyinfo->highlight_frame = dpyinfo->x_focus_frame = NULL;
  dpyinfo->n_fonts = 0;
  dpyinfo->smallest_font_height = 1;
  dpyinfo->smallest_char_width = 1;

  reset_mouse_highlight (&dpyinfo->mouse_highlight);
}

/* Open a connection to Wayland display DISPLAY_NAME, and return the
   structure that describes the open display.  If obtaining the Wayland
   connection or display fails, return NULL.  Signal an error if opening
   the display itself failed.  */

struct wlc_display_info *
wlc_term_init (Lisp_Object display_name)
{
  struct terminal *terminal;
  struct wlc_display_info *dpyinfo;
  struct wl_display *display;
  Lisp_Object color_file, color_map;
  static int x_initialized = 0;

  if (!x_initialized)
    {
      /* any_help_event_p = false; */


/* #ifdef USE_CAIRO */
/*       gui_init_fringe (&pgtk_redisplay_interface); */
/* #endif */

      ++x_initialized;
    }

  if (NILP (display_name)) {
    display = wl_display_connect(NULL);
  } else {
    CHECK_STRING (display_name);
    display = wl_display_connect(SSDATA (display_name));
  }

  if (!display)
    return NULL;

  /* We have definitely succeeded.  Record the new connection.  */

  dpyinfo = xzalloc (sizeof *dpyinfo);
  wlc_initialize_display_info (dpyinfo);
  terminal = wlc_create_terminal (dpyinfo);

  {
    struct wlc_display_info *share;

    for (share = x_display_list; share; share = share->next)
      if (same_x_server (SSDATA (XCAR (share->name_list_element)), SSDATA (display_name)))
	break;
    if (share)
      terminal->kboard = share->terminal->kboard;
    else
      {
	terminal->kboard = allocate_kboard (Qwlc);

	/* Don't let the initial kboard remain current longer than necessary.
	   That would cause problems if a file loaded on startup tries to
	   prompt in the mini-buffer.  */
	if (current_kboard == initial_kboard)
	  current_kboard = terminal->kboard;
      }
    terminal->kboard->reference_count++;
  }

  /* binds wayland registry global and save it dpyinfo */
  dpyinfo->registry = wl_display_get_registry (display);
  wl_registry_add_listener (dpyinfo->registry, &registry_listener,
			    dpyinfo);
  dpyinfo->xkb_context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
  wl_display_roundtrip(display);
  wl_display_dispatch (display);

  assert(dpyinfo->compositor);
  assert (dpyinfo->wm_base);
  assert (dpyinfo->decoration_manager);
  /* assert(display->subcompositor); */

  /* Put this display on the chain.  */
  dpyinfo->next = x_display_list;
  x_display_list = dpyinfo;

  dpyinfo->name_list_element = Fcons (display_name, Qnil);

  dpyinfo->display = display;
  dpyinfo->connection = wl_display_get_fd (dpyinfo->display);

  Fset_input_interrupt_mode (Qt);
  /* baud_rate = 19200; */
  if (dpyinfo->connection >= 0) {
    add_keyboard_wait_descriptor (dpyinfo->connection);
    init_sigio(dpyinfo->connection);

#ifdef F_SETOWN
    fcntl (dpyinfo->connection, F_SETOWN, getpid ());
#endif /* ! defined (F_SETOWN) */
  }

  /* Set the name of the terminal. */
  terminal->name = xlispstrdup (display_name);

  return dpyinfo;
}

void
syms_of_wlcterm (void)
{
  /* Tell Emacs about this window system.  */
  Fprovide (Qwlc, Qnil);

  DEFVAR_LISP ("x-keysym-table", Vx_keysym_table,
	       doc: /* Hash table of character codes indexed by X keysym codes.  */);
  Vx_keysym_table = make_hash_table (&hashtest_eql, 900, Weak_None);

  DEFVAR_BOOL ("x-use-underline-position-properties",
	       x_use_underline_position_properties,
	       doc: /* SKIP: real doc in xterm.c.  */);
  x_use_underline_position_properties = 1;

  DEFVAR_BOOL ("x-underline-at-descent-line",
	       x_underline_at_descent_line,
	       doc: /* SKIP: real doc in xterm.c.  */);
  x_underline_at_descent_line = 0;

  DEFSYM (Qmodifier_value, "modifier-value");
  DEFSYM (Qctrl, "ctrl");
  Fput (Qctrl, Qmodifier_value, make_fixnum (ctrl_modifier));
  DEFSYM (Qalt, "alt");
  Fput (Qalt, Qmodifier_value, make_fixnum (alt_modifier));
  DEFSYM (Qhyper, "hyper");
  Fput (Qhyper, Qmodifier_value, make_fixnum (hyper_modifier));
  DEFSYM (Qmeta, "meta");
  Fput (Qmeta, Qmodifier_value, make_fixnum (meta_modifier));
  DEFSYM (Qsuper, "super");
  Fput (Qsuper, Qmodifier_value, make_fixnum (super_modifier));

  DEFVAR_LISP ("x-alt-keysym", Vx_alt_keysym,
    doc: /* Which keys Emacs uses for the alt modifier.
This should be one of the symbols `alt', `hyper', `meta', `super'.
For example, `alt' means use the Alt_L and Alt_R keysyms.  The default
is nil, which is the same as `alt'.  */);
  Vx_alt_keysym = Qnil;

  DEFVAR_LISP ("x-hyper-keysym", Vx_hyper_keysym,
    doc: /* Which keys Emacs uses for the hyper modifier.
This should be one of the symbols `alt', `hyper', `meta', `super'.
For example, `hyper' means use the Hyper_L and Hyper_R keysyms.  The
default is nil, which is the same as `hyper'.  */);
  Vx_hyper_keysym = Qnil;

  DEFVAR_LISP ("x-meta-keysym", Vx_meta_keysym,
    doc: /* Which keys Emacs uses for the meta modifier.
This should be one of the symbols `alt', `hyper', `meta', `super'.
For example, `meta' means use the Meta_L and Meta_R keysyms.  The
default is nil, which is the same as `meta'.  */);
  Vx_meta_keysym = Qnil;

  DEFVAR_LISP ("x-super-keysym", Vx_super_keysym,
    doc: /* Which keys Emacs uses for the super modifier.
This should be one of the symbols `alt', `hyper', `meta', `super'.
For example, `super' means use the Super_L and Super_R keysyms.  The
default is nil, which is the same as `super'.  */);
  Vx_super_keysym = Qnil;
}

void
mark_wlcterm (void)
{}
