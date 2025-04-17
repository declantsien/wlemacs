/* Definitions and headers for webrender GUI backend.
   Copyright (C) 1995, 2001-2017 Free Software Foundation, Inc.

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

#include "lisp.h"
#include "wr_ffi_generated.h"

#ifndef EMACS_WRGUI_H
#define EMACS_WRGUI_H

typedef void *Emacs_Pixmap;

/* Windows equivalent of XImage.  */
typedef struct _WRImage
{
  unsigned char *data;
  int info;
  /* Optional RGBQUAD array for palette follows (see BITMAPINFO docs).  */
} WRImage;

#define NativeRectangle WrRect
#define CONVERT_TO_EMACS_RECT(xr, nr)		\
  ((xr).x     = (nr).origin.x,			\
   (xr).y     = (nr).origin.y,			\
   (xr).width = (nr).size.width,		\
   (xr).height = (nr).size.height)

#define CONVERT_FROM_EMACS_RECT(xr, nr)		\
  ((nr).origin.x    = (xr).x,			\
   (nr).origin.y    = (xr).y,			\
   (nr).size.width  = (xr).width,		\
   (nr).size.height = (xr).height)

#define STORE_NATIVE_RECT(nr, px, py, pwidth, pheight)	\
  ((nr).origin.x    = (px),			\
   (nr).origin.y    = (py),			\
   (nr).size.width  = (pwidth),			\
   (nr).size.height = (pheight))

extern void wrgui_init(struct frame *f);
extern void syms_of_webrender(void);


extern void wrgui_init (struct frame *f);
/* extern void wr_prepare_font(struct frame *f, struct font *font); */

#endif /* EMACS_WRGUI_H */
