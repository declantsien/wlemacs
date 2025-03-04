#include <config.h>

#include "dispextern.h"
#include "blockinput.h"
#include "nsterm.h"
#include "termchar.h"
#include "composite.h"
#include "buffer.h"
#include "window.h"
#include "xwidget.h"

/* display update */
static BOOL gsaved = NO;

static NSRect
ns_row_rect (struct window *w, struct glyph_row *row,
               enum glyph_row_area area)
/* Get the row as an NSRect.  */
{
  NSRect rect;
  int window_x, window_y, window_width;

  window_box (w, area, &window_x, &window_y, &window_width, 0);

  rect.origin.x = window_x;
  rect.origin.y = WINDOW_TO_FRAME_PIXEL_Y (w, max (0, row->y));
  rect.origin.y = max (rect.origin.y, window_y);
  rect.size.width = window_width;
  rect.size.height = row->visible_height;

  return rect;
}

static void
ns_focus (struct frame *f, NSRect *r, int n)
/* --------------------------------------------------------------------------
   Internal: Focus on given frame.  During small local updates this is used to
     draw, however during large updates, ns_update_begin and ns_update_end are
     called to wrap the whole thing, in which case these calls are stubbed out.
     Except, on GNUstep, we accumulate the rectangle being drawn into, because
     the back end won't do this automatically, and will just end up flushing
     the entire window.
   -------------------------------------------------------------------------- */
{
  NSTRACE_WHEN (NSTRACE_GROUP_FOCUS, "ns_focus");
  if (r != NULL)
    {
      NSTRACE_RECT ("r", *r);
    }

  if (f != ns_updating_frame)
    {
      EmacsView *view = FRAME_NS_VIEW (f);
      [view lockFocus];
    }

  /* clipping */
  if (r)
    {
      NSGraphicsContext *ctx = [NSGraphicsContext currentContext];
      [ctx saveGraphicsState];
#ifdef NS_IMPL_COCOA
      if (n == 2)
        NSRectClipList (r, 2);
      else
        NSRectClip (*r);
#else
      GSRectClipList (ctx, r, n);
#endif
      gsaved = YES;
    }
}


static void
ns_unfocus (struct frame *f)
/* --------------------------------------------------------------------------
     Internal: Remove focus on given frame
   -------------------------------------------------------------------------- */
{
  NSTRACE_WHEN (NSTRACE_GROUP_FOCUS, "ns_unfocus");

  if (gsaved)
    {
      [[NSGraphicsContext currentContext] restoreGraphicsState];
      gsaved = NO;
    }

  if (f != ns_updating_frame)
    {
      EmacsView *view = FRAME_NS_VIEW (f);
      [view unlockFocus];
#if defined (NS_IMPL_GNUSTEP) || MAC_OS_X_VERSION_MIN_REQUIRED < 101400
      [[view window] flushWindow];
#endif
    }
}

/* ==========================================================================

    Block drawing operations

   ========================================================================== */


#ifdef NS_IMPL_GNUSTEP
static void
ns_redraw_scroll_bars (struct frame *f)
{
  int i;
  id view;
  NSArray *subviews = [[FRAME_NS_VIEW (f) superview] subviews];
  NSTRACE ("ns_redraw_scroll_bars");
  for (i =[subviews count]-1; i >= 0; i--)
    {
      view = [subviews objectAtIndex: i];
      if (![view isKindOfClass: [EmacsScroller class]]) continue;
      [view display];
    }
}
#endif


void
ns_clear_frame (struct frame *f)
/* --------------------------------------------------------------------------
      External (hook): Erase the entire frame
   -------------------------------------------------------------------------- */
{
  NSView *view = FRAME_NS_VIEW (f);
  NSRect r;

  NSTRACE_WHEN (NSTRACE_GROUP_UPDATES, "ns_clear_frame");

 /* comes on initial frame because we have
    after-make-frame-functions = select-frame */
 if (!FRAME_DEFAULT_FACE (f))
   return;

  mark_window_cursors_off (XWINDOW (FRAME_ROOT_WINDOW (f)));

  r = [view bounds];

  block_input ();
  ns_focus (f, &r, 1);
  [[NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND
			    (FACE_FROM_ID (f, DEFAULT_FACE_ID))] set];
  NSRectFill (r);
  ns_unfocus (f);

#ifdef NS_IMPL_GNUSTEP
  ns_redraw_scroll_bars (f);
#endif
  unblock_input ();
}


void
ns_clear_frame_area (struct frame *f, int x, int y, int width, int height)
/* --------------------------------------------------------------------------
    External (RIF):  Clear section of frame
   -------------------------------------------------------------------------- */
{
  NSRect r = NSMakeRect (x, y, width, height);
  NSView *view = FRAME_NS_VIEW (f);
  struct face *face = FRAME_DEFAULT_FACE (f);

  if (!view || !face)
    return;

  NSTRACE_WHEN (NSTRACE_GROUP_UPDATES, "ns_clear_frame_area");

  r = NSIntersectionRect (r, [view frame]);
  ns_focus (f, &r, 1);
  [[NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND (face)] set];

  NSRectFill (r);

  ns_unfocus (f);
  return;
}


static void
ns_scroll_run (struct window *w, struct run *run)
/* --------------------------------------------------------------------------
    External (RIF):  Insert or delete n lines at line vpos.
   -------------------------------------------------------------------------- */
{
  struct frame *f = XFRAME (w->frame);
  int x, y, width, height, from_y, to_y, bottom_y;

  NSTRACE ("ns_scroll_run");

  /* begin copy from other terms */
  /* Get frame-relative bounding box of the text display area of W,
     without mode lines.  Include in this box the left and right
     fringe of W.  */
  window_box (w, ANY_AREA, &x, &y, &width, &height);

  from_y = WINDOW_TO_FRAME_PIXEL_Y (w, run->current_y);
  to_y = WINDOW_TO_FRAME_PIXEL_Y (w, run->desired_y);
  bottom_y = y + height;

  if (to_y < from_y)
    {
      /* Scrolling up.  Make sure we don't copy part of the mode
	 line at the bottom.  */
      if (from_y + run->height > bottom_y)
	height = bottom_y - from_y;
      else
	height = run->height;
    }
  else
    {
      /* Scrolling down.  Make sure we don't copy over the mode line.
	 at the bottom.  */
      if (to_y + run->height > bottom_y)
	height = bottom_y - to_y;
      else
	height = run->height;
    }
  /* end copy from other terms */

  if (height == 0)
      return;

  block_input ();

  gui_clear_cursor (w);

  {
    NSRect srcRect = NSMakeRect (x, from_y, width, height);
    NSPoint dest = NSMakePoint (x, to_y);
    EmacsView *view = FRAME_NS_VIEW (f);

    [view copyRect:srcRect to:dest];
#if defined (NS_IMPL_COCOA) && MAC_OS_X_VERSION_MAX_ALLOWED < 101400
    [view setNeedsDisplayInRect:srcRect];
#endif
  }

  unblock_input ();
}


static void
ns_clear_under_internal_border (struct frame *f)
{
  NSTRACE ("ns_clear_under_internal_border");

  if (FRAME_LIVE_P (f) && FRAME_INTERNAL_BORDER_WIDTH (f) > 0)
    {
      int border = FRAME_INTERNAL_BORDER_WIDTH (f);
      int width = FRAME_PIXEL_WIDTH (f);
      int height = FRAME_PIXEL_HEIGHT (f);
      int margin = FRAME_TOP_MARGIN_HEIGHT (f);
      int bottom_margin = FRAME_BOTTOM_MARGIN_HEIGHT (f);
      int face_id =
        (FRAME_PARENT_FRAME (f)
         ? (!NILP (Vface_remapping_alist)
            ? lookup_basic_face (NULL, f, CHILD_FRAME_BORDER_FACE_ID)
            : CHILD_FRAME_BORDER_FACE_ID)
         : (!NILP (Vface_remapping_alist)
            ? lookup_basic_face (NULL, f, INTERNAL_BORDER_FACE_ID)
            : INTERNAL_BORDER_FACE_ID));
      struct face *face = FACE_FROM_ID_OR_NULL (f, face_id);

      if (!face)
        face = FRAME_DEFAULT_FACE (f);

      /* Sometimes with new frames we reach this point and have no
         face.  I'm not sure why we have a live frame but no face, so
         just give up.  */
      if (!face)
        return;

      ns_focus (f, NULL, 1);
      [[NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND (face)] set];
      NSRectFill (NSMakeRect (0, margin, width, border));
      NSRectFill (NSMakeRect (0, 0, border, height));
      NSRectFill (NSMakeRect (0, margin, width, border));
      NSRectFill (NSMakeRect (width - border, 0, border, height));
      NSRectFill (NSMakeRect (0, height - bottom_margin - border,
			      width, border));
      ns_unfocus (f);
    }
}


static void
ns_after_update_window_line (struct window *w, struct glyph_row *desired_row)
/* --------------------------------------------------------------------------
    External (RIF): preparatory to fringe update after text was updated
   -------------------------------------------------------------------------- */
{
  struct frame *f;
  int width, height;

  NSTRACE_WHEN (NSTRACE_GROUP_UPDATES, "ns_after_update_window_line");

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
	  width != 0)
      && (height = desired_row->visible_height,
	  height > 0))
    {
      int y = WINDOW_TO_FRAME_PIXEL_Y (w, max (0, desired_row->y));
      int face_id =
        !NILP (Vface_remapping_alist)
        ? lookup_basic_face (NULL, f, INTERNAL_BORDER_FACE_ID)
        : INTERNAL_BORDER_FACE_ID;
      struct face *face = FACE_FROM_ID_OR_NULL (f, face_id);

      block_input ();
      if (face)
        {
          NSRect r = NSMakeRect (0, y, FRAME_PIXEL_WIDTH (f), height);
          ns_focus (f, &r, 1);

          [[NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND (face)] set];
          NSRectFill (NSMakeRect (0, y, width, height));
          NSRectFill (NSMakeRect (FRAME_PIXEL_WIDTH (f) - width,
                                  y, width, height));

          ns_unfocus (f);
        }
      else
        {
          ns_clear_frame_area (f, 0, y, width, height);
          ns_clear_frame_area (f,
                               FRAME_PIXEL_WIDTH (f) - width,
                               y, width, height);
        }
      unblock_input ();
    }
}


static void
ns_shift_glyphs_for_insert (struct frame *f,
                           int x, int y, int width, int height,
                           int shift_by)
