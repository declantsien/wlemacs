/* #if defined (HAVE_PGTK) */
/* typedef struct pgtk_output output; */
/* #elif defined (HAVE_WAYLAND_CLIENT) */
/* typedef struct wlc_output output; */
/* #endif */

/* #include TERM_HEADER */
/* #include "wr_ffi.h" */
#include "dispextern.h"
#include "font.h"

extern void
wr_row_clip_bounds (struct window *w, struct glyph_row *row,
		    enum glyph_row_area area, Emacs_Rectangle *rect);
/* extern void emacs_rust_init_syms(void); */
extern void wrgui_init(struct frame *f);
extern void syms_of_webrender(void);

/* #define FRAME_WR_DATA(f) (FRAME_OUTPUT_DATA (f)->wr_data) */

/* #define BLACK_PIX_DEFAULT(f) 0 */
/* #define WHITE_PIX_DEFAULT(f) 65535 */
