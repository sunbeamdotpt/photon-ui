use base64::{
    Engine as _,
    engine::general_purpose::STANDARD,
};

/// Supported terminal image protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageProtocol {
    /// Kitty graphics protocol.
    #[default]
    Kitty,
    /// iTerm2 inline image protocol.
    Iterm2,
}

/// Encode image data for the Kitty graphics protocol.
///
/// The payload is treated as a PNG image (`f=100`), base64-encoded and
/// transmitted in chunks. `q=1` suppresses terminal responses, `m=0` marks
/// the final chunk, and a trailing `a=p` command places the image by id.
///
/// `cols` and `rows` tell the terminal how many cells the image should occupy
/// on screen. The image is scaled to fit that cell rectangle, which lets the
/// layout engine reserve the matching amount of space.
///
/// Chunking follows the spec: only the first chunk carries the full set of
/// keys (`a`, `f`, `i`, `q`, `m`); continuation chunks only carry `q` and `m`.
pub fn encode_kitty(id: u32, data: &[u8], cols: u16, rows: u16) -> String {
    let b64 = STANDARD.encode(data);
    if b64.is_empty() {
        return String::new();
    }
    let mut seq = String::new();
    const CHUNK_SIZE: usize = 4096;
    let chunks: Vec<&[u8]> = b64.as_bytes().chunks(CHUNK_SIZE).collect();
    let last = chunks.len() - 1;
    for (i, chunk) in chunks.iter().enumerate() {
        let more = if i == last { 0 } else { 1 };
        if i == 0 {
            seq.push_str(&format!("\x1b_Ga=t,f=100,i={},q=1,m={};", id, more));
        } else {
            seq.push_str(&format!("\x1b_Gq=1,m={};", more));
        }
        if let Ok(s) = std::str::from_utf8(chunk) {
            seq.push_str(s);
        }
        seq.push_str("\x1b\\");
    }
    seq.push_str(&format!(
        "\x1b_Ga=p,i={},c={},r={},q=1\x1b\\",
        id, cols, rows
    ));
    seq
}

/// Encode image data for the iTerm2 inline image protocol.
pub fn encode_iterm2(data: &[u8], _mime_type: &str) -> String {
    let b64 = STANDARD.encode(data);
    format!("\x1b]1337;File=inline=1:{}\x07", b64)
}

