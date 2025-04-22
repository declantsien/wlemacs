#include "config.h"

/* #include "limits.h" */

#include "lisp.h"

/* #include "atimer.h" */
#include "blockinput.h"
#include "buffer.h"
/* #include "category.h" */
/* #include "ccl.h" */
/* #include "character.h" */
/* #include "charset.h" */
/* #include "cm.h" */
/* #include "coding.h" */
/* #include "commands.h" */
#include "composite.h"
#include "dispextern.h"
/* #include "disptab.h" */
/* #include "dynlib.h" */
/* #include "emacs-icon.h" */
/* #include "emacs-module.h" */
/* #include "epaths.h" */
#include "font.h"
#include "xwidget.h"

/* #ifdef HAVE_FREETYPE */
/* #include <fontconfig/fontconfig.h> */
/* #include "ftfont.h" */
/* #endif */
#ifdef NS_IMPL_COCOA
/* #include "macfont.h" */
typedef const struct _EmacsScreenFont *ScreenFontRef; /* opaque */
typedef const struct _CGFont *CGFontRef; /* opaque */
typedef const struct _CTFont *CTFontRef; /* opaque */

/* The actual structure for Mac font that can be cast to struct font.  */

struct macfont_info
{
  struct font font;
  CTFontRef macfont;
  CGFontRef cgfont;
  ScreenFontRef screen_font;
  struct macfont_cache *cache;
  struct macfont_metrics **metrics;
  short metrics_nrows;
  bool_bf synthetic_italic_p : 1;
  bool_bf synthetic_bold_p : 1;
  unsigned spacing : 2;
  unsigned antialias : 2;
  bool_bf color_bitmap_p : 1;
  struct frame *f;
  WrFontKey key;
  WrFontInstanceKey instance_key;
};
#endif
#ifdef NS_IMPL_GNUSTEP
#include "nsfont.h"
#endif
/* #include "fontset.h" */
/* #include "frame.h" */
/* #include "getpagesize.h" */
/* //#include "globals.h" */
/* #include "gnutls.h" */
/* #include "indent.h" */
/* #include "intervals.h" */
/* #include "keyboard.h" */
/* #include "keymap.h" */
/* #include "macros.h" */
/* #include "macuvs.h" */
/* #include "menu.h" */
/* #include "nsterm.h" */
/* #include "process.h" */
/* /\* #include "regex.h" *\/ */
/* #include "region-cache.h" */
/* #include "syntax.h" */
/* #include "sysselect.h" */
/* #include "syssignal.h" */
/* #include "sysstdio.h" */
/* #include "systhread.h" */
/* #include "systime.h" */
/* #include "systty.h" */
/* #include "syswait.h" */
/* #include "termchar.h" */
/* #include "termhooks.h" */
/* #include "termopts.h" */
/* #include "thread.h" */
/* /\* #include "tparam.h" *\/ */
/* #ifdef HAVE_X_WINDOWS */
/* # include "widget.h" */
/* # include "widgetprv.h" */
/* # include "xsettings.h" */
/* #endif */
/* #include "window.h" */
/* #include "xgselect.h" */
/* #ifdef HAVE_PGTK */
/* # include "pgtkterm.h" */
/* # include "gtkutil.h" */
/* #endif */
/* #ifdef HAVE_WAYLAND_CLIENT */
/* # include "wlcterm.h" */
/* #endif */
#include TERM_HEADER
