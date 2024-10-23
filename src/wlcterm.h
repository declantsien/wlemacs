/* Definitions and headers for communication with Wayland protocol.
   Copyright (C) 2024 Free Software Foundation,
   Inc.

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

#ifndef WLCTERM_H
#define WLCTERM_H

#include "dispextern.h"
#include "termhooks.h"
# include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>
#include "xdg-shell-client-protocol.h"
#include "viewporter-client-protocol.h"
#include "xdg-decoration-protocol.h"

INLINE_HEADER_BEGIN

struct wlc_bitmap_record
{
  char *file;
  int refcount;
  int height, width, depth;
};

enum sync_mode
{
  NONE_	= 0,
  SWAP		= 1,
  COMMIT	= 2,
  FLUSH	= 3,
  QUERY	= 4,
};

/* Wayland code */
enum wlc_pointer_event_mask
{
  POINTER_EVENT_ENTER = 1 << 0,
  POINTER_EVENT_LEAVE = 1 << 1,
  POINTER_EVENT_MOTION = 1 << 2,
  POINTER_EVENT_BUTTON = 1 << 3,
  POINTER_EVENT_AXIS = 1 << 4,
  POINTER_EVENT_AXIS_SOURCE = 1 << 5,
  POINTER_EVENT_AXIS_STOP = 1 << 6,
  POINTER_EVENT_AXIS_DISCRETE = 1 << 7,
};

struct wlc_pointer_event
{
  uint32_t event_mask;
  wl_fixed_t surface_x, surface_y;
  uint32_t button, state;
  uint32_t time;
  uint32_t serial;
  struct
  {
    bool valid;
    wl_fixed_t value;
    int32_t discrete;
  } axes[2];
  uint32_t axis_source;
};

enum wlc_touch_event_mask
{
  TOUCH_EVENT_DOWN		= 1 << 0,
  TOUCH_EVENT_UP		= 1 << 1,
  TOUCH_EVENT_MOTION		= 1 << 2,
  TOUCH_EVENT_CANCEL		= 1 << 3,
  TOUCH_EVENT_SHAPE		= 1 << 4,
  TOUCH_EVENT_ORIENTATION	= 1 << 5,
};

struct wlc_touch_point
{
  bool valid;
  int32_t id;
  uint32_t event_mask;
  wl_fixed_t surface_x, surface_y;
  wl_fixed_t major, minor;
  wl_fixed_t orientation;
};

struct wlc_touch_event
{
  uint32_t event_mask;
  uint32_t time;
  uint32_t serial;
  struct wlc_touch_point points[10];
};

/* For each Wayland display, we have a structure that records
   information about it.  */

struct wlc_display_info
{
  /* Chain of all wlc_display_info structures.  */
  struct wlc_display_info *next;

  /* The generic display parameters corresponding to this X display. */
  struct terminal *terminal;

  struct wl_display* display;
  struct wl_registry* registry;
  struct wl_compositor* compositor;
  struct wl_subcompositor* subcompositor;
  struct xdg_wm_base* wm_base;
  struct wl_seat* seat;
  struct wl_pointer* pointer;
  struct wl_touch* touch;
  struct wl_keyboard* keyboard;
  struct wl_shm* shm;
  struct wl_cursor_theme* cursor_theme;
  struct wl_cursor* default_cursor;
  struct wl_surface* cursor_surface;
  struct wp_viewporter *viewporter;
  struct zxdg_decoration_manager_v1 *decoration_manager;

  struct wlc_pointer_event pointer_event;
  struct xkb_state *xkb_state;
  struct xkb_context *xkb_context;
  struct xkb_keymap *xkb_keymap;
  xkb_mod_mask_t control_mod_mask, shift_mod_mask;
  xkb_mod_mask_t meta_mod_mask, shift_lock_mask;
  xkb_mod_mask_t alt_mod_mask, super_mod_mask, hyper_mod_mask;
  struct wlc_touch_event touch_event;

  /* This is a cons cell of the form (NAME . FONT-LIST-CACHE).  */
  Lisp_Object name_list_element;

  /* List of predefined X colors.  */
  Lisp_Object color_map;

  /* Number of frames that are on this display.  */
  int reference_count;

  /* Minimum width over all characters in all fonts in font_table.  */
  int smallest_char_width;

  /* Minimum font height over all fonts in font_table.  */
  int smallest_font_height;

  /* Information about the range of text currently shown in
     mouse-face.  */
  Mouse_HLInfo mouse_highlight;

  /* The number of fonts opened for this display.  */
  int n_fonts;

  /* Pointer to bitmap records.  */
  struct wlc_bitmap_record *bitmaps;

  /* Allocated size of bitmaps field.  */
  ptrdiff_t bitmaps_size;

  /* Last used bitmap index.  */
  ptrdiff_t bitmaps_last;

  /* Dots per inch of the screen.  */
  double resx, resy;

  /* Number of planes on this screen.  */
  int n_planes;

  /* Mask of things that cause the mouse to be grabbed.  */
  int grabbed;

  /* The root window of this screen.  */
  Window root_window;

  /* The frame (if any) which has the X window that has keyboard focus.
     Zero if none.  This is examined by Ffocus_frame in xfns.c.  Note
     that a mere EnterNotify event can set this; if you need to know the
     last frame specified in a FocusIn or FocusOut event, use
     x_focus_event_frame.  */
  struct frame *x_focus_frame;

  /* The last frame mentioned in a FocusIn or FocusOut event.  This is
     separate from x_focus_frame, because whether or not LeaveNotify
     events cause us to lose focus depends on whether or not we have
     received a FocusIn event for it.

     This field is not used when the input extension is being
     utilized.  */
  struct frame *x_focus_event_frame;

