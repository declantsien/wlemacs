/* ftwrfont.c -- FreeType font driver on cairo.
   Copyright (C) 2015-2025 Free Software Foundation, Inc.

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
#include <math.h>

#include "lisp.h"
#ifdef HAVE_X_WINDOWS
#include "xterm.h"
#elif HAVE_HAIKU
#include "haikuterm.h"
#include "haiku_support.h"
#include "termchar.h"
#else
#include "pgtkterm.h"
#endif
#include "blockinput.h"
#include "charset.h"
#include "composite.h"
#include "font.h"
#include "ftfont.h"
#include "pdumper.h"
#ifdef HAVE_PGTK
#include "xsettings.h"
#endif

#ifdef USE_BE_CAIRO
#define RED_FROM_ULONG(color)	(((color) >> 16) & 0xff)
#define GREEN_FROM_ULONG(color)	(((color) >> 8) & 0xff)
#define BLUE_FROM_ULONG(color)	((color) & 0xff)
#endif

#define METRICS_NCOLS_PER_ROW	(128)

enum metrics_status
  {
    METRICS_INVALID = -1,    /* metrics entry is invalid */
  };

#define METRICS_STATUS(metrics)	((metrics)->ascent + (metrics)->descent)
#define METRICS_SET_STATUS(metrics, status) \
  ((metrics)->ascent = 0, (metrics)->descent = (status))

static int
ftwrfont_glyph_extents (struct font *font,
                        unsigned glyph,
                        struct font_metrics *metrics)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;
  int row, col;
  struct font_metrics *cache;

  row = glyph / METRICS_NCOLS_PER_ROW;
  col = glyph % METRICS_NCOLS_PER_ROW;
  if (row >= ftwrfont_info->metrics_nrows)
    {
      ftwrfont_info->metrics =
	xrealloc (ftwrfont_info->metrics,
		  sizeof (struct font_metrics *) * (row + 1));
      memset (ftwrfont_info->metrics + ftwrfont_info->metrics_nrows, 0,
	      (sizeof (struct font_metrics *)
	       * (row + 1 - ftwrfont_info->metrics_nrows)));
      ftwrfont_info->metrics_nrows = row + 1;
    }
  if (ftwrfont_info->metrics[row] == NULL)
    {
      struct font_metrics *new;
      int i;

      new = xmalloc (sizeof (struct font_metrics) * METRICS_NCOLS_PER_ROW);
      for (i = 0; i < METRICS_NCOLS_PER_ROW; i++)
	METRICS_SET_STATUS (new + i, METRICS_INVALID);
      ftwrfont_info->metrics[row] = new;
    }
  cache = ftwrfont_info->metrics[row] + col;

  if (METRICS_STATUS (cache) == METRICS_INVALID)
    {
      /* TODO */
      /* cairo_glyph_t cr_glyph = {.index = glyph}; */
      /* cairo_text_extents_t extents; */

      /* cairo_scaled_font_glyph_extents (ftwrfont_info->cr_scaled_font, */
      /* 				       &cr_glyph, 1, &extents); */
      /* cache->lbearing = floor (extents.x_bearing); */
      /* cache->rbearing = ceil (extents.width + extents.x_bearing); */
      /* cache->width = lround (extents.x_advance); */
      /* /\* The subtraction of a small number is to avoid rounding up due */
      /* 	 to floating-point inaccuracies with some fonts, which then */
      /* 	 could cause unpleasant effects while scrolling (see bug */
      /* 	 #44284), since we then think that a glyph row's ascent is too */
      /* 	 small to accommodate a glyph with a higher phys_ascent.  *\/ */
      /* cache->ascent = ceil (- extents.y_bearing - 1.0 / 256); */
      /* cache->descent = ceil (extents.height + extents.y_bearing); */
    }

  if (metrics)
    *metrics = *cache;

  return cache->width;
}

static Lisp_Object
ftwrfont_list (struct frame *f, Lisp_Object spec)
{
  return ftfont_list2 (f, spec, Qftwr);
}

