use photon_ui::image::{
    get_gif_dimensions,
    get_jpeg_dimensions,
    get_png_dimensions,
    get_webp_dimensions,
};

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
fn webp_invalid_too_short() {
    assert_eq!(get_webp_dimensions(&[0u8; 10]), None);
}

#[test]
fn webp_invalid_no_riff() {
    let mut data = vec![0u8; 30];
    data[8..12].copy_from_slice(b"WEBP");
    assert_eq!(get_webp_dimensions(&data), None);
}

#[test]
fn png_invalid_too_short() {
    assert_eq!(get_png_dimensions(&[0u8; 10]), None);
}

#[test]
fn png_invalid_header() {
    assert_eq!(get_png_dimensions(b"NOTPNG"), None);
}

#[test]
fn jpeg_no_marker() {
    assert_eq!(get_jpeg_dimensions(b"\xFF\xD8\xFF\xE0"), None);
}

#[test]
fn gif_too_short() {
    assert_eq!(get_gif_dimensions(&[0u8; 5]), None);
}
