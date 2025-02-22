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
#include "wr_ffi_generated.h"
#ifdef HAVE_FREETYPE
#include "ftfont.h"
#endif /* HAVE_FREETYPE */
#ifdef HAVE_NS
/* #include "nsterm.h" */
#endif /* HAVE_NS */
#include TERM_HEADER

/* void */
/* wr_get_font_metrics_by_descriptor (const char *data, int index) { */
/*   fprintf(stderr, "font data: %s \n", data); */
/* } */

/* void */
/* wr_get_font_metrics (struct font *font, struct frame *f) */
/* { */
/*   wr_font_metrics font_metrics; */
/* #ifdef HAVE_NS */
/*   Lisp_Object fullname = font->props[FONT_NAME_INDEX]; */
/*   fprintf(stderr, SSDATA (fullname)); */
/*   /\* struct nsfont_info *font_info = (struct nsfont_info *) font; *\/ */
/*   /\* /\\* char *name = font_info->name; *\\/ *\/ */
/*   /\* const wr_vec_u8 data = { font_info->name, strlen (font_info->name), 0}; *\/ */
/*   /\* // On macOS, the descriptor string is a concatenation of the PostScript name *\/ */
/*   /\* // and the font file path (to disambiguate cases where there are multiple *\/ */
/*   /\* // faces with the same psname present). The index is the length of the psname *\/ */
/*   /\* // portion of the descriptor (= starting offset of the path). *\/ */
/*   /\* // Here, we split the descriptor into its two components for further use. *\/ */
/*   /\* const index = strlen (name); *\/ */
/*   /\* double scale_factor = ns_frame_scale_factor (f); *\/ */
/* #endif /\* HAVE_NS *\/ */

/*   /\* wr_font_metrics_imp(WR_FONT_TEMPLATE_NATIVE, *\/ */
/*   /\* 		      &data, *\/ */
/*   /\* 		      index, *\/ */
/*   /\* 		      font->pixel_size, *\/ */
/*   /\* 		      scale_factor, *\/ */
/*   /\* 		      &font_metrics); *\/ */
/* } */

/* void */
/* wr_after_update_window_line (struct window *w, */
/* 			       struct glyph_row *desired_row) */
/* { */
/*   struct frame *f; */
/*   int width, height; */

/*   /\* begin copy from other terms *\/ */
/*   eassert (w); */

/*   if (!desired_row->mode_line_p && !w->pseudo_window_p) */
/*     desired_row->redraw_fringe_bitmaps_p = 1; */

/*   /\* When a window has disappeared, make sure that no rest of */
/*      full-width rows stays visible in the internal border.  *\/ */
/*   if (windows_or_buffers_changed */
/*       && desired_row->full_width_p */
/*       && (f = XFRAME (w->frame), */
/* 	  width = FRAME_INTERNAL_BORDER_WIDTH (f), */
/* 	  width != 0) && (height = desired_row->visible_height, height > 0)) */
/*     { */
/*       int y = WINDOW_TO_FRAME_PIXEL_Y (w, max (0, desired_row->y)); */

/*       block_input (); */
/*       wr_clear_frame_area (f, 0, y, width, height); */
/*       wr_clear_frame_area (f, */
/* 			     FRAME_PIXEL_WIDTH (f) - width, y, width, height); */
/*       unblock_input (); */
/*     } */
/* } */

/* void */
/* wr_clear_frame_area (struct frame *f, int x, int y, int width, int height) */
/* { */
/*   wr_clear_area (FRAME_WR_DATA (f), f->background_pixel, x, y, width, height); */
/* } */

/* void */
/* wr_clear_frame (struct frame *f) */
/* { */
/*   if (!FRAME_DEFAULT_FACE (f)) */
/*     return; */

/*   if (!FRAME_WR_DATA (f)) */
/*     return; */

/*   mark_window_cursors_off (XWINDOW (FRAME_ROOT_WINDOW (f))); */

/*   block_input (); */
/*   wr_clear_area (FRAME_WR_DATA (f), f->background_pixel, 0, 0, FRAME_PIXEL_WIDTH (f), FRAME_PIXEL_HEIGHT (f)); */
/*   unblock_input (); */
/* } */

/* /\* Draw a vertical window border from (x,y0) to (x,y1)  *\/ */

/* void */
/* wr_draw_vertical_window_border (struct window *w, int x, int y0, int y1) */
/* { */
/*   struct frame *f = XFRAME (WINDOW_FRAME (w)); */
/*   struct face *face; */

/*   face = FACE_FROM_ID_OR_NULL (f, VERTICAL_BORDER_FACE_ID); */

/*   const Emacs_Rectangle bounds = {x, y0, 1, y1 - y0}; */
/*   wr_push_rect(FRAME_WR_DATA (f), face->foreground, &bounds, NULL); */
/* } */


/* /\* Draw a window divider from (x0,y0) to (x1,y1)  *\/ */