static Lisp_Object
ftwrfont_match (struct frame *f, Lisp_Object spec)
{
  return ftfont_match2 (f, spec, Qftwr);
}

static Lisp_Object
ftwrfont_open (struct frame *f, Lisp_Object entity, int pixel_size)
{
  FcResult result;
  Lisp_Object val, filename, idx, font_object;
  FcPattern *pat, *match;
  struct font_info *ftwrfont_info;
  struct font *font;
  double size = 0;
  /* cairo_font_face_t *font_face; */
  /* cairo_font_extents_t extents; */
  FT_Face ft_face;
  FcMatrix *matrix;

  val = assq_no_quit (QCfont_entity, AREF (entity, FONT_EXTRA_INDEX));
  if (! CONSP (val))
    return Qnil;
  val = XCDR (val);
  filename = XCAR (val);
  idx = XCDR (val);
  size = XFIXNUM (AREF (entity, FONT_SIZE_INDEX));
  if (size == 0)
    size = pixel_size;

  block_input ();

  pat = ftfont_entity_pattern (entity, pixel_size);
  FcConfigSubstitute (NULL, pat, FcMatchPattern);
  FcDefaultSubstitute (pat);
  match = FcFontMatch (NULL, pat, &result);
  ftfont_fix_match (pat, match);

  FcPatternDestroy (pat);

  font_object = font_build_object (VECSIZE (struct font_info),
				   AREF (entity, FONT_TYPE_INDEX),
				   entity, size);
  ASET (font_object, FONT_FILE_INDEX, filename);
  font = XFONT_OBJECT (font_object);
  font->pixel_size = size;
#ifdef HAVE_HARFBUZZ
  if (EQ (AREF (font_object, FONT_TYPE_INDEX), Qftwrhb))
    font->driver = &ftwrhbfont_driver;
  else
#endif	/* HAVE_HARFBUZZ */
  font->driver = &ftwrfont_driver;
  font->encoding_charset = font->repertory_charset = -1;
  font->default_ascent = 0;
  font->vertical_centering = false;    
  font->baseline_offset = 0;
  font->relative_compose = 0;

  ftwrfont_info = (struct font_info *) font;
  ftwrfont_info->index = XFIXNUM (idx);

  /* This means that there's no need of transformation.  */
  ftwrfont_info->matrix.xx = 0;
  if (FcPatternGetMatrix (match, FC_MATRIX, 0, &matrix) == FcResultMatch)
    {
      ftwrfont_info->matrix.xx = 0x10000L * matrix->xx;
      ftwrfont_info->matrix.yy = 0x10000L * matrix->yy;
      ftwrfont_info->matrix.xy = 0x10000L * matrix->xy;
      ftwrfont_info->matrix.yx = 0x10000L * matrix->yx;
    }

  ftwrfont_info->metrics = NULL;
  ftwrfont_info->metrics_nrows = 0;

  block_input ();
  font->min_width = font->max_width = 0;
  font->average_width = font->space_width = 0;
  wr_prepara_font(f, font);  


  /* cairo_scaled_font_extents (ftwrfont_info->cr_scaled_font, &extents); */
  /* font->ascent = lround (extents.ascent); */
  val = assq_no_quit (QCminspace, AREF (entity, FONT_EXTRA_INDEX));
  /* if (!(CONSP (val) && NILP (XCDR (val)))) */
  /*   { */
  /*     font->descent = lround (extents.descent); */
  /*     font->height = font->ascent + font->descent; */
  /*   } */
  /* else */
  /*   { */
  /*     font->height = lround (extents.height); */
  /*     font->descent = font->height - font->ascent; */
  /*   } */

  if (XFIXNUM (AREF (entity, FONT_SIZE_INDEX)) == 0)
    {
      /* int upEM = ft_face->units_per_EM; */

      /* font->underline_position = -ft_face->underline_position * size / upEM; */
      /* font->underline_thickness = ft_face->underline_thickness * size / upEM; */
      /* if (font->underline_thickness > 2) */
      /* 	font->underline_position -= font->underline_thickness / 2; */
    }
  else
    {
      font->underline_position = -1;
      font->underline_thickness = 0;
    }
#ifdef HAVE_LIBOTF
  /* ftwrfont_info->maybe_otf = (ft_face->face_flags & FT_FACE_FLAG_SFNT) != 0; */
  ftwrfont_info->otf = NULL;
#endif	/* HAVE_LIBOTF */
#ifdef HAVE_HARFBUZZ
  /* ftwrfont_info->hb_font = NULL; */
#endif	/* HAVE_HARFBUZZ */
  /* if (ft_face->units_per_EM) */
  /*   ftwrfont_info->bitmap_position_unit = 0; */
  /* else { */
  /*   /\* ftwrfont_info->bitmap_position_unit = (extents.height *\/ */
  /*   /\* 					   / ft_face->size->metrics.height); *\/ */
  /* } */
  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */
  unblock_input ();

  eassert (font->max_width < 512 * 1024 * 1024);
  if (NILP (font_object)) {
    fprintf(stderr, "font object nil ");
  } else {
    fprintf(stderr, "font object not nil ");
  }
  return font_object;
}