  /* The frame which currently has the visual highlight, and should get
     keyboard input (other sorts of input have the frame encoded in the
     event).  It points to the X focus frame's selected window's
     frame.  It differs from x_focus_frame when we're using a global
     minibuffer.  */
  struct frame *highlight_frame;

  /* Time of last user interaction.  */
  Time last_user_time;

  /* The frame where the mouse was last time we reported a ButtonPress event.  */
  struct frame *last_mouse_frame;

  /* The frame where the mouse was last time we reported a mouse motion.  */
  struct frame *last_mouse_motion_frame;

  /* Position where the mouse was last time we reported a motion.
     This is a position on last_mouse_motion_frame.  It is used in
     some situations to report the mouse position as well: see
     XTmouse_position.  */
  int last_mouse_motion_x;
  int last_mouse_motion_y;
};

extern struct wlc_display_info *wlc_term_init (Lisp_Object);

/* This is a chain of structures for all the PGTK displays currently in use.  */
extern struct wlc_display_info *x_display_list;

struct wlc_output
{
  /* Inner perporty in Rust */
  void *gl_renderer;

  bool enable_compositor;
  enum sync_mode sync_mode;

  struct wl_surface* surface;
  struct xdg_surface* xdg_surface;
  struct xdg_toplevel* xdg_toplevel;
  struct wl_callback* callback;
  struct wp_viewport *viewport;
  struct zxdg_toplevel_decoration_v1 *decoration;
  bool wait_for_configure;
  uint32_t last_surface_frame;
  float offset;

  /* Default ASCII font of this frame.  */
  struct font *font;

  /* The baseline offset of the default ASCII font.  */
  int baseline_offset;

  /* If a fontset is specified for this frame instead of font, this
     value contains an ID of the fontset, else -1.  */
  int fontset;

  /* This is the Emacs structure for the X display this frame is on.  */
  struct wlc_display_info *display_info;

  /* True means our parent is another application's window
     and was explicitly specified.  */
  bool_bf explicit_parent : 1;

  /* The Wayland window used for this frame.
     May be zero while the frame object is being created
     and the X window has not yet been created.  */
  Window window_desc;

  /* The Wayland window that is the parent of this Wayland window.
     Usually this is a window that was made by the window manager,
     but it can be the root window, and it can be explicitly specified
     (see the explicit_parent field, below).  */
  Window parent_desc;

  /* Descriptor for the cursor in use for this window.  */
  Emacs_Cursor current_cursor;
  Emacs_Cursor text_cursor;
  Emacs_Cursor nontext_cursor;
  Emacs_Cursor modeline_cursor;
  Emacs_Cursor hand_cursor;
  Emacs_Cursor hourglass_cursor;
  Emacs_Cursor horizontal_drag_cursor;
  Emacs_Cursor vertical_drag_cursor;
  Emacs_Cursor left_edge_cursor;
  Emacs_Cursor top_left_corner_cursor;
  Emacs_Cursor top_edge_cursor;
  Emacs_Cursor top_right_corner_cursor;
  Emacs_Cursor right_edge_cursor;
  Emacs_Cursor bottom_right_corner_cursor;
  Emacs_Cursor bottom_edge_cursor;
  Emacs_Cursor bottom_left_corner_cursor;

};

extern struct wlc_display_info *check_wlc_display_info (Lisp_Object);

/* Symbol initializations implemented in each wlc sources. */
extern void syms_of_wlcterm (void);
extern void syms_of_wlcfns (void);
extern void syms_of_wlcmenu (void);
extern void syms_of_wlcselect (void);
extern void syms_of_wlcim (void);

extern unsigned long wlc_get_pixel (Emacs_Pix_Context, int, int);
extern void wlc_put_pixel (Emacs_Pix_Context, int, int, unsigned long);
extern void image_sync_to_pixmaps (struct frame *, struct image *);
extern void image_pixmap_draw_cross(struct frame *, Emacs_Pixmap, int, int, unsigned int,
  unsigned int, unsigned long);

/* Defined in wlcterm.c */
extern void wlc_make_frame_visible (struct frame *);
extern void wlc_make_frame_invisible (struct frame *);
extern void wlc_delete_terminal (struct terminal *);
extern bool wlc_handle_xdg_toplevel_close (struct frame *);

extern int xkb_to_emacs_modifiers (struct wlc_display_info *, int);
extern int emacs_to_xkb_modifiers (struct wlc_display_info *, intmax_t);

/* Return the X output data for frame F.  */
#define FRAME_WAYLAND_OUTPUT(f) ((f)->output_data.wlc)
#define FRAME_OUTPUT_DATA(f) FRAME_WAYLAND_OUTPUT (f)
#define FRAME_X_OUTPUT(f) FRAME_WAYLAND_OUTPUT (f)

/* Return the X window used for displaying data in frame F.  */
#define FRAME_WAYLAND_WINDOW(f) ((f)->output_data.wlc->window_desc)
#define FRAME_NATIVE_WINDOW(f) FRAME_WAYLAND_WINDOW (f)
#define FRAME_BASELINE_OFFSET(f) ((f)->output_data.wlc->baseline_offset)


#define FRAME_FONT(f) ((f)->output_data.wlc->font)
#define FRAME_FONTSET(f) ((f)->output_data.wlc->fontset)

/* This gives the wl_display_info structure for the display F is on.  */
#define FRAME_DISPLAY_INFO(f) ((f)->output_data.wlc->display_info)

extern int wlc_select (int fds_lim, fd_set *rfds, fd_set *wfds, fd_set *efds,
		      struct timespec const *timeout, sigset_t const *sigmask);

#include "webrender_ffi.h"
INLINE_HEADER_END

#endif /* WLCTERM_H */