/* void */
/* wr_draw_window_divider (struct window *w, int x0, int x1, int y0, int y1) */
/* { */
/*   struct frame *f = XFRAME (WINDOW_FRAME (w)); */
/*   struct face *face = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_FACE_ID); */
/*   struct face *face_first */
/*     = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID); */
/*   struct face *face_last */
/*     = FACE_FROM_ID_OR_NULL (f, WINDOW_DIVIDER_LAST_PIXEL_FACE_ID); */
/*   unsigned long color = face ? face->foreground : FRAME_FOREGROUND_PIXEL (f); */
/*   unsigned long color_first = (face_first */
/* 			       ? face_first->foreground */
/* 			       : FRAME_FOREGROUND_PIXEL (f)); */
/*   unsigned long color_last = (face_last */
/* 			      ? face_last->foreground */
/* 			      : FRAME_FOREGROUND_PIXEL (f)); */

/*   static Emacs_Rectangle bounds; */

/*   if (y1 - y0 > x1 - x0 && x1 - x0 > 2) */
/*     /\* Vertical.  *\/ */
/*     { */
/*       bounds = (Emacs_Rectangle){x0, y0, 1, y1 - y0}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color_first, &bounds, NULL); */
/*       bounds = (Emacs_Rectangle){x0 + 1, y0, x1 - x0 - 2, y1 - y0}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color, &bounds, NULL); */
/*       bounds = (Emacs_Rectangle){x1 - 1, y0, 1, y1 - y0}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color_last, &bounds, NULL); */
/*     } */
/*   else if (x1 - x0 > y1 - y0 && y1 - y0 > 3) */
/*     /\* Horizontal.  *\/ */
/*     { */
/*       bounds = (Emacs_Rectangle){x0, y0, x1 - x0, 1}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color_first, &bounds, NULL); */
/*       bounds = (Emacs_Rectangle){x0, y0 + 1, x1 - x0, y1 - y0 - 2}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color, &bounds, NULL); */
/*       bounds = (Emacs_Rectangle){x0, y1 - 1, x1 - x0, 1}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color_last, &bounds, NULL); */
/*     } */
/*   else */
/*     { */
/*       bounds = (Emacs_Rectangle){x0, y0, x1 - x0, y1 - y0}; */
/*       wr_push_rect(FRAME_WR_DATA (f), color, &bounds, NULL); */
/*     } */
/* } */

/* void */
/* wr_flush_display (struct frame *f) */
/* { */
/*   wr_flush(FRAME_WR_DATA (f)); */
/* } */

/* void */
/* wr_free_pixmap (struct frame *f, Emacs_Pixmap pixmap) { */
/*   wr_free_pixmap_impl(FRAME_WR_DATA (f), pixmap); */
/* } */

/* extern void */
/* wr_free_frame_resources (struct frame *f) { */
/*   wr_destroy (FRAME_WR_DATA (f)); */
/*   FRAME_WR_DATA (f) = NULL; */
/* } */

/* void */
/* wr_clear_under_internal_border (struct frame *f) { */
/*   wr_clear_under_internal_border_impl(f, FRAME_WR_DATA (f)); */
/* } */

/* /\* Set clipping for output of glyph string S.  S may be part of a mode */
/*    line or menu if we don't have X toolkit support.  *\/ */

/* void */
/* wr_set_glyph_string_clipping (struct glyph_string *s) */
/* { */
/*   Emacs_Rectangle r[2]; */
/*   int n = get_glyph_string_clip_rects (s, r, 2); */

/*   if (n > 0) */
/*     { */
/*       for (int i = 0; i < n; i++) */
/* 	{ */
/* 	  wr_define_clip_rect (FRAME_WR_DATA (s->f), &r[i]); */
/* 	} */
/*     } */
/*   s->num_clips = n; */
/* } */


/* /\* Set SRC's clipping for output of glyph string DST.  This is called */
/*    when we are drawing DST's left_overhang or right_overhang only in */
/*    the area of SRC.  *\/ */

/* void */
/* wr_set_glyph_string_clipping_exactly (struct glyph_string *src, */
/* 					struct glyph_string *dst) */
/* { */
/*   dst->clip[0].x = src->x; */
/*   dst->clip[0].y = src->y; */
/*   dst->clip[0].width = src->width; */
/*   dst->clip[0].height = src->height; */
/*   dst->num_clips = 1; */

/*   Emacs_Rectangle clip = {src->x, src->y, src->width, src->height}; */
/*   wr_define_clip_rect (FRAME_WR_DATA (src->f), &clip); */
/* } */

/* #ifdef HAVE_FREETYPE */
/* int */
/* ftwrfont_draw (struct glyph_string *s, */
/*                int from, int to, int x, int y, bool with_background) */
/* { */
/*   struct frame *f = s->f; */
/*   char *filename = SSDATA(s->font->props[FONT_FILE_INDEX]); */
/*   struct font_info *font_info = (struct font_info *) s->font; */
/*   unsigned *glyphs; */
/*   int len = to - from; */
/*   int i; */
/*   const wr_vec_u8 filename_data = { filename, strlen (filename), 0}; */
/*   const wr_vec_u32 char2b  = { s->char2b, s->nchars, 0}; */

/*   /\* printf("%s", SSDATA (filename)); *\/ */