static void
ftwrfont_close (struct font *font)
{
  if (font_data_structures_may_be_ill_formed ())
    return;

  struct font_info *ftwrfont_info = (struct font_info *) font;

  block_input ();
#ifdef HAVE_LIBOTF
  if (ftwrfont_info->otf)
    {
      OTF_close (ftwrfont_info->otf);
      ftwrfont_info->otf = NULL;
    }
#endif
#ifdef HAVE_HARFBUZZ
  if (ftwrfont_info->hb_font)
    {
      hb_font_destroy (ftwrfont_info->hb_font);
      ftwrfont_info->hb_font = NULL;
    }
#endif
  if (ftwrfont_info->metrics)
    {
      for (int i = 0; i < ftwrfont_info->metrics_nrows; i++)
	if (ftwrfont_info->metrics[i])
	  xfree (ftwrfont_info->metrics[i]);
      if (ftwrfont_info->metrics)
	xfree (ftwrfont_info->metrics);
      ftwrfont_info->metrics = NULL;
    }
  /* if (ftwrfont_info->cr_scaled_font) */
  /*   { */
  /*     cairo_scaled_font_destroy (ftwrfont_info->cr_scaled_font); */
  /*     ftwrfont_info->cr_scaled_font = NULL; */
  /*   } */
  unblock_input ();
}

static int
ftwrfont_has_char (Lisp_Object font, int c)
{
  if (FONT_ENTITY_P (font))
    return ftfont_has_char (font, c);

  struct charset *cs = NULL;

  if (EQ (AREF (font, FONT_ADSTYLE_INDEX), Qja)
      && charset_jisx0208 >= 0)
    cs = CHARSET_FROM_ID (charset_jisx0208);
  else if (EQ (AREF (font, FONT_ADSTYLE_INDEX), Qko)
      && charset_ksc5601 >= 0)
    cs = CHARSET_FROM_ID (charset_ksc5601);
  if (cs)
    return (ENCODE_CHAR (cs, c) != CHARSET_INVALID_CODE (cs));

  return -1;
}

/* Return a glyph code of FONT for character C (Unicode code point).
   If FONT doesn't have such a glyph, return FONT_INVALID_CODE.  */
static unsigned
ftwrfont_encode_char (struct font *font, int c)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;
  unsigned code = FONT_INVALID_CODE;
  unsigned char utf8[MAX_MULTIBYTE_LENGTH];
  int utf8len = CHAR_STRING (c, utf8);
  /* cairo_glyph_t stack_glyph; */
  /* cairo_glyph_t *glyphs = &stack_glyph; */
  int num_glyphs = 1;

  /* if (cairo_scaled_font_text_to_glyphs (ftwrfont_info->cr_scaled_font, 0, 0, */
  /* 					(char *) utf8, utf8len, */
  /* 					&glyphs, &num_glyphs, */
  /* 					NULL, NULL, NULL) */
  /*     == CAIRO_STATUS_SUCCESS) */
  /*   { */
  /*     if (glyphs != &stack_glyph) */
  /* 	cairo_glyph_free (glyphs); */
  /*     else if (stack_glyph.index) */
  /* 	code = stack_glyph.index; */
  /*   } */

  return code;
}

