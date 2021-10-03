mod utils;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{
    imageops, DynamicImage, GenericImageView, ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage,
};
use std::io::{Cursor, Write};
use wasm_bindgen::prelude::*;
use web_sys;
use zip::read::ZipFile;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

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
    process_idx: u32
}

#[wasm_bindgen]
impl ArchiveReader {
    pub fn new(buffer: Vec<u8>) -> ArchiveReader {
        let reader = Cursor::new(buffer);
        let zip = ZipArchive::new(reader).unwrap();

        ArchiveReader { zip, process_idx: 0 }
    }

    #[wasm_bindgen(js_name = getLength)]
    pub fn get_length(&self) -> JsValue {
        JsValue::from(self.zip.len())
    }

    #[wasm_bindgen(js_name = getProcessIdx)]
    pub fn get_process_idx(&self) -> JsValue {
        JsValue::from(self.process_idx)
    }

    #[wasm_bindgen(js_name = extractFilenames)]
    pub fn extract_filenames(&mut self) -> Vec<JsValue> {
        let zip = &mut self.zip;

        let mut filenames = Vec::new();

        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();
            self.process_idx = i as u32;

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
        std::io::copy(&mut file, &mut buffer).unwrap();

        buffer
    }
}

#[wasm_bindgen]
pub struct ArchiveWriter {
    zip: ZipWriter<Cursor<Vec<u8>>>,
}

#[wasm_bindgen]
impl ArchiveWriter {
    pub fn new() -> ArchiveWriter {
        let buffer = Cursor::new(Vec::new());
        let zip = ZipWriter::new(buffer);

        ArchiveWriter { zip }
    }

    #[wasm_bindgen(js_name = transformImages)]
    pub fn transform_images(&mut self, reader: &mut ArchiveReader) {
        utils::set_panic_hook();
        for i in 0..reader.zip.len() {
            let file = reader.zip.by_index(i).unwrap();

            if !file.is_file() {
                continue;
            }

            let filename = file.name().to_lowercase().to_owned();

            let img = self.resize_img(file);
            let img = ImageBuffer::from(img.into_rgba8());
            let img = self.convert(RgbaImage::from(img));

            let ((x, y), mut overlay) = self.create_overlay(&img.dimensions());
            imageops::overlay(&mut overlay, &img, x, y);

            let mut writer = Vec::new();
            let mut encoder = JpegEncoder::new(&mut writer);
            JpegEncoder::encode_image(&mut encoder, &overlay).unwrap();

            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            self.zip.start_file(filename, options).unwrap();
            self.zip.write_all(&*writer).unwrap();
        }
    }

    #[wasm_bindgen(js_name = extractBinary)]
    pub fn extract_binary(&mut self) -> Vec<u8> {
        let zip = &mut self.zip;
        let result = zip.finish().unwrap();

        result.into_inner()
    }
}

impl ArchiveWriter {
    fn create_overlay(&self, (height, width): &(u32, u32)) -> ((u32, u32), RgbImage) {
        let (x, y, size) = match height {
            x if x > width => (0, (height - width) / 2, *height),
            x if x < width => ((width - height) / 2, 0, *width),
            _ => (0, 0, *height),
        };

        let mut buffer: RgbImage = ImageBuffer::new(size, size);
        for to in buffer.pixels_mut() {
            *to = Rgb([255, 255, 255]);
        }

        ((x, y), buffer)
    }

    pub fn convert(&self, img: RgbaImage) -> RgbImage {
        let (width, height) = img.dimensions();
        let mut buffer: RgbImage = ImageBuffer::new(width, height);

        for (to, &Rgba([r, g, b, _])) in buffer.pixels_mut().zip(img.pixels()) {
            *to = Rgb([r, g, b]);
        }

        buffer
    }

    fn resize_img(&self, mut file: ZipFile) -> DynamicImage {
        let mut buffer = Vec::with_capacity(file.size() as usize);
        std::io::copy(&mut file, &mut buffer).unwrap();
        let img = image::load_from_memory(&*buffer).unwrap();

        let size = match img.height() {
            x if x > img.width() => img.height(),
            _ => img.width(),
        };

        let resized_img = img.resize(size, size, FilterType::Nearest);
        drop(file);

        resized_img
    }
}
