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