static void
ftwrfont_text_extents (struct font *font,
                       const unsigned *code,
                       int nglyphs,
                       struct font_metrics *metrics)
{
  int width, i;

  block_input ();
  width = ftwrfont_glyph_extents (font, code[0], metrics);
  for (i = 1; i < nglyphs; i++)
    {
      struct font_metrics m;
      int w = ftwrfont_glyph_extents (font, code[i], metrics ? &m : NULL);

      if (metrics)
	{
	  if (width + m.lbearing < metrics->lbearing)
	    metrics->lbearing = width + m.lbearing;
	  if (width + m.rbearing > metrics->rbearing)
	    metrics->rbearing = width + m.rbearing;
	  if (m.ascent > metrics->ascent)
	    metrics->ascent = m.ascent;
	  if (m.descent > metrics->descent)
	    metrics->descent = m.descent;
	}
      width += w;
    }
  unblock_input ();

  if (metrics)
    metrics->width = width;
}

static int
ftwrfont_get_bitmap (struct font *font, unsigned int code,
		     struct font_bitmap *bitmap, int bits_per_pixel)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;

  if (ftwrfont_info->bitmap_position_unit)
    return -1;

  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */
  /* FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font); */

  /* ftwrfont_info->ft_size = ft_face->size; */
  /* int result = ftfont_get_bitmap (font, code, bitmap, bits_per_pixel); */
  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */

  /* return result; */
  return -1;
}

static int
ftwrfont_anchor_point (struct font *font, unsigned int code, int idx,
		       int *x, int *y)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;

  if (ftwrfont_info->bitmap_position_unit)
    return -1;

  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */
  /* FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font); */

  /* ftwrfont_info->ft_size = ft_face->size; */
  /* int result = ftfont_anchor_point (font, code, idx, x, y); */
  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */

  /* return result; */
  return -1;
}

#ifdef HAVE_LIBOTF
static Lisp_Object
ftwrfont_otf_capability (struct font *font)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;
  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */
  /* FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font); */

  /* ftwrfont_info->ft_size = ft_face->size; */
  /* Lisp_Object result = ftfont_otf_capability (font); */
  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */

  /* return result; */
  return Qnil;
}
#endif

#if defined HAVE_M17N_FLT && defined HAVE_LIBOTF
static Lisp_Object
ftwrfont_shape (Lisp_Object lgstring, Lisp_Object direction)
{
  struct font *font = CHECK_FONT_GET_OBJECT (LGSTRING_FONT (lgstring));
  struct font_info *ftwrfont_info = (struct font_info *) font;

  if (ftwrfont_info->bitmap_position_unit)
    return make_fixnum (0);

  cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font;
  FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font);

  ftwrfont_info->ft_size = ft_face->size;
  Lisp_Object result = ftfont_shape (lgstring, direction);
  cairo_ft_scaled_font_unlock_face (scaled_font);
  ftwrfont_info->ft_size = NULL;

  return result;
}
#endif

#if defined HAVE_OTF_GET_VARIATION_GLYPHS || defined HAVE_FT_FACE_GETCHARVARIANTINDEX
static int
ftwrfont_variation_glyphs (struct font *font, int c, unsigned variations[256])
{
  struct font_info *ftwrfont_info = (struct font_info *) font;
  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */
  /* FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font); */

  /* ftwrfont_info->ft_size = ft_face->size; */
  /* int result = ftfont_variation_glyphs (font, c, variations); */
  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */

  /* return result; */
  return -1;
}
#endif	/* HAVE_OTF_GET_VARIATION_GLYPHS || HAVE_FT_FACE_GETCHARVARIANTINDEX */

