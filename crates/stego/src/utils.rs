use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};

/// Generate a default cover image (gradient pattern)
pub fn generate_cover_image(width: u32, height: u32) -> DynamicImage {
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let r = ((x * 255) / width) as u8;
        let g = ((y * 255) / height) as u8;
        let b = (((x + y) * 255) / (width + height)) as u8;
        Rgb([r, g, b])
    });
    DynamicImage::ImageRgb8(img)
}

/// Detect MIME type from magic bytes
pub fn get_mime_type(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "application/octet-stream";
    }

    // PNG
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return "image/png";
    }

    // JPEG
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "image/jpeg";
    }

    // GIF
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return "image/gif";
    }

    // WebP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return "image/webp";
    }

    // BMP
    if data.starts_with(b"BM") {
        return "image/bmp";
    }

    "application/octet-stream"
}

/// Generate a synthetic dataset image (for stress testing)
pub fn generate_dataset_image(index: usize, base_width: u32, base_height: u32) -> DynamicImage {
    let seed = index as u32;
    let width = base_width + (seed % 200);
    let height = base_height + ((seed * 7) % 200);

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let r = ((x * seed) % 256) as u8;
        let g = ((y * seed * 3) % 256) as u8;
        let b = (((x + y) * seed * 7) % 256) as u8;
        Rgb([r, g, b])
    });
    DynamicImage::ImageRgb8(img)
}
