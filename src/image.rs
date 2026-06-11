use base64::{
    Engine as _,
    engine::general_purpose::STANDARD,
};

/// Encode image data for the Kitty graphics protocol.
///
/// The data is base64-encoded and chunked with `m=1` (more data follows)
/// delimiters. Each chunk is up to 4096 bytes of base64 text.
pub fn encode_kitty(id: u32, data: &[u8], _mime_type: &str) -> String {
    let b64 = STANDARD.encode(data);
    let mut seq = format!("\x1b_Ga=T,f=100,i={},m=1;", id);
    const CHUNK_SIZE: usize = 4096;
    for chunk in b64.as_bytes().chunks(CHUNK_SIZE) {
        if let Ok(s) = std::str::from_utf8(chunk) {
            seq.push_str(s);
        }
        seq.push_str("\x1b\\");
    }
    seq
}

/// Encode image data for the iTerm2 inline image protocol.
pub fn encode_iterm2(data: &[u8], _mime_type: &str) -> String {
    let b64 = STANDARD.encode(data);
    format!("\x1b]1337;File=inline=1:{}\x07", b64)
}

/// Generate a Kitty graphics protocol delete command for the given image id.
pub fn delete_kitty_image(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={}\x1b\\", id)
}

/// Parse width and height from PNG file header bytes.
///
/// Returns `None` if the data is too short or the PNG signature is missing.
pub fn get_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((width, height))
}

/// Parse width and height from JPEG SOF0 / SOF2 marker segments.
///
/// Searches for `0xFFC0` (baseline) or `0xFFC2` (progressive) markers.
pub fn get_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2;
    while i < data.len().saturating_sub(9) {
        if data[i] == 0xff && (data[i + 1] == 0xc0 || data[i + 1] == 0xc2) {
            let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            return Some((w, h));
        }
        i += 1;
    }
    None
}

/// Parse width and height from GIF logical screen descriptor.
pub fn get_gif_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 10 {
        return None;
    }
    let w = u16::from_le_bytes([data[6], data[7]]) as u32;
    let h = u16::from_le_bytes([data[8], data[9]]) as u32;
    Some((w, h))
}

/// Parse width and height from a VP8X WebP chunk.
///
/// Returns `None` if the RIFF/WEBP signatures are missing or the VP8X chunk
/// is not present.
pub fn get_webp_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 30 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return None;
    }
    if &data[12..16] == b"VP8X" {
        let w = u32::from_le_bytes([data[24], data[25], data[26], 0]) + 1;
        let h = u32::from_le_bytes([data[27], data[28], data[29], 0]) + 1;
        return Some((w, h));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_dimensions_valid() {
        let mut data = vec![0xff, 0xd8];
        data.extend_from_slice(&[0xff, 0xc0]);
        data.extend_from_slice(&[0x00, 0x0b]);
        data.extend_from_slice(&[0x08]);
        data.extend_from_slice(&[0x00, 0x10]);
        data.extend_from_slice(&[0x00, 0x20]);
        data.extend_from_slice(&[0x01, 0x01, 0x11, 0x00]);
        assert_eq!(get_jpeg_dimensions(&data), Some((32, 16)));
    }

    #[test]
    fn jpeg_dimensions_sof2() {
        let mut data = vec![0xff, 0xd8];
        data.extend_from_slice(&[0xff, 0xc2]);
        data.extend_from_slice(&[0x00, 0x0b]);
        data.extend_from_slice(&[0x08]);
        data.extend_from_slice(&[0x00, 0x20]);
        data.extend_from_slice(&[0x00, 0x10]);
        data.extend_from_slice(&[0x01, 0x01, 0x11, 0x00]);
        assert_eq!(get_jpeg_dimensions(&data), Some((16, 32)));
    }
}
