use photon_ui::image::{
    delete_kitty_image,
    encode_iterm2,
    encode_kitty,
    get_gif_dimensions,
    get_png_dimensions,
};

#[test]
fn kitty_encode_produces_sequence() {
    let data = b"fake_image_data";
    let seq = encode_kitty(1, data, 8, 4);
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

#[test]
fn kitty_encode_uses_transmit_and_placement_actions() {
    let data = b"fake_image_data";
    let seq = encode_kitty(1, data, 8, 4);
    // Transmission uses lowercase 'a=t', responses are suppressed,
    // and the final chunk plus placement command display the image.
    assert!(seq.contains("a=t"));
    assert!(seq.contains("a=p"));
    assert!(!seq.contains("a=T"));
    assert!(seq.contains("m=0"));
    assert!(seq.contains("q=1"));
    assert!(seq.contains("c=8"));
    assert!(seq.contains("r=4"));
}

#[test]
fn kitty_encode_first_chunk_has_full_keys_continuation_does_not() {
    // Enough data to force at least one intermediate 4096-byte base64 chunk.
    let data = vec![0u8; 10000];
    let seq = encode_kitty(1, &data, 16, 8);
    let chunks: Vec<&str> = seq.split("\x1b\\").collect();
    // First chunk: full keys.
    assert!(chunks[0].contains("a=t"));
    assert!(chunks[0].contains("f=100"));
    assert!(chunks[0].contains("i=1"));
    // An intermediate continuation chunk must only carry q and m.
    let continuation = chunks
        .iter()
        .find(|c| c.starts_with("\x1b_Gq=1,m=1"))
        .expect("expected an intermediate continuation chunk");
    assert!(!continuation.contains("a=t"));
    assert!(!continuation.contains("f=100"));
    assert!(!continuation.contains("i=1"));
}

#[test]
fn kitty_encode_empty_data_returns_empty() {
    assert!(encode_kitty(1, &[], 1, 1).is_empty());
}