/// Generate a Kitty graphics protocol delete command for the given image id.
pub fn delete_kitty_image(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={},q=1\x1b\\", id)
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
    fn image_protocol_default_is_kitty() {
        assert_eq!(ImageProtocol::default(), ImageProtocol::Kitty);
    }

    #[test]
    fn image_protocol_traits() {
        let proto = ImageProtocol::Kitty;
        let cloned = proto;
        assert_eq!(proto, cloned);
        assert_ne!(ImageProtocol::Kitty, ImageProtocol::Iterm2);
    }

    #[test]
    fn kitty_encode_empty_data_returns_empty() {
        assert!(encode_kitty(1, &[], 1, 1).is_empty());
    }

    #[test]
    fn kitty_encode_single_chunk() {
        let seq = encode_kitty(7, b"png", 10, 5);
        assert!(seq.starts_with("\x1b_Ga=t,f=100,i=7,q=1,m=0;"));
        assert!(seq.contains("a=p,i=7,c=10,r=5,q=1\x1b\\"));
        assert!(seq.contains("cG5n"));
    }

    #[test]
    fn kitty_encode_multiple_chunks_only_first_has_full_keys() {
        let data = vec![0u8; 10000];
        let seq = encode_kitty(3, &data, 20, 10);
        let parts: Vec<&str> = seq.split("\x1b\\").collect();
        // parts ends with an empty string after the final terminator.
        assert!(parts[0].starts_with("\x1b_Ga=t,f=100,i=3,q=1,m=1;"));
        let continuation = parts
            .iter()
            .find(|p| p.starts_with("\x1b_Gq=1,m=1;"))
            .expect("expected an intermediate continuation chunk");
        assert!(!continuation.contains("a=t"));
        assert!(!continuation.contains("f=100"));
        assert!(!continuation.contains("i=3"));
        assert!(seq.contains("\x1b_Ga=p,i=3,c=20,r=10,q=1\x1b\\"));
    }

    #[test]
    fn iterm2_encode_produces_sequence() {
        let seq = encode_iterm2(b"data", "image/png");
        assert!(seq.starts_with("\x1b]1337;File=inline=1:"));
        assert!(seq.ends_with("\x07"));
    }

    #[test]
    fn iterm2_encode_empty_data() {
        let seq = encode_iterm2(&[], "image/png");
        assert_eq!(seq, "\x1b]1337;File=inline=1:\x07");
    }

    #[test]
    fn delete_kitty_image_produces_sequence() {
        let seq = delete_kitty_image(42);
        assert_eq!(seq, "\x1b_Ga=d,d=I,i=42,q=1\x1b\\");
    }

    #[test]
    fn png_dimensions_valid() {
        let mut data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&100u32.to_be_bytes());
        data.extend_from_slice(&200u32.to_be_bytes());
        assert_eq!(get_png_dimensions(&data), Some((100, 200)));
    }

    #[test]
    fn png_dimensions_too_short() {
        assert_eq!(get_png_dimensions(&[0u8; 10]), None);
    }

    #[test]
    fn png_dimensions_invalid_signature() {
        assert_eq!(get_png_dimensions(b"NOTPNG"), None);
    }

    #[test]
    fn jpeg_dimensions_valid_baseline() {
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
    fn jpeg_dimensions_progressive() {
        let mut data = vec![0xff, 0xd8];
        data.extend_from_slice(&[0xff, 0xc2]);
        data.extend_from_slice(&[0x00, 0x0b]);
        data.extend_from_slice(&[0x08]);
        data.extend_from_slice(&[0x00, 0x20]);
        data.extend_from_slice(&[0x00, 0x10]);
        data.extend_from_slice(&[0x01, 0x01, 0x11, 0x00]);
        assert_eq!(get_jpeg_dimensions(&data), Some((16, 32)));
    }

    #[test]
    fn jpeg_dimensions_no_sof_marker() {
        assert_eq!(get_jpeg_dimensions(b"\xff\xd8\xff\xe0"), None);
    }

    #[test]
    fn gif_dimensions_valid() {
        let mut data = b"GIF89a".to_vec();
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&50u16.to_le_bytes());
        assert_eq!(get_gif_dimensions(&data), Some((100, 50)));
    }

    #[test]
    fn gif_dimensions_too_short() {
        assert_eq!(get_gif_dimensions(&[0u8; 5]), None);
    }

    #[test]
    fn webp_dimensions_vp8x() {
        let mut data = vec![0u8; 30];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        data[12..16].copy_from_slice(b"VP8X");
        data[24] = 99; // width - 1 = 99 -> width = 100
        data[27] = 49; // height - 1 = 49 -> height = 50
        assert_eq!(get_webp_dimensions(&data), Some((100, 50)));
    }

    #[test]
    fn webp_dimensions_too_short() {
        assert_eq!(get_webp_dimensions(&[0u8; 10]), None);
    }

    #[test]
    fn webp_dimensions_missing_riff() {
        let mut data = vec![0u8; 30];
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(get_webp_dimensions(&data), None);
    }

    #[test]
    fn webp_dimensions_missing_webp() {
        let mut data = vec![0u8; 30];
        data[0..4].copy_from_slice(b"RIFF");
        assert_eq!(get_webp_dimensions(&data), None);
    }

    #[test]
    fn webp_dimensions_missing_vp8x() {
        let mut data = vec![0u8; 30];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(get_webp_dimensions(&data), None);
    }
}