/* --------------------------------------------------------------------------
    External (RIF): copy an area horizontally, don't worry about clearing src
   -------------------------------------------------------------------------- */
{
  NSRect srcRect = NSMakeRect (x, y, width, height);
  NSPoint dest = NSMakePoint (x+shift_by, y);

  NSTRACE ("ns_shift_glyphs_for_insert");

  [FRAME_NS_VIEW (f) copyRect:srcRect to:dest];
}



/* ==========================================================================

    Character encoding and metrics

   ========================================================================== */


static void
ns_compute_glyph_string_overhangs (struct glyph_string *s)
/* --------------------------------------------------------------------------
     External (RIF); compute left/right overhang of whole string and set in s
   -------------------------------------------------------------------------- */
{
  if (s->cmp == NULL
      && (s->first_glyph->type == CHAR_GLYPH
	  || s->first_glyph->type == COMPOSITE_GLYPH))
    {
      struct font_metrics metrics;

      if (s->first_glyph->type == CHAR_GLYPH)
	{
	  struct font *font = s->font;
	  font->driver->text_extents (font, s->char2b, s->nchars, &metrics);
	}
      else
	{
	  Lisp_Object gstring = composition_gstring_from_id (s->cmp_id);

	  composition_gstring_width (gstring, s->cmp_from, s->cmp_to, &metrics);
	}
      s->right_overhang = (metrics.rbearing > metrics.width
			   ? metrics.rbearing - metrics.width : 0);
      s->left_overhang = metrics.lbearing < 0 ? - metrics.lbearing : 0;
    }
  else if (s->cmp)
    {
      s->right_overhang = s->cmp->rbearing - s->cmp->pixel_width;
      s->left_overhang = - s->cmp->lbearing;
    }
}



/* ==========================================================================

    Fringe and cursor drawing

   ========================================================================== */

static NSMutableDictionary *fringe_bmp;

static void
ns_define_fringe_bitmap (int which, unsigned short *bits, int h, int w)
{
  NSBezierPath *p = [NSBezierPath bezierPath];

  if (!fringe_bmp)
    fringe_bmp = [[NSMutableDictionary alloc] initWithCapacity:25];

  [p moveToPoint:NSMakePoint (0, 0)];

  for (int y = 0 ; y < h ; y++)
    for (int x = 0 ; x < w ; x++)
      {
        bool bit = bits[y] & (1 << (w - x - 1));
        if (bit)
          [p appendBezierPathWithRect:NSMakeRect (x, y, 1, 1)];
      }

  [fringe_bmp setObject:p forKey:[NSNumber numberWithInt:which]];
}


static void
ns_destroy_fringe_bitmap (int which)
{
  [fringe_bmp removeObjectForKey:[NSNumber numberWithInt:which]];
}


static void
ns_draw_fringe_bitmap (struct window *w, struct glyph_row *row,
                      struct draw_fringe_bitmap_params *p)
/* --------------------------------------------------------------------------
    External (RIF); fringe-related
   -------------------------------------------------------------------------- */
{
  /* Fringe bitmaps comes in two variants, normal and periodic.  A
     periodic bitmap is used to create a continuous pattern.  Since a
     bitmap is rendered one text line at a time, the start offset (dh)
     of the bitmap varies.  Concretely, this is used for the empty
     line indicator.

     For a bitmap, "h + dh" is the full height and is always
     invariant.  For a normal bitmap "dh" is zero.

     For example, when the period is three and the full height is 72
     the following combinations exists:

       h=72 dh=0
       h=71 dh=1
       h=70 dh=2 */

  struct frame *f = XFRAME (WINDOW_FRAME (w));
  struct face *face = p->face;
  NSRect clearRect = NSZeroRect;
  NSRect rowRect = ns_row_rect (w, row, ANY_AREA);

  NSTRACE_WHEN (NSTRACE_GROUP_FRINGE, "ns_draw_fringe_bitmap");
  NSTRACE_MSG ("which:%d cursor:%d overlay:%d width:%d height:%d period:%d",
               p->which, p->cursor_p, p->overlay_p, p->wd, p->h, p->dh);

  /* Clear screen unless overlay.  */
  if (!p->overlay_p)
    {
      /* Work out the rectangle we will need to clear.  */
      clearRect = NSMakeRect (p->x, p->y, p->wd, p->h);

      if (p->bx >= 0)
        clearRect = NSUnionRect (clearRect, NSMakeRect (p->bx, p->by, p->nx, p->ny));

      /* Handle partially visible rows.  */
      clearRect = NSIntersectionRect (clearRect, rowRect);

      /* The visible portion of imageRect will always be contained
	 within clearRect.  */
      ns_focus (f, &clearRect, 1);
      if (!NSIsEmptyRect (clearRect))
        {
          NSTRACE_RECT ("clearRect", clearRect);

          [[NSColor colorWithUnsignedLong:face->background] set];
          NSRectFill (clearRect);
        }
    }

  NSBezierPath *bmp = [fringe_bmp objectForKey:[NSNumber numberWithInt:p->which]];

  if (bmp == nil
      && p->which < max_used_fringe_bitmap)
    {
      gui_define_fringe_bitmap (f, p->which);
      bmp = [fringe_bmp objectForKey: [NSNumber numberWithInt: p->which]];
    }

  if (bmp)
    {
      NSAffineTransform *transform = [NSAffineTransform transform];
      NSColor *bm_color;

      /* Because the image is defined at (0, 0) we need to take a copy
         and then transform that copy to the new origin.  */
      bmp = [bmp copy];
      [transform translateXBy:p->x yBy:p->y - p->dh];
      [bmp transformUsingAffineTransform:transform];

      if (!p->cursor_p)
        bm_color = [NSColor colorWithUnsignedLong:face->foreground];
      else if (p->overlay_p)
        bm_color = [NSColor colorWithUnsignedLong:face->background];
      else
        bm_color = f->output_data.ns->cursor_color;

      [bm_color set];
      [bmp fill];

      [bmp release];
    }
  ns_unfocus (f);
}


static void
ns_draw_window_cursor (struct window *w, struct glyph_row *glyph_row,
		       int x, int y, enum text_cursor_kinds cursor_type,
		       int cursor_width, bool on_p, bool active_p)
/* --------------------------------------------------------------------------
     External call (RIF): draw cursor.
     Note that CURSOR_WIDTH is meaningful only for (h)bar cursors.
   -------------------------------------------------------------------------- */
{
  NSRect r;
  int fx, fy, h, cursor_height;
  struct frame *f = WINDOW_XFRAME (w);
  struct glyph *phys_cursor_glyph;
  struct glyph *cursor_glyph;

  /* If cursor is out of bounds, don't draw garbage.  This can happen
     in mini-buffer windows when switching between echo area glyphs
     and mini-buffer.  */

  NSTRACE ("ns_draw_window_cursor (on = %d, cursor_type = %d)",
	   on_p, cursor_type);

  if (!on_p)
    return;

  w->phys_cursor_type = cursor_type;
  w->phys_cursor_on_p = on_p;

  if (cursor_type == NO_CURSOR)
    {
      w->phys_cursor_width = 0;
      return;
    }

  if ((phys_cursor_glyph = get_phys_cursor_glyph (w)) == NULL)
    {
      NSTRACE_MSG ("No phys cursor glyph was found!");

      if (glyph_row->exact_window_width_line_p
          && w->phys_cursor.hpos >= glyph_row->used[TEXT_AREA])
        {
          glyph_row->cursor_in_fringe_p = 1;
          draw_fringe_bitmap (w, glyph_row, 0);
        }
      return;
    }

  get_phys_cursor_geometry (w, glyph_row, phys_cursor_glyph, &fx, &fy, &h);

  /* The above get_phys_cursor_geometry call set w->phys_cursor_width
     to the glyph width; replace with CURSOR_WIDTH for (V)BAR cursors.  */
  if (cursor_type == BAR_CURSOR)
    {
      if (cursor_width < 1)
	cursor_width = max (FRAME_CURSOR_WIDTH (f), 1);

      /* The bar cursor should never be wider than the glyph.  */
      if (cursor_width < w->phys_cursor_width)
        w->phys_cursor_width = cursor_width;

      /* If the character under cursor is R2L, draw the bar cursor
         on the right of its glyph, rather than on the left.  */
      cursor_glyph = get_phys_cursor_glyph (w);
      if ((cursor_glyph->resolved_level & 1) != 0)
        fx += cursor_glyph->pixel_width - w->phys_cursor_width;
    }
  /* If we have an HBAR, "cursor_width" MAY specify height.  */
  else if (cursor_type == HBAR_CURSOR)
    {
      cursor_height = (cursor_width < 1) ? lrint (0.25 * h) : cursor_width;
      if (cursor_height > glyph_row->height)
        cursor_height = glyph_row->height;
      if (h > cursor_height) // Cursor smaller than line height, move down
        fy += h - cursor_height;
      h = cursor_height;
    }

  r.origin.x = fx, r.origin.y = fy;
  r.size.height = h;
  r.size.width = w->phys_cursor_width;

  /* Prevent the cursor from being drawn outside the text area.  */
  r = NSIntersectionRect (r, ns_row_rect (w, glyph_row, TEXT_AREA));

  ns_focus (f, NULL, 0);

  NSGraphicsContext *ctx = [NSGraphicsContext currentContext];
  [ctx saveGraphicsState];
#ifdef NS_IMPL_GNUSTEP
  GSRectClipList (ctx, &r, 1);
#else
  NSRectClip (r);
#endif

  [FRAME_CURSOR_COLOR (f) set];

  switch (cursor_type)
    {
    case DEFAULT_CURSOR:
    case NO_CURSOR:
      break;
    case FILLED_BOX_CURSOR:
      /* The call to draw_phys_cursor_glyph can end up undoing the
	 ns_focus, so unfocus here and regain focus later.  */
      [ctx restoreGraphicsState];
      ns_unfocus (f);
      draw_phys_cursor_glyph (w, glyph_row, DRAW_CURSOR);
      ns_focus (f, &r, 1);
      break;
    case HOLLOW_BOX_CURSOR:
      /* This works like it does in PostScript, not X Windows.  */
      [NSBezierPath strokeRect: NSInsetRect (r, 0.5, 0.5)];
      [ctx restoreGraphicsState];
      break;
    case HBAR_CURSOR:
    case BAR_CURSOR:
      NSRectFill (r);
      [ctx restoreGraphicsState];
      break;
    }

  ns_unfocus (f);
}


