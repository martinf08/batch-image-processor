mod utils;

use std::io;
use std::io::{Cursor, Write};
use wasm_bindgen::prelude::*;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};
use zip::read::ZipFile;
use zip::result::ZipResult;
use std::error::Error;
use image::{RgbaImage, ImageBuffer, GenericImageView, DynamicImage};
use image::io::Reader;
use image::ImageFormat::Jpeg;
use web_sys;
use std::process::exit;
use image::imageops::FilterType;


macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct ArchiveReader {
    zip: ZipArchive<Cursor<Vec<u8>>>,
}

#[wasm_bindgen]
impl ArchiveReader {
    pub fn new(buffer: Vec<u8>) -> ArchiveReader {
        let reader = Cursor::new(buffer);
        let zip = ZipArchive::new(reader).unwrap();

        ArchiveReader { zip }
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
        let zip = &mut self.zip;

        let mut file = zip.by_name(filename).unwrap();
        let mut buffer = Vec::with_capacity(file.size() as usize);
        std::io::copy(&mut file, &mut buffer);

        buffer
    }
}

#[wasm_bindgen]
pub struct ArchiveWriter {
    zip: ZipWriter<Cursor<Vec<u8>>>,
    options: FileOptions,
}

#[wasm_bindgen]
impl ArchiveWriter {
    pub fn new() -> ArchiveWriter {
        let buffer = Cursor::new(Vec::new());
        let zip = ZipWriter::new(buffer);
        let options = FileOptions::default();

        ArchiveWriter { zip, options }
    }

    #[wasm_bindgen(js_name = resizeAndRenameToLowerCase)]
    pub fn rename_to_uppercase_and_rotate90(&mut self, reader: &mut ArchiveReader) {
        utils::set_panic_hook();
        for i in 0..reader.zip.len() {
            let mut file = reader.zip.by_index(i).unwrap();

            if !file.is_file() {
                continue;
            }

            let mut buffer = Vec::with_capacity(file.size() as usize);
            std::io::copy(&mut file, &mut buffer);
            let img = image::load_from_memory(&*buffer).unwrap();

            log!("{:?}", img.dimensions());

            let img = (|| -> DynamicImage {
                if img.width() > img.height() {
                    return img.resize_to_fill(img.width(), img.width(), FilterType::Nearest);
                }

                return img.resize_to_fill(img.height(), img.height(), FilterType::Nearest);
            })();

            log!("{:?}", img.dimensions());

            let mut buffer = Vec::new();
            img.write_to(&mut buffer, Jpeg);

            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            let filename = file.name().to_lowercase().to_owned();

            drop(file);
            self.zip.start_file(filename, options).unwrap();
            self.zip.write_all(&*buffer).unwrap();
        }
    }

    #[wasm_bindgen(js_name = extractBinary)]
    pub fn extract_binary(&mut self) -> Vec<u8> {
        let mut zip = &mut self.zip;
        let result = zip.finish().unwrap();

        result.into_inner()
    }
}