static int
ftwrfont_draw (struct glyph_string *s,
               int from, int to, int x, int y, bool with_background)
{}

#ifdef HAVE_PGTK
/* Determine if FONT_OBJECT is a valid cached font for ENTITY by
   comparing the options used to open it with the user's current
   preferences specified via GSettings.  */
static bool
ftwrfont_cached_font_ok (struct frame *f, Lisp_Object font_object,
			 Lisp_Object entity)
{
  struct font_info *info = (struct font_info *) XFONT_OBJECT (font_object);

  cairo_font_options_t *options = cairo_font_options_create ();
  cairo_scaled_font_get_font_options (info->cr_scaled_font, options);
  cairo_font_options_t *gsettings_options = xsettings_get_font_options ();

  bool equal = cairo_font_options_equal (options, gsettings_options);
  cairo_font_options_destroy (options);
  cairo_font_options_destroy (gsettings_options);

  return equal;
}
#endif

#ifdef HAVE_HARFBUZZ

static Lisp_Object
ftwrhbfont_list (struct frame *f, Lisp_Object spec)
{
  return ftfont_list2 (f, spec, Qftwrhb);
}

static Lisp_Object
ftwrhbfont_match (struct frame *f, Lisp_Object spec)
{
  return ftfont_match2 (f, spec, Qftwrhb);
}

static hb_font_t *
ftwrhbfont_begin_hb_font (struct font *font, double *position_unit)
{
  struct font_info *ftwrfont_info = (struct font_info *) font;
  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */
  /* FT_Face ft_face = cairo_ft_scaled_font_lock_face (scaled_font); */

  /* ftwrfont_info->ft_size = ft_face->size; */
  /* hb_font_t *hb_font = fthbfont_begin_hb_font (font, position_unit); */
  /* /\* HarfBuzz 5 correctly scales bitmap-only fonts without position */
  /*    unit adjustment. */
  /*    (https://github.com/harfbuzz/harfbuzz/issues/489) */

  /*    Update: HarfBuzz 5.2.0 no longer does this for an hb_font_t font */
  /*    object created from a given FT_Face. */
  /*    (https://github.com/harfbuzz/harfbuzz/issues/3788) *\/ */
  /* if ((hb_version_atleast (5, 2, 0) || !hb_version_atleast (5, 0, 0)) */
  /*     && ftwrfont_info->bitmap_position_unit) */
  /*   *position_unit = ftwrfont_info->bitmap_position_unit; */

  /* return hb_font; */
  return NULL;
}

static void
ftwrhbfont_end_hb_font (struct font *font, hb_font_t *hb_font)
{
  /* struct font_info *ftwrfont_info = (struct font_info *) font; */
  /* cairo_scaled_font_t *scaled_font = ftwrfont_info->cr_scaled_font; */

  /* eassert (hb_font == ftwrfont_info->hb_font); */
  /* /\* ftwrfont_info->hb_font holds a reference to the FT_Face returned by */
  /*    cairo_ft_scaled_font_lock_face.  Keeping it around after the matching */
  /*    unlock call would violate the API contract, and cause corrupted */
  /*    display of composed characters (Bug#73752).  We destroy and NULLify */
  /*    hb_font here, which will then cause fthbfont_begin_hb_font, called by */
  /*    ftwrhbfont_begin_hb_font, to recreate hb_font anew, taking into */
  /*    consideration any scale changes in FT_Face.  *\/ */
  /* hb_font_destroy (ftwrfont_info->hb_font); */
  /* ftwrfont_info->hb_font = NULL; */

  /* cairo_ft_scaled_font_unlock_face (scaled_font); */
  /* ftwrfont_info->ft_size = NULL; */
}

#endif	/* HAVE_HARFBUZZ */


static void syms_of_ftwrfont_for_pdumper (void);