static void
ns_draw_vertical_window_border (struct window *w, int x, int y0, int y1)
/* --------------------------------------------------------------------------
     External (RIF): Draw a vertical line.
   -------------------------------------------------------------------------- */
{
  struct frame *f = XFRAME (WINDOW_FRAME (w));
  struct face *face;
  NSRect r = NSMakeRect (x, y0, 1, y1-y0);

  NSTRACE ("ns_draw_vertical_window_border");

  face = FACE_FROM_ID_OR_NULL (f, VERTICAL_BORDER_FACE_ID);

  ns_focus (f, &r, 1);
  if (face)
    [[NSColor colorWithUnsignedLong:face->foreground] set];

  NSRectFill(r);
  ns_unfocus (f);
}


static void
ns_draw_window_divider (struct window *w, int x0, int x1, int y0, int y1)
/* --------------------------------------------------------------------------
     External (RIF): Draw a window divider.
   -------------------------------------------------------------------------- */
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
  NSRect divider = NSMakeRect (x0, y0, x1-x0, y1-y0);

  NSTRACE ("ns_draw_window_divider");

  ns_focus (f, &divider, 1);

  if ((y1 - y0 > x1 - x0) && (x1 - x0 >= 3))
    /* A vertical divider, at least three pixels wide: Draw first and
       last pixels differently.  */
    {
      [[NSColor colorWithUnsignedLong:color_first] set];
      NSRectFill(NSMakeRect (x0, y0, 1, y1 - y0));
      [[NSColor colorWithUnsignedLong:color] set];
      NSRectFill(NSMakeRect (x0 + 1, y0, x1 - x0 - 2, y1 - y0));
      [[NSColor colorWithUnsignedLong:color_last] set];
      NSRectFill(NSMakeRect (x1 - 1, y0, 1, y1 - y0));
    }
  else if ((x1 - x0 > y1 - y0) && (y1 - y0 >= 3))
    /* A horizontal divider, at least three pixels high: Draw first and
       last pixels differently.  */
    {
      [[NSColor colorWithUnsignedLong:color_first] set];
      NSRectFill(NSMakeRect (x0, y0, x1 - x0, 1));
      [[NSColor colorWithUnsignedLong:color] set];
      NSRectFill(NSMakeRect (x0, y0 + 1, x1 - x0, y1 - y0 - 2));
      [[NSColor colorWithUnsignedLong:color_last] set];
      NSRectFill(NSMakeRect (x0, y1 - 1, x1 - x0, 1));
    }
  else
    {
      /* In any other case do not draw the first and last pixels
         differently.  */
      [[NSColor colorWithUnsignedLong:color] set];
      NSRectFill(divider);
    }

  ns_unfocus (f);
}


static void
ns_show_hourglass (struct frame *f)
{
  /* TODO: add NSProgressIndicator to all frames.  */
}

static void
ns_hide_hourglass (struct frame *f)
{
  /* TODO: remove NSProgressIndicator from all frames.  */
}

/* ==========================================================================

    Glyph drawing operations

   ========================================================================== */

static int
ns_get_glyph_string_clip_rect (struct glyph_string *s, NativeRectangle *nr)
/* --------------------------------------------------------------------------
    Wrapper utility to account for internal border width on full-width lines,
    and allow top full-width rows to hit the frame top.  nr should be pointer
    to two successive NSRects.  Number of rects actually used is returned.
   -------------------------------------------------------------------------- */
{
  int n = get_glyph_string_clip_rects (s, nr, 2);
  return n;
}

/* --------------------------------------------------------------------
   Draw a wavy line under glyph string s. The wave fills wave_height
   pixels from y.

                    x          wave_length = 2
                                 --
                y    *   *   *   *   *
                     |* * * * * * * * *
    wave_height = 3  | *   *   *   *
  --------------------------------------------------------------------- */

static void
ns_draw_underwave (struct glyph_string *s, EmacsCGFloat width, EmacsCGFloat x)
{
  int wave_height = 3, wave_length = 2;
  int y, dx, dy, odd, xmax;
  NSPoint a, b;
  NSRect waveClip;

  dx = wave_length;
  dy = wave_height - 1;
  y =  s->ybase - wave_height + 3;
  xmax = x + width;

  /* Find and set clipping rectangle */
  waveClip = NSMakeRect (x, y, width, wave_height);
  [[NSGraphicsContext currentContext] saveGraphicsState];
  NSRectClip (waveClip);

  /* Draw the waves */
  a.x = x - ((int)(x) % dx) + (EmacsCGFloat) 0.5;
  b.x = a.x + dx;
  odd = (int)(a.x/dx) % 2;
  a.y = b.y = y + 0.5;

  if (odd)
    a.y += dy;
  else
    b.y += dy;

  while (a.x <= xmax)
    {
      [NSBezierPath strokeLineFromPoint:a toPoint:b];
      a.x = b.x, a.y = b.y;
      b.x += dx, b.y = y + 0.5 + odd*dy;
      odd = !odd;
    }

  /* Restore previous clipping rectangle(s) */
  [[NSGraphicsContext currentContext] restoreGraphicsState];
}

/* Draw a dashed underline of thickness THICKNESS and width WIDTH onto
   the focused frame at a vertical offset of OFFSET from the position of
   the glyph string S, with each segment SEGMENT pixels in length.  */

static void
ns_draw_dash (struct glyph_string *s, int width, int segment,
	      int offset, int thickness)
{
  CGFloat pattern[2], y_center = s->ybase + offset + thickness / 2.0;
  NSBezierPath *path = [[NSBezierPath alloc] init];

  pattern[0] = segment;
  pattern[1] = segment;

  [path setLineDash: pattern count: 2 phase: (CGFloat) s->x];
  [path setLineWidth: thickness];
  [path moveToPoint: NSMakePoint (s->x, y_center)];
  [path lineToPoint: NSMakePoint (s->x + width, y_center)];
  [path stroke];
  [path release];
}

/* Draw an underline of STYLE onto the focused frame at an offset of
   POSITION from the baseline of the glyph string S, S->WIDTH in length,
   and THICKNESS in height.  */

static void
ns_fill_underline (struct glyph_string *s, enum face_underline_type style,
		   int position, int thickness)
{
  int segment;
  NSRect rect;

  segment = thickness * 3;

  switch (style)
    {
      /* FACE_UNDERLINE_DOUBLE_LINE is treated identically to SINGLE, as
	 the second line will be filled by another invocation of this
	 function.  */
    case FACE_UNDERLINE_SINGLE:
    case FACE_UNDERLINE_DOUBLE_LINE:
      rect = NSMakeRect (s->x, s->ybase + position, s->width, thickness);
      NSRectFill (rect);
      break;

    case FACE_UNDERLINE_DOTS:
      segment = thickness;
      FALLTHROUGH;

    case FACE_UNDERLINE_DASHES:
      ns_draw_dash (s, s->width, segment, position, thickness);
      break;

    case FACE_NO_UNDERLINE:
    case FACE_UNDERLINE_WAVE:
    default:
      emacs_abort ();
    }
}

static void
ns_draw_text_decoration (struct glyph_string *s, struct face *face,
                         NSColor *defaultCol, CGFloat width, CGFloat x)
/* --------------------------------------------------------------------------
   Draw underline, overline, and strike-through on glyph string s.
   -------------------------------------------------------------------------- */
{
  if (s->for_overlaps)
    return;

  if (s->hl == DRAW_CURSOR)
    [FRAME_BACKGROUND_COLOR (s->f) set];
  else
    [defaultCol set];

  /* Do underline.  */
  if (face->underline)
    {
      if (s->face->underline == FACE_UNDERLINE_WAVE)
        {
          if (!face->underline_defaulted_p)
            [[NSColor colorWithUnsignedLong:face->underline_color] set];

          ns_draw_underwave (s, width, x);
        }
      else if (face->underline >= FACE_UNDERLINE_SINGLE)
        {
          unsigned long thickness, position;

          /* If the prev was underlined, match its appearance.  */
          if (s->prev
	      && (s->prev->face->underline != FACE_UNDERLINE_WAVE
		  && s->prev->face->underline >= FACE_UNDERLINE_SINGLE)
              && s->prev->underline_thickness > 0
	      && (s->prev->face->underline_at_descent_line_p
		  == s->face->underline_at_descent_line_p)
	      && (s->prev->face->underline_pixels_above_descent_line
		  == s->face->underline_pixels_above_descent_line))
            {
              thickness = s->prev->underline_thickness;
              position = s->prev->underline_position;
            }
          else
            {
	      struct font *font = font_for_underline_metrics (s);
              unsigned long descent = s->y + s->height - s->ybase;
              unsigned long minimum_offset;
              BOOL underline_at_descent_line, use_underline_position_properties;
	      Lisp_Object val = (WINDOW_BUFFER_LOCAL_VALUE
				 (Qunderline_minimum_offset, s->w));

	      if (FIXNUMP (val))
		minimum_offset = XFIXNAT (val);
	      else
		minimum_offset = 1;

	      val = (WINDOW_BUFFER_LOCAL_VALUE
		     (Qx_underline_at_descent_line, s->w));
	      underline_at_descent_line = (!(NILP (val) || EQ (val, Qunbound))
					   || s->face->underline_at_descent_line_p);

	      val = (WINDOW_BUFFER_LOCAL_VALUE
		     (Qx_use_underline_position_properties, s->w));
	      use_underline_position_properties
		= !(NILP (val) || EQ (val, Qunbound));

              /* Use underline thickness of font, defaulting to 1.  */
              thickness = (font && font->underline_thickness > 0)
                ? font->underline_thickness : 1;

              /* Determine the offset of underlining from the baseline.  */
              if (underline_at_descent_line)
                position = (descent - thickness
			    - s->face->underline_pixels_above_descent_line);
              else if (use_underline_position_properties
                       && font && font->underline_position >= 0)
                position = font->underline_position;
              else if (font)
                position = lround (font->descent / 2);
              else
                position = minimum_offset;

	      if (!s->face->underline_pixels_above_descent_line)
		position = max (position, minimum_offset);

              /* Ensure underlining is not cropped.  */
              if (descent <= position)
                {
                  position = descent - 1;
                  thickness = 1;
                }
              else if (descent < position + thickness)
                thickness = 1;
            }

          s->underline_thickness = thickness;
          s->underline_position = position;

          if (!face->underline_defaulted_p)
            [[NSColor colorWithUnsignedLong:face->underline_color] set];

	  ns_fill_underline (s, s->face->underline, position,
			     thickness);

	  /* Place a second underline above the first if this was
	     requested in the face specification.  */

	  if (s->face->underline == FACE_UNDERLINE_DOUBLE_LINE)
	    {
	      /* Compute the position of the second underline.  */
	      position = position - thickness - 1;
	      ns_fill_underline (s, s->face->underline, position,
				 thickness);
	    }
        }
    }
  /* Do overline. We follow other terms in using a thickness of 1
     and ignoring overline_margin.  */
  if (face->overline_p)
    {
      NSRect r;
      r = NSMakeRect (x, s->y, width, 1);

      if (!face->overline_color_defaulted_p)
        [[NSColor colorWithUnsignedLong:face->overline_color] set];

      NSRectFill (r);
    }

  /* Do strike-through.  We follow other terms for thickness and
     vertical position.  */
  if (face->strike_through_p)
    {
      NSRect r;
      /* Y-coordinate and height of the glyph string's first glyph.
	 We cannot use s->y and s->height because those could be
	 larger if there are taller display elements (e.g., characters
	 displayed with a larger font) in the same glyph row.  */
      int glyph_y = s->ybase - s->first_glyph->ascent;
      int glyph_height = s->first_glyph->ascent + s->first_glyph->descent;
      /* Strike-through width and offset from the glyph string's
	 top edge.  */
      unsigned long h = 1;
      unsigned long dy;

      dy = lrint ((glyph_height - h) / 2);
      r = NSMakeRect (x, glyph_y + dy, width, 1);

      if (!face->strike_through_color_defaulted_p)
        [[NSColor colorWithUnsignedLong:face->strike_through_color] set];

      NSRectFill (r);
    }
}

