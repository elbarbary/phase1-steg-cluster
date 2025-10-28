use crate::error::{Result, StegoError};
use crc32fast::Hasher;
use flate2::write::{DeflateEncoder, DeflateDecoder};
use flate2::Compression;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use std::io::Write;

const MAGIC: u32 = 0x53544547; // "STEG"

#[derive(Debug, Clone)]
pub struct CoverInfo {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub lsb_per_channel: u8,
    pub capacity_bytes: u64,
}

/// Embed secret bytes into a cover image using LSB steganography
pub fn embed(
    cover: &DynamicImage,
    secret_bytes: &[u8],
    lsb_per_channel: u8,
    compress: bool,
) -> Result<(DynamicImage, CoverInfo)> {
    let (width, height) = cover.dimensions();
    let channels = 3u8; // RGB only
    
    // Calculate capacity
    let total_pixels = width as u64 * height as u64;
    let bits_available = total_pixels * channels as u64 * lsb_per_channel as u64;
    let capacity_bytes = bits_available / 8;

    let cover_info = CoverInfo {
        width,
        height,
        channels,
        lsb_per_channel,
        capacity_bytes,
    };

    // Prepare payload
    let payload = if compress {
        compress_data(secret_bytes)?
    } else {
        secret_bytes.to_vec()
    };

    let payload_len = payload.len() as u32;
    let crc = compute_crc(&payload);

    // Build header: magic(4) + len(4) + crc(4) = 12 bytes
    let mut header = Vec::with_capacity(12 + payload.len());
    header.extend_from_slice(&MAGIC.to_be_bytes());
    header.extend_from_slice(&payload_len.to_be_bytes());
    header.extend_from_slice(&crc.to_be_bytes());
    header.extend_from_slice(&payload);

    let required_bits = header.len() as u64 * 8;
    if required_bits > bits_available {
        return Err(StegoError::CapacityExceeded {
            needed: required_bits.div_ceil(8),
            available: capacity_bytes,
        });
    }

    // Convert cover to RGB8
    let rgb_img = cover.to_rgb8();
    let mut stego = ImageBuffer::new(width, height);

    let mut bit_index = 0usize;
    let total_bits = header.len() * 8;

    for y in 0..height {
        for x in 0..width {
            if bit_index >= total_bits {
                // Copy remaining pixels unchanged
                let pixel = rgb_img.get_pixel(x, y);
                stego.put_pixel(x, y, *pixel);
                continue;
            }

            let pixel = rgb_img.get_pixel(x, y);
            let mut new_pixel = [pixel[0], pixel[1], pixel[2]];

            for _ in 0..3 {
                if bit_index >= total_bits {
                    break;
                }

                let byte_idx = bit_index / 8;
                let bit_offset = 7 - (bit_index % 8); // MSB first
                let bit = (header[byte_idx] >> bit_offset) & 1;

                // Clear LSB and set new bit
                new_pixel[bit_index % 3] = (new_pixel[bit_index % 3] & !1) | bit;
                bit_index += 1;
            }

            stego.put_pixel(x, y, Rgb(new_pixel));
        }
    }

    Ok((DynamicImage::ImageRgb8(stego), cover_info))
}