struct font_driver const ftwrfont_driver =
  {
  .type = LISPSYM_INITIALLY (Qftwr),
  .get_cache = ftfont_get_cache,
  .list = ftwrfont_list,
  .match = ftwrfont_match,
  .list_family = ftfont_list_family,
  .open_font = ftwrfont_open,
  .close_font = ftwrfont_close,
  .has_char = ftwrfont_has_char,
  .encode_char = ftwrfont_encode_char,
  .text_extents = ftwrfont_text_extents,
  .draw = ftwrfont_draw,
  .get_bitmap = ftwrfont_get_bitmap,
  .anchor_point = ftwrfont_anchor_point,
#ifdef HAVE_LIBOTF
  .otf_capability = ftwrfont_otf_capability,
#endif
#if defined HAVE_M17N_FLT && defined HAVE_LIBOTF
  .shape = ftwrfont_shape,
#endif
#if defined HAVE_OTF_GET_VARIATION_GLYPHS || defined HAVE_FT_FACE_GETCHARVARIANTINDEX
  .get_variation_glyphs = ftwrfont_variation_glyphs,
#endif
  .filter_properties = ftfont_filter_properties,
  .combining_capability = ftfont_combining_capability,
#ifdef HAVE_PGTK
  .cached_font_ok = ftwrfont_cached_font_ok,
#endif
  };
#ifdef HAVE_HARFBUZZ
struct font_driver ftwrhbfont_driver;
#endif	/* HAVE_HARFBUZZ */

void
syms_of_ftwrfont (void)
{
  DEFSYM (Qftwr, "ftwr");
#ifdef HAVE_HARFBUZZ
  DEFSYM (Qftwrhb, "ftwrhb");
  Fput (Qftwr, Qfont_driver_superseded_by, Qftwrhb);
#endif	/* HAVE_HARFBUZZ */
  pdumper_do_now_and_after_load (syms_of_ftwrfont_for_pdumper);
}

#ifdef HAVE_X_WINDOWS

/* Place the default font options used by Cairo on the given display
   in OPTIONS.  */

void
ftwrfont_get_default_font_options (struct x_display_info *dpyinfo,
				   cairo_font_options_t *options)
{
  Pixmap drawable;
  cairo_surface_t *surface;

  /* Cairo doesn't allow fetching the default font options for a
     display, so the only option is to create a drawable, and an Xlib
     surface for that drawable, and to get the font options from there
     instead.  */

  drawable = XCreatePixmap (dpyinfo->display, dpyinfo->root_window,
			    1, 1, dpyinfo->n_planes);
  surface = cairo_xlib_surface_create (dpyinfo->display, drawable,
				       dpyinfo->visual, 1, 1);

  if (!surface)
    {
      XFreePixmap (dpyinfo->display, drawable);
      return;
    }

  cairo_surface_get_font_options (surface, options);
  XFreePixmap (dpyinfo->display, drawable);
  cairo_surface_destroy (surface);
  return;
}

#endif

static void
syms_of_ftwrfont_for_pdumper (void)
{
  register_font_driver (&ftwrfont_driver, NULL);
#ifdef HAVE_HARFBUZZ
  ftwrhbfont_driver = ftwrfont_driver;
  ftwrhbfont_driver.type = Qftwrhb;
  ftwrhbfont_driver.list = ftwrhbfont_list;
  ftwrhbfont_driver.match = ftwrhbfont_match;
  ftwrhbfont_driver.otf_capability = hbfont_otf_capability;
  ftwrhbfont_driver.shape = hbfont_shape;
  ftwrhbfont_driver.combining_capability = hbfont_combining_capability;
  ftwrhbfont_driver.begin_hb_font = ftwrhbfont_begin_hb_font;
  ftwrhbfont_driver.end_hb_font = ftwrhbfont_end_hb_font;
  register_font_driver (&ftwrhbfont_driver, NULL);
#endif	/* HAVE_HARFBUZZ */
}