static void
ns_draw_box (NSRect r, CGFloat hthickness, CGFloat vthickness,
             NSColor *col, char left_p, char right_p)
/* --------------------------------------------------------------------------
    Draw an unfilled rect inside r, optionally leaving left and/or right open.
    Note we can't just use an NSDrawRect command, because of the possibility
    of some sides not being drawn, and because the rect will be filled.
   -------------------------------------------------------------------------- */
{
  NSRect s = r;
  [col set];

  /* top, bottom */
  s.size.height = hthickness;
  NSRectFill (s);
  s.origin.y += r.size.height - hthickness;
  NSRectFill (s);

  s.size.height = r.size.height;
  s.origin.y = r.origin.y;

  /* left, right (optional) */
  s.size.width = vthickness;
  if (left_p)
    NSRectFill (s);
  if (right_p)
    {
      s.origin.x += r.size.width - vthickness;
      NSRectFill (s);
    }
}

/* Set up colors for the relief lines around glyph string S.  */

static void
ns_setup_relief_colors (struct glyph_string *s)
{
  struct ns_output *di = FRAME_OUTPUT_DATA (s->f);
  NSColor *color;

  if (s->face->use_box_color_for_shadows_p)
    color = [NSColor colorWithUnsignedLong: s->face->box_color];
  else
    color = [NSColor colorWithUnsignedLong: s->face->background];

  if (s->hl == DRAW_CURSOR)
    color = FRAME_CURSOR_COLOR (s->f);

  if (color == nil)
    color = [NSColor grayColor];

  if (color != di->relief_background_color)
    {
      [di->relief_background_color release];
      di->relief_background_color = [color retain];
      [di->light_relief_color release];
      di->light_relief_color = [[color highlightWithLevel: 0.4] retain];
      [di->dark_relief_color release];
      di->dark_relief_color = [[color shadowWithLevel: 0.4] retain];
    }
}

static void
ns_draw_relief (NSRect outer, int hthickness, int vthickness, char raised_p,
		char top_p, char bottom_p, char left_p, char right_p,
		struct glyph_string *s)
/* --------------------------------------------------------------------------
    Draw a relief rect inside r, optionally leaving some sides open.
    Note we can't just use an NSDrawBezel command, because of the possibility
    of some sides not being drawn, and because the rect will be filled.
   -------------------------------------------------------------------------- */
{
  NSRect inner;
  NSBezierPath *p = nil;

  NSTRACE ("ns_draw_relief");

  /* set up colors */
  ns_setup_relief_colors (s);

  /* Calculate the inner rectangle.  */
  inner = outer;

  if (left_p)
    {
      inner.origin.x += vthickness;
      inner.size.width -= vthickness;
    }

  if (right_p)
    inner.size.width -= vthickness;

  if (top_p)
    {
      inner.origin.y += hthickness;
      inner.size.height -= hthickness;
    }

  if (bottom_p)
    inner.size.height -= hthickness;

  struct ns_output *di = FRAME_OUTPUT_DATA (s->f);

  [(raised_p ? di->light_relief_color : di->dark_relief_color) set];

  if (top_p || left_p)
    {
      p = [NSBezierPath bezierPath];

      [p moveToPoint: NSMakePoint (NSMinX (outer), NSMinY (outer))];
      if (top_p)
        {
          [p lineToPoint: NSMakePoint (NSMaxX (outer), NSMinY (outer))];
          [p lineToPoint: NSMakePoint (NSMaxX (inner), NSMinY (inner))];
        }
      [p lineToPoint: NSMakePoint (NSMinX (inner), NSMinY (inner))];
      if (left_p)
        {
          [p lineToPoint: NSMakePoint (NSMinX (inner), NSMaxY (inner))];
          [p lineToPoint: NSMakePoint (NSMinX (outer), NSMaxY (outer))];
        }
      [p closePath];
      [p fill];
    }

  [(raised_p ? di->dark_relief_color : di->light_relief_color) set];

  if (bottom_p || right_p)
    {
      p = [NSBezierPath bezierPath];

      [p moveToPoint: NSMakePoint (NSMaxX (outer), NSMaxY (outer))];
      if (right_p)
        {
          [p lineToPoint: NSMakePoint (NSMaxX (outer), NSMinY (outer))];
          [p lineToPoint: NSMakePoint (NSMaxX (inner), NSMinY (inner))];
        }
      [p lineToPoint:NSMakePoint (NSMaxX (inner), NSMaxY (inner))];
      if (bottom_p)
        {
          [p lineToPoint: NSMakePoint (NSMinX (inner), NSMaxY (inner))];
          [p lineToPoint: NSMakePoint (NSMinX (outer), NSMaxY (outer))];
        }
      [p closePath];
      [p fill];
    }

  /* If one of h/vthickness are more than 1, draw the outermost line
     on the respective sides in the black relief color.  */

  if (p)
    [p removeAllPoints];
  else
    p = [NSBezierPath bezierPath];

  if (hthickness > 1 && top_p)
    {
      [p moveToPoint: NSMakePoint (NSMinX (outer),
				   NSMinY (outer) + 0.5)];
      [p lineToPoint: NSMakePoint (NSMaxX (outer),
				   NSMinY (outer) + 0.5)];
    }

  if (hthickness > 1 && bottom_p)
    {
      [p moveToPoint: NSMakePoint (NSMinX (outer),
				   NSMaxY (outer) - 0.5)];
      [p lineToPoint: NSMakePoint (NSMaxX (outer),
				   NSMaxY (outer) - 0.5)];
    }

  if (vthickness > 1 && left_p)
    {
      [p moveToPoint: NSMakePoint (NSMinX (outer) + 0.5,
				   NSMinY (outer) + 0.5)];
      [p lineToPoint: NSMakePoint (NSMinX (outer) + 0.5,
				   NSMaxY (outer) - 0.5)];
    }

  if (vthickness > 1 && left_p)
    {
      [p moveToPoint: NSMakePoint (NSMinX (outer) + 0.5,
				   NSMinY (outer) + 0.5)];
      [p lineToPoint: NSMakePoint (NSMinX (outer) + 0.5,
				   NSMaxY (outer) - 0.5)];
    }

  [di->dark_relief_color set];
  [p stroke];

  if (vthickness > 1 && hthickness > 1)
    {
      [FRAME_BACKGROUND_COLOR (s->f) set];

      if (left_p && top_p)
	[NSBezierPath fillRect: NSMakeRect (NSMinX (outer),
					    NSMinY (outer),
					    1, 1)];

      if (right_p && top_p)
	[NSBezierPath fillRect: NSMakeRect (NSMaxX (outer) - 1,
					    NSMinY (outer),
					    1, 1)];

      if (right_p && bottom_p)
	[NSBezierPath fillRect: NSMakeRect (NSMaxX (outer) - 1,
					    NSMaxY (outer) - 1,
					    1, 1)];

      if (left_p && bottom_p)
	[NSBezierPath fillRect: NSMakeRect (NSMinX (outer),
					    NSMaxY (outer) - 1,
					    1, 1)];
    }
}


static void
ns_dumpglyphs_box_or_relief (struct glyph_string *s)
/* --------------------------------------------------------------------------
      Function modeled after x_draw_glyph_string_box ().
      Sets up parameters for drawing.
   -------------------------------------------------------------------------- */
{
  int right_x, last_x;
  char left_p, right_p;
  struct glyph *last_glyph;
  NSRect r;
  int hthickness, vthickness;
  struct face *face = s->face;

  vthickness = face->box_vertical_line_width;
  hthickness = face->box_horizontal_line_width;

  NSTRACE ("ns_dumpglyphs_box_or_relief");

  last_x = ((s->row->full_width_p && !s->w->pseudo_window_p)
	    ? WINDOW_RIGHT_EDGE_X (s->w)
	    : window_box_right (s->w, s->area));
  if (s->cmp || s->img)
    last_glyph = s->first_glyph;
  else if (s->first_glyph->type == COMPOSITE_GLYPH
	   && s->first_glyph->u.cmp.automatic)
    {
        struct glyph *end = s->row->glyphs[s->area] + s->row->used[s->area];
	struct glyph *g = s->first_glyph;
	for (last_glyph = g++;
	     g < end && g->u.cmp.automatic && g->u.cmp.id == s->cmp_id
	       && g->slice.cmp.to < s->cmp_to;
	     last_glyph = g++)
	  ;
    }
  else
    last_glyph = s->first_glyph + s->nchars - 1;

  right_x = ((s->row->full_width_p && s->extends_to_end_of_line_p
	      ? last_x - 1 : min (last_x, s->x + s->background_width) - 1));

  left_p = (s->first_glyph->left_box_line_p
	    || (s->hl == DRAW_MOUSE_FACE
		&& (s->prev == NULL || s->prev->hl != s->hl)));
  right_p = (last_glyph->right_box_line_p
	     || (s->hl == DRAW_MOUSE_FACE
		 && (s->next == NULL || s->next->hl != s->hl)));

  r = NSMakeRect (s->x, s->y, right_x - s->x + 1, s->height);

  /* TODO: Sometimes box_color is 0 and this seems wrong; should investigate.  */
  if (s->face->box == FACE_SIMPLE_BOX && s->face->box_color)
    {
      ns_draw_box (r, abs (hthickness), abs (vthickness),
                   [NSColor colorWithUnsignedLong:face->box_color],
                   left_p, right_p);
    }
  else
    {
      ns_draw_relief (r, abs (hthickness), abs (vthickness),
                      s->face->box == FACE_RAISED_BOX,
                      1, 1, left_p, right_p, s);
    }
}

