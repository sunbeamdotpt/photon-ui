use photon_ui::image::{
    delete_kitty_image,
    encode_iterm2,
    encode_kitty,
    get_gif_dimensions,
    get_jpeg_dimensions,
    get_png_dimensions,
};

#[test]
fn kitty_encode_produces_sequence() {
    let data = b"fake_image_data";
    let seq = encode_kitty(1, data, "image/png");
    assert!(seq.contains("_G"));
}

#[test]
fn iterm2_encode_produces_sequence() {
    let data = b"fake_image_data";
    let seq = encode_iterm2(data, "image/png");
    assert!(seq.contains("1337"));
}

#[test]
fn delete_kitty_produces_sequence() {
    let seq = delete_kitty_image(42);
    assert!(seq.contains("42"));
}

#[test]
fn png_dimensions_parse() {
    let mut data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    data.extend_from_slice(&[0; 8]);
    data.extend_from_slice(&100u32.to_be_bytes());
    data.extend_from_slice(&200u32.to_be_bytes());
    assert_eq!(get_png_dimensions(&data), Some((100, 200)));
}

#[test]
fn gif_dimensions_parse() {
    let mut data = b"GIF89a".to_vec();
    data.extend_from_slice(&100u16.to_le_bytes());
    data.extend_from_slice(&50u16.to_le_bytes());
    assert_eq!(get_gif_dimensions(&data), Some((100, 50)));
}
