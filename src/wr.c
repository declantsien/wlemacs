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
