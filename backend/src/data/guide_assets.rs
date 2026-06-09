use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType as ResizeFilter;
use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage};

const MAX_DIMENSION: u32 = 1600;
const JPEG_QUALITY: u8 = 85;

#[derive(Debug, Clone)]
pub struct OptimizedGuideImage {
    pub bytes: Vec<u8>,
    pub filename: String,
}

fn extension(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn replace_extension(filename: &str, new_ext: &str) -> String {
    Path::new(filename)
        .file_stem()
        .map(|stem| format!("{}.{}", stem.to_string_lossy(), new_ext))
        .unwrap_or_else(|| format!("{}.{}", filename, new_ext))
}

fn resize_if_needed(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    if longest <= MAX_DIMENSION {
        return img;
    }
    let scale = MAX_DIMENSION as f32 / longest as f32;
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    img.resize(nw, nh, ResizeFilter::Lanczos3)
}

fn has_transparency(rgba: &RgbaImage) -> bool {
    rgba.pixels().any(|p| p[3] < 255)
}

fn encode_jpeg(img: &DynamicImage, out: &mut Vec<u8>) -> Result<(), String> {
    let mut enc = JpegEncoder::new_with_quality(out, JPEG_QUALITY);
    enc.encode_image(img)
        .map_err(|e| format!("encode jpeg: {}", e))
}

/// Resize and re-encode raster guide images. SVG and GIF are left to the caller.
pub fn optimize_guide_image(filename: &str, data: &[u8]) -> Result<OptimizedGuideImage, String> {
    let ext = extension(filename).ok_or_else(|| "missing extension".to_string())?;
    match ext.as_str() {
        "svg" | "gif" => return Err("not optimized".to_string()),
        "png" | "jpg" | "jpeg" | "webp" => {}
        _ => return Err("unsupported".to_string()),
    }

    let img = image::load_from_memory(data).map_err(|e| format!("decode: {}", e))?;
    let img = resize_if_needed(img);
    let mut out = Vec::new();
    let mut save_filename = filename.to_string();

    match ext.as_str() {
        "jpeg" | "jpg" => {
            encode_jpeg(&img, &mut out)?;
        }
        "png" => {
            let rgba = img.to_rgba8();
            if has_transparency(&rgba) {
                let (w, h) = rgba.dimensions();
                let enc = PngEncoder::new_with_quality(
                    &mut out,
                    CompressionType::Best,
                    FilterType::Adaptive,
                );
                enc.write_image(rgba.as_raw(), w, h, ExtendedColorType::Rgba8)
                    .map_err(|e| format!("encode png: {}", e))?;
            } else {
                encode_jpeg(&img, &mut out)?;
                save_filename = replace_extension(filename, "jpg");
            }
        }
        "webp" => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let enc = WebPEncoder::new_lossless(&mut out);
            enc.encode(rgba.as_raw(), w, h, ExtendedColorType::Rgba8)
                .map_err(|e| format!("encode webp: {}", e))?;
        }
        _ => unreachable!(),
    }

    Ok(OptimizedGuideImage {
        bytes: out,
        filename: save_filename,
    })
}
