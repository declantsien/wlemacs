/* module for window systems using WebRender.

Copyright (C) 1989, 1993-1994, 2005-2006, 2008-2025 Free Software
Foundation, Inc.

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

/* This should be the first include, as it may set up #defines affecting
   interpretation of even the system includes. */
#include <config.h>

#include "lisp.h"
#include "blockinput.h"
#include "frame.h"
#include TERM_HEADER

#define FRAME_WR_DATA(f) ((f)->output_data.wlc->wr_data)

void
wr_after_update_window_line (struct window *w,
			       struct glyph_row *desired_row)
{
  struct frame *f;
  int width, height;

  /* begin copy from other terms */
  eassert (w);

  if (!desired_row->mode_line_p && !w->pseudo_window_p)
    desired_row->redraw_fringe_bitmaps_p = 1;

  /* When a window has disappeared, make sure that no rest of
     full-width rows stays visible in the internal border.  */
  if (windows_or_buffers_changed
      && desired_row->full_width_p
      && (f = XFRAME (w->frame),
	  width = FRAME_INTERNAL_BORDER_WIDTH (f),
	  width != 0) && (height = desired_row->visible_height, height > 0))
    {
      int y = WINDOW_TO_FRAME_PIXEL_Y (w, max (0, desired_row->y));

      block_input ();
      wr_clear_frame_area (f, 0, y, width, height);
      wr_clear_frame_area (f,
			     FRAME_PIXEL_WIDTH (f) - width, y, width, height);
      unblock_input ();
    }
}

void
wr_clear_frame_area (struct frame *f, int x, int y, int width, int height)
{
  wr_clear_area (f, x, y, width, height);
}

void
wr_clear_frame (struct frame *f)
{
  if (!FRAME_DEFAULT_FACE (f))
    return;

  mark_window_cursors_off (XWINDOW (FRAME_ROOT_WINDOW (f)));

  block_input ();
  wr_clear_area (f, 0, 0, FRAME_PIXEL_WIDTH (f), FRAME_PIXEL_HEIGHT (f));
  unblock_input ();
}

/* Draw a vertical window border from (x,y0) to (x,y1)  */

void
wr_draw_vertical_window_border (struct window *w, int x, int y0, int y1)
{
  struct frame *f = XFRAME (WINDOW_FRAME (w));
  struct face *face;

  face = FACE_FROM_ID_OR_NULL (f, VERTICAL_BORDER_FACE_ID);

  wr_push_rect(FRAME_WR_DATA (f), face->foreground, x, y0, 1, y1 - y0);
}


/* Draw a window divider from (x0,y0) to (x1,y1)  */

void
wr_draw_window_divider (struct window *w, int x0, int x1, int y0, int y1)
{
  struct frame *f = XFRAME (WINDOW_FRAME (w));
  struct face *face = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_FACE_ID);
  struct face *face_first
    = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID);
  struct face *face_last
    = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_LAST_PIXEL_FACE_ID);
  unsigned long color = face ? face->foreground : FRAME_FOREGROUND_PIXEL (f);
  unsigned long color_first = (face_first
			       ? face_first->foreground
			       : FRAME_FOREGROUND_PIXEL (f));
  unsigned long color_last = (face_last
			      ? face_last->foreground
			      : FRAME_FOREGROUND_PIXEL (f));

  if (y1 - y0 > x1 - x0 && x1 - x0 > 2)
    /* Vertical.  */
    {
      wr_push_rect(FRAME_WR_DATA (f), color_first, x0, y0, 1, y1 - y0);
      wr_push_rect(FRAME_WR_DATA (f), color, x0 + 1, y0, x1 - x0 - 2, y1 - y0);
      wr_push_rect(FRAME_WR_DATA (f), color_last, x1 - 1, y0, 1, y1 - y0);
    }
  else if (x1 - x0 > y1 - y0 && y1 - y0 > 3)
    /* Horizontal.  */
    {
      wr_push_rect(FRAME_WR_DATA (f), color_first, x0, y0, x1 - x0, 1);
      wr_push_rect(FRAME_WR_DATA (f), color, x0, y0 + 1, x1 - x0, y1 - y0 - 2);
      wr_push_rect(FRAME_WR_DATA (f), color_last, x0, y1 - 1, x1 - x0, 1);
    }
  else
    {
      wr_push_rect(FRAME_WR_DATA (f), color, x0, y0, x1 - x0, y1 - y0);
    }
}

void
wr_flush_display (struct frame *f)
{
  wr_flush(FRAME_WR_DATA (f));
}

void
syms_of_webrender (void)
{
  DEFSYM (Qwr, "wr");
  DEFSYM (Qwebrender, "webrender");
  DEFSYM (Qwr_capture, "wr-capture");

  /* Tell Emacs about this renderer.  */
  Fprovide (Qwr, Qnil);
  Fprovide (Qwebrender, Qnil);

#ifdef GLYPH_DEBUG
  Fprovide (Qwr_capture, Qnil);
#endif
}