/// Extract secret bytes from a stego image
pub fn extract(
    stego: &DynamicImage,
    _lsb_per_channel: u8,
    compress: bool,
) -> Result<Vec<u8>> {
    let (width, height) = stego.dimensions();
    let rgb_img = stego.to_rgb8();

    // Extract header (12 bytes = 96 bits)
    let mut header_bits = Vec::new();
    let mut bit_count = 0usize;
    let header_bits_needed = 96;

    'outer: for y in 0..height {
        for x in 0..width {
            if bit_count >= header_bits_needed {
                break 'outer;
            }

            let pixel = rgb_img.get_pixel(x, y);
            for ch in 0..3 {
                if bit_count >= header_bits_needed {
                    break 'outer;
                }
                let bit = pixel[ch] & 1;
                header_bits.push(bit);
                bit_count += 1;
            }
        }
    }

    if header_bits.len() < header_bits_needed {
        return Err(StegoError::ExtractionFailed(
            "Image too small for header".to_string(),
        ));
    }

    // Reconstruct header bytes
    let header_bytes = bits_to_bytes(&header_bits[..96]);
    
    let magic = u32::from_be_bytes([header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3]]);
    if magic != MAGIC {
        return Err(StegoError::InvalidMagic(magic));
    }

    let payload_len = u32::from_be_bytes([header_bytes[4], header_bytes[5], header_bytes[6], header_bytes[7]]) as usize;
    let expected_crc = u32::from_be_bytes([header_bytes[8], header_bytes[9], header_bytes[10], header_bytes[11]]);

    // Extract payload
    let payload_bits_needed = payload_len * 8;
    let mut payload_bits = Vec::new();
    bit_count = 0;

    'outer2: for y in 0..height {
        for x in 0..width {
            let pixel = rgb_img.get_pixel(x, y);
            for ch in 0..3 {
                if bit_count < 96 {
                    // Skip header bits
                    bit_count += 1;
                    continue;
                }

                if payload_bits.len() >= payload_bits_needed {
                    break 'outer2;
                }

                let bit = pixel[ch] & 1;
                payload_bits.push(bit);
                bit_count += 1;
            }
        }
    }

    if payload_bits.len() < payload_bits_needed {
        return Err(StegoError::ExtractionFailed(
            format!("Not enough data: expected {} bits, got {}", payload_bits_needed, payload_bits.len()),
        ));
    }

    let payload = bits_to_bytes(&payload_bits);

    // Verify CRC
    let actual_crc = compute_crc(&payload);
    if actual_crc != expected_crc {
        return Err(StegoError::CrcMismatch {
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    // Decompress if needed
    let secret = if compress {
        decompress_data(&payload)?
    } else {
        payload
    };

    Ok(secret)
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).map_err(StegoError::Compression)?;
    encoder.finish().map_err(StegoError::Compression)
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(Vec::new());
    decoder.write_all(data).map_err(StegoError::Compression)?;
    decoder.finish().map_err(StegoError::Compression)
}

fn compute_crc(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << (7 - i);
            }
            byte
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn create_test_cover(width: u32, height: u32) -> DynamicImage {
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_round_trip() {
        let cover = create_test_cover(100, 100);
        let secret = b"Hello, steganography world!";

        let (stego, _info) = embed(&cover, secret, 1, false).unwrap();
        let recovered = extract(&stego, 1, false).unwrap();

        assert_eq!(secret.as_slice(), recovered.as_slice());
    }

    #[test]
    fn test_round_trip_compressed() {
        let cover = create_test_cover(200, 200);
        let secret = b"Compressed secret data that should survive round-trip!".repeat(10);

        let (stego, _info) = embed(&cover, &secret, 1, true).unwrap();
        let recovered = extract(&stego, 1, true).unwrap();

        assert_eq!(secret, recovered);
    }

    #[test]
    fn test_capacity_exceeded() {
        let cover = create_test_cover(10, 10); // Very small
        let secret = vec![0u8; 1000]; // Too large

        let result = embed(&cover, &secret, 1, false);
        assert!(matches!(result, Err(StegoError::CapacityExceeded { .. })));
    }

    #[test]
    fn test_invalid_magic() {
        let mut img = ImageBuffer::from_pixel(100, 100, Rgb([128, 128, 128]));
        // Embed wrong magic
        img.get_pixel_mut(0, 0)[0] = 0xFF;
        
        let stego = DynamicImage::ImageRgb8(img);
        let result = extract(&stego, 1, false);
        assert!(matches!(result, Err(StegoError::InvalidMagic(_))));
    }
}