static void
ns_maybe_dumpglyphs_background (struct glyph_string *s, char force_p)
/* --------------------------------------------------------------------------
      Modeled after x_draw_glyph_string_background, which draws BG in
      certain cases.  Others are left to the text rendering routine.
   -------------------------------------------------------------------------- */
{
  struct face *face = s->face;
  NSRect r;

  NSTRACE ("ns_maybe_dumpglyphs_background");

  if (!s->background_filled_p)
    {
      int box_line_width = max (s->face->box_horizontal_line_width, 0);

      if (s->stippled_p)
	{
	  struct ns_display_info *dpyinfo = FRAME_DISPLAY_INFO (s->f);
	  [[dpyinfo->bitmaps[face->stipple-1].img stippleMask] set];
	  goto fill;
	}
      else if (FONT_HEIGHT (s->font) < s->height - 2 * box_line_width
	       /* When xdisp.c ignores FONT_HEIGHT, we cannot trust font
		  dimensions, since the actual glyphs might be much
		  smaller.  So in that case we always clear the
		  rectangle with background color.  */
	       || FONT_TOO_HIGH (s->font)
	       || s->font_not_found_p
	       || s->extends_to_end_of_line_p
	       || force_p)
	{
	  if (s->hl != DRAW_CURSOR)
	    [(NS_FACE_BACKGROUND (face) != 0
	      ? [NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND (face)]
	      : FRAME_BACKGROUND_COLOR (s->f)) set];
	  else if (face && (NS_FACE_BACKGROUND (face)
			    == [(NSColor *) FRAME_CURSOR_COLOR (s->f)
					    unsignedLong]))
	    [[NSColor colorWithUnsignedLong:NS_FACE_FOREGROUND (face)] set];
	  else
	    [FRAME_CURSOR_COLOR (s->f) set];

	fill:
	  r = NSMakeRect (s->x, s->y + box_line_width,
			  s->background_width,
			  s->height - 2 * box_line_width);
	  NSRectFill (r);
	  s->background_filled_p = 1;
	}
    }
}

static void
ns_draw_image_relief (struct glyph_string *s)
{
  int x1, y1, thick;
  bool raised_p, top_p, bot_p, left_p, right_p;
  int extra_x, extra_y;
  int x = s->x;
  int y = s->ybase - image_ascent (s->img, s->face, &s->slice);

  /* If first glyph of S has a left box line, start drawing it to the
     right of that line.  */
  if (s->face->box != FACE_NO_BOX
      && s->first_glyph->left_box_line_p
      && s->slice.x == 0)
    x += max (s->face->box_vertical_line_width, 0);

  /* If there is a margin around the image, adjust x- and y-position
     by that margin.  */
  if (s->slice.x == 0)
    x += s->img->hmargin;
  if (s->slice.y == 0)
    y += s->img->vmargin;

  if (s->hl == DRAW_IMAGE_SUNKEN
      || s->hl == DRAW_IMAGE_RAISED)
    {
      if (s->face->id == TAB_BAR_FACE_ID)
	thick = (tab_bar_button_relief < 0
		 ? DEFAULT_TAB_BAR_BUTTON_RELIEF
		 : min (tab_bar_button_relief, 1000000));
      else
	thick = (tool_bar_button_relief < 0
		 ? DEFAULT_TOOL_BAR_BUTTON_RELIEF
		 : min (tool_bar_button_relief, 1000000));
      raised_p = s->hl == DRAW_IMAGE_RAISED;
    }
  else
    {
      thick = eabs (s->img->relief);
      raised_p = s->img->relief > 0;
    }

  x1 = x + s->slice.width - 1;
  y1 = y + s->slice.height - 1;

  extra_x = extra_y = 0;
  if (s->face->id == TAB_BAR_FACE_ID)
    {
      if (CONSP (Vtab_bar_button_margin)
	  && FIXNUMP (XCAR (Vtab_bar_button_margin))
	  && FIXNUMP (XCDR (Vtab_bar_button_margin)))
	{
	  extra_x = XFIXNUM (XCAR (Vtab_bar_button_margin)) - thick;
	  extra_y = XFIXNUM (XCDR (Vtab_bar_button_margin)) - thick;
	}
      else if (FIXNUMP (Vtab_bar_button_margin))
	extra_x = extra_y = XFIXNUM (Vtab_bar_button_margin) - thick;
    }

  if (s->face->id == TOOL_BAR_FACE_ID)
    {
      if (CONSP (Vtool_bar_button_margin)
	  && FIXNUMP (XCAR (Vtool_bar_button_margin))
	  && FIXNUMP (XCDR (Vtool_bar_button_margin)))
	{
	  extra_x = XFIXNUM (XCAR (Vtool_bar_button_margin));
	  extra_y = XFIXNUM (XCDR (Vtool_bar_button_margin));
	}
      else if (FIXNUMP (Vtool_bar_button_margin))
	extra_x = extra_y = XFIXNUM (Vtool_bar_button_margin);
    }

  top_p = bot_p = left_p = right_p = false;

  if (s->slice.x == 0)
    x -= thick + extra_x, left_p = true;
  if (s->slice.y == 0)
    y -= thick + extra_y, top_p = true;
  if (s->slice.x + s->slice.width == s->img->width)
    x1 += thick + extra_x, right_p = true;
  if (s->slice.y + s->slice.height == s->img->height)
    y1 += thick + extra_y, bot_p = true;

  ns_draw_relief (NSMakeRect (x, y, x1 - x + 1, y1 - y + 1), thick,
		  thick, raised_p, top_p, bot_p, left_p, right_p, s);
}

static void
ns_dumpglyphs_image (struct glyph_string *s, NSRect r)
/* --------------------------------------------------------------------------
      Renders an image and associated borders.
   -------------------------------------------------------------------------- */
{
  EmacsImage *img = s->img->pixmap;
  int box_line_vwidth = max (s->face->box_horizontal_line_width, 0);
  int x = s->x, y = s->ybase - image_ascent (s->img, s->face, &s->slice);
  int bg_x, bg_y, bg_height;
  NSRect br;
  struct face *face = s->face;
  NSColor *tdCol;

  NSTRACE ("ns_dumpglyphs_image");

  if (s->face->box != FACE_NO_BOX
      && s->first_glyph->left_box_line_p && s->slice.x == 0)
    x += max (s->face->box_vertical_line_width, 0);

  bg_x = x;
  bg_y =  s->slice.y == 0 ? s->y : s->y + box_line_vwidth;
  bg_height = s->height;
  /* other terms have this, but was causing problems w/tabbar mode */
  /* - 2 * box_line_vwidth; */

  if (s->slice.x == 0) x += s->img->hmargin;
  if (s->slice.y == 0) y += s->img->vmargin;

  /* Draw BG: if we need larger area than image itself cleared, do that,
     otherwise, since we composite the image under NS (instead of mucking
     with its background color), we must clear just the image area.  */

  [[NSColor colorWithUnsignedLong:NS_FACE_BACKGROUND (face)] set];

  if (bg_height > s->slice.height || s->img->hmargin || s->img->vmargin
      || s->img->mask || s->img->pixmap == 0 || s->width != s->background_width)
    {
      br = NSMakeRect (bg_x, bg_y, s->background_width, bg_height);
      s->background_filled_p = 1;
    }
  else
    {
      br = NSMakeRect (x, y, s->slice.width, s->slice.height);
    }

  NSRectFill (br);

  /* Draw the image... do we need to draw placeholder if img == nil?  */
  if (img != nil)
    {
      /* The idea here is that the clipped area is set in the normal
         view coordinate system, then we transform the coordinate
         system so that when we draw the image it is rotated, resized
         or whatever as required.  This is kind of backwards, but
         there's no way to apply the transform to the image without
         creating a whole new bitmap.  */
      NSRect dr = NSMakeRect (x, y, s->slice.width, s->slice.height);
      NSRect ir = NSMakeRect (0, 0, [img size].width, [img size].height);

      NSAffineTransform *setOrigin = [NSAffineTransform transform];

      [[NSGraphicsContext currentContext] saveGraphicsState];

      /* Because of the transforms it's difficult to work out what
         portion of the original, untransformed, image will be drawn,
         so the clipping area will ensure we draw only the correct
         bit.  */
      NSRectClip (dr);

      [setOrigin translateXBy:x - s->slice.x yBy:y - s->slice.y];
      [setOrigin concat];

      NSAffineTransform *doTransform = [NSAffineTransform transform];

      /* ImageMagick images don't have transforms.  */
      if (img->transform)
        [doTransform appendTransform:img->transform];

      [doTransform concat];

      /* Smoothing is the default, so if we don't want smoothing we
         have to turn it off.  */
      if (! img->smoothing)
        [[NSGraphicsContext currentContext]
          setImageInterpolation:NSImageInterpolationNone];

      [img drawInRect:ir fromRect:ir
            operation:NSCompositingOperationSourceOver
             fraction:1.0 respectFlipped:YES hints:nil];

      /* Apparently image interpolation is not reset with
         restoreGraphicsState, so we have to manually reset it.  */
      if (! img->smoothing)
        [[NSGraphicsContext currentContext]
          setImageInterpolation:NSImageInterpolationDefault];

      [[NSGraphicsContext currentContext] restoreGraphicsState];
    }

  if (s->hl == DRAW_CURSOR)
    {
      [FRAME_CURSOR_COLOR (s->f) set];
      tdCol = [NSColor colorWithUnsignedLong: NS_FACE_BACKGROUND (face)];
    }
  else
    tdCol = [NSColor colorWithUnsignedLong: NS_FACE_FOREGROUND (face)];

  /* Draw underline, overline, strike-through.  */
  ns_draw_text_decoration (s, face, tdCol, br.size.width, br.origin.x);

  /* If we must draw a relief around the image, do it.  */
  if (s->img->relief
      || s->hl == DRAW_IMAGE_RAISED
      || s->hl == DRAW_IMAGE_SUNKEN)
    ns_draw_image_relief (s);

  /* If there is no mask, the background won't be seen, so draw a
     rectangle on the image for the cursor.  Do this for all images,
     getting transparency right is not reliable.  */
  if (s->hl == DRAW_CURSOR)
    {
      int thickness = abs (s->img->relief);
      if (thickness == 0) thickness = 1;
      ns_draw_box (br, thickness, thickness,
		   FRAME_CURSOR_COLOR (s->f), 1, 1);
    }
}


