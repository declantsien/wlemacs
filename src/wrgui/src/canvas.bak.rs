pub struct WrCanvas {
    // save font key to font_info or not
    // if not, less emacs c changes. leave it as it os
    font_keys: FastHashMap<FontTemplate, FontKey>,
}

impl WrCanvas {
    pub fn resource_updates_add_font(
        &mut self,
        bytes: &mut WrVecU8,
        index: u32,
        is_native: bool,
    )  -> FontKey{
        // nsfont current don't have path info from font.fontDescriptor URL
        let font_tpl = if is_native {
            FontTemplate::Native(read_font_descriptor(data, index))
        } else {
            FontTemplate::Raw(bytes.flush_into_vec(), index)
        };

        if let Some(key) = self.font_keys.get(&font_tpl) {
            return *key;
        }
        // needs updates
        let font_key = self.render_api.generate_font_key();
        let mut txn = Transaction::new();
        match data {
            FontTemplate::Raw(ref bytes, index) => {
                txn.add_raw_font(font_key, bytes.to_vec(), index)
            }
            FontTemplate::Native(ref native_font) => {
                txn.add_native_font(font_key, native_font.clone())
            }
        }

        self.render_api.send_transaction(self.document_id, txn);

        self.fonts.insert(data, font_key);
}
