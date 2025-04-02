#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syms_of_ftwrhb_font() {
    DEFSYM (Qftwrhb, "ftwrhb");
    Fput (Qftwr, Qfont_driver_superseded_by, Qftwrhb);
}