static void
ns_draw_stretch_glyph_string (struct glyph_string *s)
{
  struct face *face;
  NSColor *fg_color;

  if (s->hl == DRAW_CURSOR && !x_stretch_cursor_p)
    {
      /* If `x-stretch-cursor' is nil, don't draw a block cursor as
	 wide as the stretch glyph.  */
      int width, background_width = s->background_width;
      int x = s->x;

      if (!s->row->reversed_p)
	{
	  int left_x = window_box_left_offset (s->w, TEXT_AREA);

	  if (x < left_x)
	    {
	      background_width -= left_x - x;
	      x = left_x;
	    }
	}
      else
	{
	  /* In R2L rows, draw the cursor on the right edge of the
	     stretch glyph.  */
	  int right_x = window_box_right (s->w, TEXT_AREA);

	  if (x + background_width > right_x)
	    background_width -= x - right_x;
	  x += background_width;
	}

      width = min (FRAME_COLUMN_WIDTH (s->f), background_width);
      if (s->row->reversed_p)
	x -= width;

      if (s->hl == DRAW_CURSOR)
	[FRAME_CURSOR_COLOR (s->f) set];
      else
	[[NSColor colorWithUnsignedLong: s->face->foreground] set];

      NSRectFill (NSMakeRect (x, s->y, width, s->height));

      /* Clear rest using the GC of the original non-cursor face.  */
      if (width < background_width)
	{
	  int y = s->y;
	  int w = background_width - width, h = s->height;

	  if (!s->row->reversed_p)
	    x += width;
	  else
	    x = s->x;

	  if (s->row->mouse_face_p
	      && cursor_in_mouse_face_p (s->w))
	    {
	      face = FACE_FROM_ID_OR_NULL (s->f,
					   MOUSE_HL_INFO (s->f)->mouse_face_face_id);

	      // shouldn't be !face ?
	      if (!s->face)
		face = FACE_FROM_ID (s->f, MOUSE_FACE_ID);
	      prepare_face_for_display (s->f, face);

	      [[NSColor colorWithUnsignedLong: face->background] set];
	    }
	  else
	    [[NSColor colorWithUnsignedLong: s->face->background] set];
	  NSRectFill (NSMakeRect (x, y, w, h));
	}
    }
  else if (!s->background_filled_p)
    {
      int background_width = s->background_width;
      int x = s->x, text_left_x = window_box_left (s->w, TEXT_AREA);

      /* Don't draw into left fringe or scrollbar area except for
         header line and mode line.  */
      if (s->area == TEXT_AREA
	  && x < text_left_x && !s->row->mode_line_p)
	{
	  background_width -= text_left_x - x;
	  x = text_left_x;
	}

      if (!s->row->stipple_p)
	s->row->stipple_p = s->stippled_p;

      if (background_width > 0)
	{
	  struct ns_display_info *dpyinfo;

	  dpyinfo = FRAME_DISPLAY_INFO (s->f);
	  if (s->hl == DRAW_CURSOR)
	    [FRAME_CURSOR_COLOR (s->f) set];
	  else if (s->stippled_p)
	    [[dpyinfo->bitmaps[s->face->stipple - 1].img stippleMask] set];
	  else
	    [[NSColor colorWithUnsignedLong: s->face->background] set];

	  NSRectFill (NSMakeRect (x, s->y, background_width, s->height));
	}
    }

  /* Draw overlining, etc. on the stretch glyph (or the part of the
     stretch glyph after the cursor).  If the glyph has a box, then
     decorations will be drawn after drawing the box in
     ns_draw_glyph_string, in order to prevent them from being
     overwritten by the box.  */
  if (s->face->box == FACE_NO_BOX)
    {
      fg_color = [NSColor colorWithUnsignedLong:
			    NS_FACE_FOREGROUND (s->face)];
      ns_draw_text_decoration (s, s->face, fg_color,
			       s->background_width, s->x);
    }
}

static void
ns_draw_glyph_string_foreground (struct glyph_string *s)
{
  int x;
  struct font *font = s->font;

  /* If first glyph of S has a left box line, start drawing the text
     of S to the right of that box line.  */
  if (s->face && s->face->box != FACE_NO_BOX
      && s->first_glyph->left_box_line_p)
    x = s->x + max (s->face->box_vertical_line_width, 0);
  else
    x = s->x;

  font->driver->draw
    (s, s->cmp_from, s->nchars, x, s->ybase,
     !s->for_overlaps && !s->background_filled_p);
}


static void
ns_draw_composite_glyph_string_foreground (struct glyph_string *s)
{
  int i, j, x;
  struct font *font = s->font;

  /* If first glyph of S has a left box line, start drawing the text
     of S to the right of that box line.  */
  if (s->face && s->face->box != FACE_NO_BOX
      && s->first_glyph->left_box_line_p)
    x = s->x + max (s->face->box_vertical_line_width, 0);
  else
    x = s->x;

  /* S is a glyph string for a composition.  S->cmp_from is the index
     of the first character drawn for glyphs of this composition.
     S->cmp_from == 0 means we are drawing the very first character of
     this composition.  */

  /* Draw a rectangle for the composition if the font for the very
     first character of the composition could not be loaded.  */
  if (s->font_not_found_p)
    {
      if (s->cmp_from == 0)
        {
          NSRect r = NSMakeRect (s->x, s->y, s->width-1, s->height -1);
          ns_draw_box (r, 1, 1, FRAME_CURSOR_COLOR (s->f), 1, 1);
        }
    }
  else if (! s->first_glyph->u.cmp.automatic)
    {
      int y = s->ybase;

      for (i = 0, j = s->cmp_from; i < s->nchars; i++, j++)
	/* TAB in a composition means display glyphs with padding
	   space on the left or right.  */
	if (COMPOSITION_GLYPH (s->cmp, j) != '\t')
	  {
	    int xx = x + s->cmp->offsets[j * 2];
	    int yy = y - s->cmp->offsets[j * 2 + 1];

	    font->driver->draw (s, j, j + 1, xx, yy, false);
	    if (s->face->overstrike)
	      font->driver->draw (s, j, j + 1, xx + 1, yy, false);
	  }
    }
  else
    {
      Lisp_Object gstring = composition_gstring_from_id (s->cmp_id);
      Lisp_Object glyph;
      int y = s->ybase;
      int width = 0;

      for (i = j = s->cmp_from; i < s->cmp_to; i++)
	{
	  glyph = LGSTRING_GLYPH (gstring, i);
	  if (NILP (LGLYPH_ADJUSTMENT (glyph)))
	    width += LGLYPH_WIDTH (glyph);
	  else
	    {
	      int xoff, yoff, wadjust;

	      if (j < i)
		{
		  font->driver->draw (s, j, i, x, y, false);
		  if (s->face->overstrike)
		    font->driver->draw (s, j, i, x + 1, y, false);
		  x += width;
		}
	      xoff = LGLYPH_XOFF (glyph);
	      yoff = LGLYPH_YOFF (glyph);
	      wadjust = LGLYPH_WADJUST (glyph);
	      font->driver->draw (s, i, i + 1, x + xoff, y + yoff, false);
	      if (s->face->overstrike)
		font->driver->draw (s, i, i + 1, x + xoff + 1, y + yoff,
				    false);
	      x += wadjust;
	      j = i + 1;
	      width = 0;
	    }
	}
      if (j < i)
	{
	  font->driver->draw (s, j, i, x, y, false);
	  if (s->face->overstrike)
	    font->driver->draw (s, j, i, x + 1, y, false);
	}
    }
}

/* Draw the foreground of glyph string S for glyphless characters.  */
static void
ns_draw_glyphless_glyph_string_foreground (struct glyph_string *s)
{
  struct glyph *glyph = s->first_glyph;
  NSGlyph char2b[8];
  int x, i, j;

  /* If first glyph of S has a left box line, start drawing the text
     of S to the right of that box line.  */
  if (s->face && s->face->box != FACE_NO_BOX
      && s->first_glyph->left_box_line_p)
    x = s->x + max (s->face->box_vertical_line_width, 0);
  else
    x = s->x;

  s->char2b = char2b;

  for (i = 0; i < s->nchars; i++, glyph++)
    {
#ifdef GCC_LINT
      enum { PACIFY_GCC_BUG_81401 = 1 };
#else
      enum { PACIFY_GCC_BUG_81401 = 0 };
#endif
      char buf[8 + PACIFY_GCC_BUG_81401];
      char *str = NULL;
      int len = glyph->u.glyphless.len;

      if (glyph->u.glyphless.method == GLYPHLESS_DISPLAY_ACRONYM)
	{
	  if (len > 0
	      && CHAR_TABLE_P (Vglyphless_char_display)
	      && (CHAR_TABLE_EXTRA_SLOTS (XCHAR_TABLE (Vglyphless_char_display))
		  >= 1))
	    {
	      Lisp_Object acronym
		= (! glyph->u.glyphless.for_no_font
		   ? CHAR_TABLE_REF (Vglyphless_char_display,
				     glyph->u.glyphless.ch)
		   : XCHAR_TABLE (Vglyphless_char_display)->extras[0]);
	      if (CONSP (acronym))
		acronym = XCAR (acronym);
	      if (STRINGP (acronym))
		str = SSDATA (acronym);
	    }
	}
      else if (glyph->u.glyphless.method == GLYPHLESS_DISPLAY_HEX_CODE)
	{
	  unsigned int ch = glyph->u.glyphless.ch;
	  eassume (ch <= MAX_CHAR);
	  snprintf (buf, 8, "%0*X", ch < 0x10000 ? 4 : 6, ch);
	  str = buf;
	}

      if (str)
	{
	  int upper_len = (len + 1) / 2;

	  /* It is assured that all LEN characters in STR is ASCII.  */
	  for (j = 0; j < len; j++)
            char2b[j] = s->font->driver->encode_char (s->font, str[j]) & 0xFFFF;
	  s->font->driver->draw (s, 0, upper_len,
				 x + glyph->slice.glyphless.upper_xoff,
				 s->ybase + glyph->slice.glyphless.upper_yoff,
				 false);
	  s->font->driver->draw (s, upper_len, len,
				 x + glyph->slice.glyphless.lower_xoff,
				 s->ybase + glyph->slice.glyphless.lower_yoff,
				 false);
	}
      if (glyph->u.glyphless.method != GLYPHLESS_DISPLAY_THIN_SPACE)
        ns_draw_box (NSMakeRect (x, s->ybase - glyph->ascent,
                                 glyph->pixel_width - 1,
                                 glyph->ascent + glyph->descent - 1),
                     1, 1,
                     [NSColor colorWithUnsignedLong:NS_FACE_FOREGROUND (s->face)],
                     YES, YES);
      x += glyph->pixel_width;
   }

  /* GCC 12 complains even though nothing ever uses s->char2b after
     this function returns.  */
  s->char2b = NULL;
}

