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
#include "font.h"
/* #include "wr_ffi.h" */
#ifdef HAVE_FREETYPE
#include "ftfont.h"
#endif /* HAVE_FREETYPE */
#ifdef HAVE_NS
/* #include "nsterm.h" */
#endif /* HAVE_NS */
#include TERM_HEADER

void
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

  DEFVAR_BOOL ("wr-worker-thread-local-arena", wr_worker_thread_local_arena,
	       doc: /* TODO add documents */);

  wr_worker_thread_local_arena = 1;
  DEFVAR_BOOL ("wr-precache-shaders", wr_precache_shaders,
	       doc: /* TODO add documents */);

  wr_precache_shaders = 1;

}