/*   block_input (); */

/*   if (with_background) */
/*     { */
/*       const Emacs_Rectangle rect = {x, y - FONT_BASE (s->font), */
/* 				    s->width, FONT_HEIGHT (s->font)}; */
/*       wr_dp_push_rect(FRAME_WR_DATA (f), &rect, &rect, s->hl != DRAW_CURSOR, false, false, s->gc->background); */
/*     } */

/*   wr_font_draw (FRAME_WR_DATA (s->f), */
/* 		s->gc->foreground, */
/* 		&filename_data, font_info->index, &char2b, from, to, */
/* 		x, y, s->width, (s->row->mode_line_p ? s->row->height : s->row->visible_height), */
/* 		s->font->pixel_size, s->padding_p); */



/*   /\* glyphs = alloca (sizeof (unsigned) * len); *\/ */
/*   /\* for (i = 0; i < len; i++) *\/ */
/*   /\*   { *\/ */
/*   /\*     glyphs[i].index = s->char2b[from + i]; *\/ */
/*   /\*     glyphs[i].x = x; *\/ */
/*   /\*     glyphs[i].y = y; *\/ */
/*   /\*     x += (s->padding_p ? 1 : ftcrfont_glyph_extents (s->font, *\/ */
/*   /\*                                                      glyphs[i].index, *\/ */
/*   /\*                                                      NULL)); *\/ */
/*   /\*   } *\/ */
/*   /\* pgtk_set_cr_source_with_color (f, s->xgcv.foreground, false); *\/ */
/*   /\* cairo_set_scaled_font (cr, ftcrfont_info->cr_scaled_font); *\/ */
/*   /\* cairo_show_glyphs (cr, glyphs, len); *\/ */

/*   unblock_input (); */

/*   wr_vec_u8_free(filename_data); */
/*   wr_vec_u32_free(char2b); */

/*   return len; */
/* } */
/* #endif */

/* /\* int *\/ */
/* /\* wr_font_draw (struct glyph_string *s, *\/ */
/* /\*                int from, int to, int x, int y, bool with_background) *\/ */
/* /\* { *\/ */
/* /\*   struct frame *f = s->f; *\/ */
/* /\*   Lisp_Object filename = s->font->props[FONT_FILE_INDEX]; *\/ */
/* /\*   struct font_info *font_info = (struct font_info *) s->font; *\/ */
/* /\*   cairo_t *cr; *\/ */
/* /\*   cairo_glyph_t *glyphs; *\/ */
/* /\*   int len = to - from; *\/ */
/* /\*   int i; *\/ */

/* /\*   printf("%s", SSDATA (filename)); *\/ */

/* /\*   /\\* /\\\* As ENCODE_UTF_8 may cause GC and relocation of string data, *\\/ *\/ */
/* /\*   /\\*    we use it before x_encode_text that may return string data.  *\\\/ *\\/ *\/ */
/* /\*   /\\* encoded_name = ENCODE_UTF_8 (name); *\\/ *\/ */

/* /\*   gtk_window_set_title (GTK_WINDOW (FRAME_GTK_OUTER_WIDGET (f)), *\/ */
/* /\* 			SSDATA (encoded_name)); *\/ */

/* /\*   block_input (); *\/ */

/* /\*   if (with_background) *\/ */
/* /\*     { *\/ */
/* /\*       const Emacs_Rectangle rect = {x, y - FONT_BASE (s->font), *\/ */
/* /\* 				    s->width, FONT_HEIGHT (s->font)}; *\/ */
/* /\*       wr_dp_push_rect(FRAME_WR_DATA (f), &rect, &rect, false, false, false, color); *\/ */
/* /\*     } *\/ */

/* /\*   /\\* glyphs = alloca (sizeof (cairo_glyph_t) * len); *\\/ *\/ */
/* /\*   /\\* for (i = 0; i < len; i++) *\\/ *\/ */
/* /\*   /\\*   { *\\/ *\/ */
/* /\*   /\\*     glyphs[i].index = s->char2b[from + i]; *\\/ *\/ */
/* /\*   /\\*     glyphs[i].x = x; *\\/ *\/ */
/* /\*   /\\*     glyphs[i].y = y; *\\/ *\/ */
/* /\*   /\\*     x += (s->padding_p ? 1 : ftcrfont_glyph_extents (s->font, *\\/ *\/ */
/* /\*   /\\*                                                      glyphs[i].index, *\\/ *\/ */
/* /\*   /\\*                                                      NULL)); *\\/ *\/ */
/* /\*   /\\*   } *\\/ *\/ */
/* /\*   /\\* pgtk_set_cr_source_with_color (f, s->xgcv.foreground, false); *\\/ *\/ */
/* /\*   /\\* cairo_set_scaled_font (cr, ftcrfont_info->cr_scaled_font); *\\/ *\/ */
/* /\*   /\\* cairo_show_glyphs (cr, glyphs, len); *\\/ *\/ */
/* /\*   /\\* unblock_input (); *\\/ *\/ */

/* /\*   return len; *\/ */
/* /\* } *\/ */

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
