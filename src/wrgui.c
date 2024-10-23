#include <config.h>

#include "lisp.h"
#include "blockinput.h"

#include <stdio.h>
#include <wayland-client.h>
#include "xdg-shell-client-protocol.h"
#include <stdbool.h>
#include <assert.h>

#include "wlcterm.h"
#include "termhooks.h"
#include "keyboard.h"
#include "frame.h"

void
wr_clear_frame (struct frame *)
{

}

void
wr_update_end (struct frame *f) {
}

int
wr_parse_color (struct frame *f, const char *color_name,
		     Emacs_Color *color)
{
  unsigned short r, g, b;
  Lisp_Object tem, tem1;
  unsigned long lisp_color;

  if (parse_color_spec (color_name, &r, &g, &b))
    {
      color->red = r;
      color->green = g;
      color->blue = b;

      return 1;
    }

  tem = x_display_list->color_map;
  for (; CONSP (tem); tem = XCDR (tem))
    {
      tem1 = XCAR (tem);

      if (CONSP (tem1)
	  && !xstrcasecmp (SSDATA (XCAR (tem1)), color_name))
	{
	  lisp_color = XFIXNUM (XCDR (tem1));
	  color->red = RED_FROM_ULONG (lisp_color) * 257;
	  color->green = GREEN_FROM_ULONG (lisp_color) * 257;
	  color->blue = BLUE_FROM_ULONG (lisp_color) * 257;
	  return 1;
	}
    }

  return 0;
}

/* Decide if color named COLOR_NAME is valid for use on frame F.  If
   so, return the RGB values in COLOR.  If ALLOC_P, allocate the
   color.  Value is false if COLOR_NAME is invalid, or no color could
   be allocated.  MAKE_INDEX is some mysterious argument used on
   NS. */

bool
wr_defined_color (struct frame *f, const char *color_name,
		       Emacs_Color *color, bool alloc_p,
		       bool make_index)
{
  bool success_p;

  success_p = false;

  block_input ();
  success_p = wr_parse_color (f, color_name, color);
  /* if (success_p && alloc_p) */
  /*   success_p = android_alloc_nearest_color (f, color); */
  unblock_input ();

  return success_p;
}

void
wr_free_pixmap (struct frame *f, Emacs_Pixmap pixmap)
{
}

/* Return the pixel color value for color COLOR_NAME on frame F.  If F
   is a monochrome frame, return MONO_COLOR regardless of what ARG says.
   Signal an error if color can't be allocated.  */

int
wr_decode_color (struct frame *f, Lisp_Object color_name, int mono_color)
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


void wr_scroll_run (struct window *w, struct run *run) {
}

void wr_update_window_begin (struct window *) {
}
void wr_update_window_end (struct window *, bool, bool) {
}
void wr_after_update_window_line (struct window *w,
				  struct glyph_row *desired_row) {
}
void wr_flush_display (struct frame *f) {
}
void
wr_draw_fringe_bitmap (struct window *w, struct glyph_row *row,
		       struct draw_fringe_bitmap_params *p)
{
}
void
wr_draw_glyph_string (struct glyph_string *s) {
}
void wr_clear_frame_area (struct frame *, int, int, int, int) {
}
void
wr_draw_window_cursor (struct window *w, struct glyph_row *glyph_row, int x,
			 int y, enum text_cursor_kinds cursor_type,
		       int cursor_width, bool on_p, bool active_p) {
}
void
wr_draw_vertical_window_border (struct window *w, int x, int y0, int y1) {
}
void
wr_draw_window_divider (struct window *w, int x0, int x1, int y0, int y1) {
}