/* Transfer glyph string parameters from S's face to S itself.
   Set S->stipple_p as appropriate, taking the draw type into
   account.  */

static void
ns_set_glyph_string_gc (struct glyph_string *s)
{
  prepare_face_for_display (s->f, s->face);

  if (s->hl == DRAW_NORMAL_TEXT)
    {
      /* s->gc = s->face->gc; */
      s->stippled_p = s->face->stipple != 0;
    }
  else if (s->hl == DRAW_INVERSE_VIDEO)
    {
      /* x_set_mode_line_face_gc (s); */
      s->stippled_p = s->face->stipple != 0;
    }
  else if (s->hl == DRAW_CURSOR)
    {
      /* x_set_cursor_gc (s); */
      s->stippled_p = false;
    }
  else if (s->hl == DRAW_MOUSE_FACE)
    {
      /* x_set_mouse_face_gc (s); */
      s->stippled_p = s->face->stipple != 0;
    }
  else if (s->hl == DRAW_IMAGE_RAISED
	   || s->hl == DRAW_IMAGE_SUNKEN)
    {
      /* s->gc = s->face->gc; */
      s->stippled_p = s->face->stipple != 0;
    }
  else
    emacs_abort ();
}

static void
ns_draw_glyph_string (struct glyph_string *s)
/* --------------------------------------------------------------------------
      External (RIF): Main draw-text call.
   -------------------------------------------------------------------------- */
{
  /* TODO (optimize): focus for box and contents draw */
  NSRect r[2];
  int n;
  char box_drawn_p = 0;
  struct font *font = s->face->font;
  if (! font) font = FRAME_FONT (s->f);

  NSTRACE ("ns_draw_glyph_string (hl = %u)", s->hl);

  if (s->next && s->right_overhang && !s->for_overlaps)
    {
      int width;
      struct glyph_string *next;

      for (width = 0, next = s->next;
	   next && width < s->right_overhang;
	   width += next->width, next = next->next)
	if (next->first_glyph->type != IMAGE_GLYPH)
          {
	    ns_set_glyph_string_gc (next);
	    n = ns_get_glyph_string_clip_rect (s->next, r);
	    ns_focus (s->f, r, n);
            if (next->first_glyph->type != STRETCH_GLYPH)
	      ns_maybe_dumpglyphs_background (s->next, 1);
	    else
	      ns_draw_stretch_glyph_string (s->next);
	    ns_unfocus (s->f);
            next->num_clips = 0;
          }
    }

  ns_set_glyph_string_gc (s);

  if (!s->for_overlaps && s->face->box != FACE_NO_BOX
        && (s->first_glyph->type == CHAR_GLYPH
	    || s->first_glyph->type == COMPOSITE_GLYPH))
    {
      n = ns_get_glyph_string_clip_rect (s, r);
      ns_focus (s->f, r, n);
      ns_maybe_dumpglyphs_background (s, 1);
      ns_dumpglyphs_box_or_relief (s);
      ns_unfocus (s->f);
      box_drawn_p = 1;
    }

  n = ns_get_glyph_string_clip_rect (s, r);

  if (!s->clip_head /* draw_glyphs didn't specify a clip mask. */
      && !s->clip_tail
      && ((s->prev && s->prev->hl != s->hl && s->left_overhang)
	  || (s->next && s->next->hl != s->hl && s->right_overhang)))
    r[0] = NSIntersectionRect (r[0], NSMakeRect (s->x, s->y, s->width, s->height));

  ns_focus (s->f, r, n);

  switch (s->first_glyph->type)
    {

    case IMAGE_GLYPH:
      ns_dumpglyphs_image (s, r[0]);
      break;

    case XWIDGET_GLYPH:
      x_draw_xwidget_glyph_string (s);
      break;

    case STRETCH_GLYPH:
      ns_draw_stretch_glyph_string (s);
      break;

    case CHAR_GLYPH:
    case COMPOSITE_GLYPH:
      {
	BOOL isComposite = s->first_glyph->type == COMPOSITE_GLYPH;
	if (s->for_overlaps || (isComposite
				&& (s->cmp_from > 0
				    && ! s->first_glyph->u.cmp.automatic)))
	  s->background_filled_p = 1;
	else
	  ns_maybe_dumpglyphs_background
	    (s, s->first_glyph->type == COMPOSITE_GLYPH);

	if (isComposite)
	  ns_draw_composite_glyph_string_foreground (s);
	else
	  ns_draw_glyph_string_foreground (s);

	{
	  NSColor *col = (NS_FACE_FOREGROUND (s->face) != 0
			  ? [NSColor colorWithUnsignedLong:NS_FACE_FOREGROUND (s->face)]
			  : FRAME_FOREGROUND_COLOR (s->f));

	  /* Draw underline, overline, strike-through. */
	  ns_draw_text_decoration (s, s->face, col, s->width, s->x);
	}
      }

      break;

    case GLYPHLESS_GLYPH:
      if (s->for_overlaps || (s->cmp_from > 0
			      && ! s->first_glyph->u.cmp.automatic))
        s->background_filled_p = 1;
      else
        ns_maybe_dumpglyphs_background
          (s, s->first_glyph->type == COMPOSITE_GLYPH);
      ns_draw_glyphless_glyph_string_foreground (s);
      break;

    default:
      emacs_abort ();
    }

  /* Draw box if not done already.  */
  if (!s->for_overlaps && !box_drawn_p && s->face->box != FACE_NO_BOX)
    ns_dumpglyphs_box_or_relief (s);

  if (s->face->box != FACE_NO_BOX
      && s->first_glyph->type == STRETCH_GLYPH)
    {
      NSColor *fg_color;

      fg_color = [NSColor colorWithUnsignedLong: NS_FACE_FOREGROUND (s->face)];

      ns_draw_text_decoration (s, s->face, fg_color,
			       s->background_width, s->x);
    }

  ns_unfocus (s->f);

  /* Draw surrounding overhangs. */
  if (s->prev)
    {
      ns_focus (s->f, NULL, 0);
      struct glyph_string *prev;

      for (prev = s->prev; prev; prev = prev->prev)
	if (prev->hl != s->hl
	    && prev->x + prev->width + prev->right_overhang > s->x)
	  {
	    /* As prev was drawn while clipped to its own area, we
	       must draw the right_overhang part using s->hl now.  */
	    enum draw_glyphs_face save = prev->hl;

	    prev->hl = s->hl;
	    NSRect r = NSMakeRect (s->x, s->y, s->width, s->height);
	    NSRect rc;
	    get_glyph_string_clip_rect (s, &rc);
	    [[NSGraphicsContext currentContext] saveGraphicsState];
	    NSRectClip (r);
	    if (n)
	      NSRectClip (rc);
#ifdef NS_IMPL_GNUSTEP
	    DPSgsave ([NSGraphicsContext currentContext]);
	    DPSrectclip ([NSGraphicsContext currentContext], s->x, s->y,
			 s->width, s->height);
	    DPSrectclip ([NSGraphicsContext currentContext], NSMinX (rc),
			 NSMinY (rc), NSWidth (rc), NSHeight (rc));
#endif
	    if (prev->first_glyph->type == CHAR_GLYPH)
	      ns_draw_glyph_string_foreground (prev);
	    else
	      ns_draw_composite_glyph_string_foreground (prev);
#ifdef NS_IMPL_GNUSTEP
	    DPSgrestore ([NSGraphicsContext currentContext]);
#endif
	    [[NSGraphicsContext currentContext] restoreGraphicsState];
	    prev->hl = save;
	  }
      ns_unfocus (s->f);
    }

  if (s->next)
    {
      ns_focus (s->f, NULL, 0);
      struct glyph_string *next;

      for (next = s->next; next; next = next->next)
	if (next->hl != s->hl
	    && next->x - next->left_overhang < s->x + s->width)
	  {
	    /* As next will be drawn while clipped to its own area,
	       we must draw the left_overhang part using s->hl now.  */
	    enum draw_glyphs_face save = next->hl;

	    next->hl = s->hl;
	    NSRect r = NSMakeRect (s->x, s->y, s->width, s->height);
	    NSRect rc;
	    get_glyph_string_clip_rect (s, &rc);
	    [[NSGraphicsContext currentContext] saveGraphicsState];
	    NSRectClip (r);
	    NSRectClip (rc);
#ifdef NS_IMPL_GNUSTEP
	    DPSgsave ([NSGraphicsContext currentContext]);
	    DPSrectclip ([NSGraphicsContext currentContext], s->x, s->y,
			 s->width, s->height);
	    DPSrectclip ([NSGraphicsContext currentContext], NSMinX (rc),
			 NSMinY (rc), NSWidth (rc), NSHeight (rc));
#endif
	    if (next->first_glyph->type == CHAR_GLYPH)
	      ns_draw_glyph_string_foreground (next);
	    else
	      ns_draw_composite_glyph_string_foreground (next);
#ifdef NS_IMPL_GNUSTEP
	    DPSgrestore ([NSGraphicsContext currentContext]);
#endif
	    [[NSGraphicsContext currentContext] restoreGraphicsState];
	    next->hl = save;
	    next->clip_head = s->next;
	  }
      ns_unfocus (s->f);
    }
  s->num_clips = 0;
}

