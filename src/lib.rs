mod utils;

use wasm_bindgen::prelude::*;
use zip::ZipArchive;
use std::io::Cursor;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct ZipReader {
    zip: ZipArchive<Cursor<Vec<u8>>>,
}

#[wasm_bindgen]
impl ZipReader {
    pub fn new(buffer: Vec<u8>) -> ZipReader {
        let reader = Cursor::new(buffer);
        let zip = ZipArchive::new(reader).unwrap();

        ZipReader { zip }
    }

    #[wasm_bindgen(js_name = extractFilenames)]
    pub fn extract_filenames(&mut self) -> Vec<JsValue> {
        let zip = &mut self.zip;

        let mut filenames = Vec::new();

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();

            if !file.is_file() {
                continue;
            }

            let name = file.name().to_owned();
            filenames.push(name);
        }

        filenames.iter().map(|x| JsValue::from_str(x)).collect()
    }

    #[wasm_bindgen(js_name = extractBinary)]
    pub fn extract_binary(&mut self, filename: &str) -> Vec<u8> {
        utils::set_panic_hook();

        let zip = &mut self.zip;

        let mut file = zip.by_name(filename).unwrap();
        let mut buffer = Vec::with_capacity(file.size() as usize);
        std::io::copy(&mut file, &mut buffer);

        buffer
    }
}