#ifdef NS_IMPL_GNUSTEP
static void
ns_update_window_end (struct window *w, bool cursor_on_p,
		      bool mouse_face_overwritten_p)
{
  NSTRACE ("ns_update_window_end (cursor_on_p = %d)", cursor_on_p);

  ns_redraw_scroll_bars (WINDOW_XFRAME (w));
}
#endif

static void
ns_flush_display (struct frame *f)
{
  struct input_event ie;

  EVENT_INIT (ie);
  ns_read_socket_1 (FRAME_TERMINAL (f), &ie, YES);
}

/* this and next define (many of the) public functions in this
   file.  */
/* gui_* are generic versions in xdisp.c that we, and other terms, get
   away with using despite presence in the "system dependent"
   redisplay interface.  In addition, many of the ns_ methods have
   code that is shared with all terms, indicating need for further
   refactoring.  */
struct redisplay_interface ns_redisplay_interface =
{
  ns_frame_parm_handlers,
  gui_produce_glyphs,
  gui_write_glyphs,
  gui_insert_glyphs,
  gui_clear_end_of_line,
  ns_scroll_run,
  ns_after_update_window_line,
  NULL, /* update_window_begin */
#ifndef NS_IMPL_GNUSTEP
  NULL, /* update_window_end   */
#else
  ns_update_window_end,
#endif
  ns_flush_display,
  gui_clear_window_mouse_face,
  gui_get_glyph_overhangs,
  gui_fix_overlapping_area,
  ns_draw_fringe_bitmap,
  ns_define_fringe_bitmap,
  ns_destroy_fringe_bitmap,
  ns_compute_glyph_string_overhangs,
  ns_draw_glyph_string,
  ns_define_frame_cursor,
  ns_clear_frame_area,
  ns_clear_under_internal_border, /* clear_under_internal_border */
  ns_draw_window_cursor,
  ns_draw_vertical_window_border,
  ns_draw_window_divider,
  ns_shift_glyphs_for_insert,
  ns_show_hourglass,
  ns_hide_hourglass,
  ns_default_font_parameter
};

/* ==========================================================================

    Scrollbar handling

   ========================================================================== */


void
ns_set_vertical_scroll_bar (struct window *window,
                           int portion, int whole, int position)
/* --------------------------------------------------------------------------
      External (hook): Update or add scrollbar
   -------------------------------------------------------------------------- */
{
  Lisp_Object win;
  NSRect r, v;
  struct frame *f = XFRAME (WINDOW_FRAME (window));
  EmacsView *view = FRAME_NS_VIEW (f);
  EmacsScroller *bar;
  int window_y, window_height;
  int top, left, height, width;
  BOOL update_p = YES;

  /* Optimization; display engine sends WAY too many of these.  */
  if (!NILP (window->vertical_scroll_bar))
    {
      bar = XNS_SCROLL_BAR (window->vertical_scroll_bar);
      if ([bar checkSamePosition: position portion: portion whole: whole])
        {
          if (view->scrollbarsNeedingUpdate == 0)
            {
              if (!windows_or_buffers_changed)
                  return;
            }
          else
            view->scrollbarsNeedingUpdate--;
          update_p = NO;
        }
    }

  NSTRACE ("ns_set_vertical_scroll_bar");

  /* Get dimensions.  */
  window_box (window, ANY_AREA, 0, &window_y, 0, &window_height);
  top = window_y;
  height = window_height;
  width = WINDOW_SCROLL_BAR_AREA_WIDTH (window);
  left = WINDOW_SCROLL_BAR_AREA_X (window);

  r = NSMakeRect (left, top, width, height);
  /* The parent view is flipped, so we need to flip y value.  */
  v = [view frame];
  r.origin.y = (v.size.height - r.size.height - r.origin.y);

  XSETWINDOW (win, window);
  block_input ();

  /* We want at least 5 lines to display a scrollbar.  */
  if (WINDOW_TOTAL_LINES (window) < 5)
    {
      if (!NILP (window->vertical_scroll_bar))
        {
          bar = XNS_SCROLL_BAR (window->vertical_scroll_bar);
          [bar removeFromSuperview];
          wset_vertical_scroll_bar (window, Qnil);
          [bar release];
          ns_clear_frame_area (f, left, top, width, height);
        }
      unblock_input ();
      return;
    }

  if (NILP (window->vertical_scroll_bar))
    {
      if (width > 0 && height > 0)
	ns_clear_frame_area (f, left, top, width, height);

      bar = [[EmacsScroller alloc] initFrame: r window: win];
      wset_vertical_scroll_bar (window, make_mint_ptr (bar));
      update_p = YES;
    }
  else
    {
      NSRect oldRect;
      bar = XNS_SCROLL_BAR (window->vertical_scroll_bar);
      oldRect = [bar frame];
      r.size.width = oldRect.size.width;
      if (FRAME_LIVE_P (f) && !NSEqualRects (oldRect, r))
        {
          if (! NSEqualRects (oldRect, r))
              ns_clear_frame_area (f, left, top, width, height);
          [bar setFrame: r];
        }
    }

  if (update_p)
    [bar setPosition: position portion: portion whole: whole];
  unblock_input ();
}


void
ns_set_horizontal_scroll_bar (struct window *window,
			      int portion, int whole, int position)
/* --------------------------------------------------------------------------
      External (hook): Update or add scrollbar.
   -------------------------------------------------------------------------- */
{
  Lisp_Object win;
  NSRect r, v;
  struct frame *f = XFRAME (WINDOW_FRAME (window));
  EmacsView *view = FRAME_NS_VIEW (f);
  EmacsScroller *bar;
  int top, height, left, width;
  int window_x, window_width;
  BOOL update_p = YES;

  /* Optimization; display engine sends WAY too many of these.  */
  if (!NILP (window->horizontal_scroll_bar))
    {
      bar = XNS_SCROLL_BAR (window->horizontal_scroll_bar);
      if ([bar checkSamePosition: position portion: portion whole: whole])
        {
          if (view->scrollbarsNeedingUpdate == 0)
            {
              if (!windows_or_buffers_changed)
                  return;
            }
          else
            view->scrollbarsNeedingUpdate--;
          update_p = NO;
        }
    }

  NSTRACE ("ns_set_horizontal_scroll_bar");

  /* Get dimensions.  */
  window_box (window, ANY_AREA, &window_x, 0, &window_width, 0);
  left = window_x;
  width = window_width;
  height = WINDOW_SCROLL_BAR_AREA_HEIGHT (window);
  top = WINDOW_SCROLL_BAR_AREA_Y (window);

  r = NSMakeRect (left, top, width, height);
  /* The parent view is flipped, so we need to flip y value.  */
  v = [view frame];
  r.origin.y = (v.size.height - r.size.height - r.origin.y);

  XSETWINDOW (win, window);
  block_input ();

  if (NILP (window->horizontal_scroll_bar))
    {
      if (width > 0 && height > 0)
	ns_clear_frame_area (f, left, top, width, height);

      bar = [[EmacsScroller alloc] initFrame: r window: win];
      wset_horizontal_scroll_bar (window, make_mint_ptr (bar));
      update_p = YES;
    }
  else
    {
      NSRect oldRect;
      bar = XNS_SCROLL_BAR (window->horizontal_scroll_bar);
      oldRect = [bar frame];
      if (FRAME_LIVE_P (f) && !NSEqualRects (oldRect, r))
        {
          ns_clear_frame_area (f, left, top, width, height);
          [bar setFrame: r];
          update_p = YES;
        }
    }

  /* If there are both horizontal and vertical scroll-bars they leave
     a square that belongs to neither. We need to clear it otherwise
     it fills with junk.  */
  if (!NILP (window->vertical_scroll_bar))
    ns_clear_frame_area (f, WINDOW_SCROLL_BAR_AREA_X (window), top,
                         WINDOW_SCROLL_BAR_AREA_WIDTH (window), height);

  if (update_p)
    [bar setPosition: position portion: portion whole: whole];
  unblock_input ();
}


void
ns_condemn_scroll_bars (struct frame *f)
/* --------------------------------------------------------------------------
     External (hook): arrange for all frame's scrollbars to be removed
     at next call to judge_scroll_bars, except for those redeemed.
   -------------------------------------------------------------------------- */
{
  int i;
  id view;
  NSArray *subviews = [[FRAME_NS_VIEW (f) superview] subviews];

  NSTRACE ("ns_condemn_scroll_bars");

  for (i =[subviews count]-1; i >= 0; i--)
    {
      view = [subviews objectAtIndex: i];
      if ([view isKindOfClass: [EmacsScroller class]])
        [view condemn];
    }
}


void
ns_redeem_scroll_bar (struct window *window)
/* --------------------------------------------------------------------------
     External (hook): arrange to spare this window's scrollbar
     at next call to judge_scroll_bars.
   -------------------------------------------------------------------------- */
{
  id bar;
  NSTRACE ("ns_redeem_scroll_bar");
  if (!NILP (window->vertical_scroll_bar)
      && WINDOW_HAS_VERTICAL_SCROLL_BAR (window))
    {
      bar = XNS_SCROLL_BAR (window->vertical_scroll_bar);
      [bar reprieve];
    }

  if (!NILP (window->horizontal_scroll_bar)
      && WINDOW_HAS_HORIZONTAL_SCROLL_BAR (window))
    {
      bar = XNS_SCROLL_BAR (window->horizontal_scroll_bar);
      [bar reprieve];
    }
}


void
ns_judge_scroll_bars (struct frame *f)
/* --------------------------------------------------------------------------
     External (hook): destroy all scrollbars on frame that weren't
     redeemed after call to condemn_scroll_bars.
   -------------------------------------------------------------------------- */
{
  int i;
  id view;
  EmacsView *eview = FRAME_NS_VIEW (f);
  NSArray *subviews = [[eview superview] subviews];

  NSTRACE ("ns_judge_scroll_bars");
  for (i = [subviews count]-1; i >= 0; --i)
    {
      view = [subviews objectAtIndex: i];
      if (![view isKindOfClass: [EmacsScroller class]]) continue;
      [view judge];
    }
}

/* ==========================================================================

    Image Hooks

   ========================================================================== */

void
ns_free_pixmap (struct frame *_f, Emacs_Pixmap pixmap)
{
  ns_release_object (pixmap);
}
