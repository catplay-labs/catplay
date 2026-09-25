use catplay_hap::cipher::{HomeKitChaChaNonce, HomeKitCipherFast, HomeKitCipherRing};

// #[test]
// fn fast_cipher_matches_ring_and_roundtrips() {
fn main() {
    let key = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x10, 0x21, 0x32, 0x43, 0x54, 0x65,
        0x76, 0x87, 0x98, 0xa9, 0xba, 0xcb, 0xdc, 0xed, 0xfe, 0x0f,
    ];
    let aad = b"\x05\x00\x10\x00";
    let nonce = HomeKitChaChaNonce(0x0102_0304_0506_0708);
    let plaintext = b"encrypted payload".to_vec();

    let mut ring_cipher = HomeKitCipherRing::new(key);
    let mut fast_cipher = HomeKitCipherFast::new(key);

    let mut ring_data = plaintext.clone();
    let ring_tag = ring_cipher
        .encrypt(&mut ring_data, aad, HomeKitChaChaNonce(nonce.0))
        .unwrap();

    let mut fast_data = plaintext.clone();
    let fast_tag = fast_cipher
        .encrypt(&mut fast_data, aad, HomeKitChaChaNonce(nonce.0))
        .unwrap();

    assert_eq!(fast_data, ring_data);
    assert_eq!(fast_tag, ring_tag);

    assert_eq!(
        &fast_tag[..],
        &[207, 56, 180, 18, 14, 131, 182, 103, 141, 80, 239, 2, 69, 37, 136, 97]
    );
    assert_eq!(
        &fast_data[..],
        &[138, 151, 41, 159, 52, 33, 55, 35, 121, 155, 95, 55, 160, 243, 213, 147, 140]
    );

    let mut fast_combined = fast_data.clone();
    fast_combined.extend_from_slice(&fast_tag);
    let decrypted = fast_cipher
        .decrypt(&mut fast_combined, aad, HomeKitChaChaNonce(nonce.0))
        .unwrap();
    assert_eq!(decrypted, plaintext.as_slice());
    println!("Cipher OK");

    let key1 = HomeKitCipherRing::compute_key(&[0x42u8; 32], b"test", b"info");
    let key2 = HomeKitCipherFast::compute_key(&[0x42u8; 32], b"test", b"info");

    let expected_key = [
        30, 30, 48, 163, 19, 168, 18, 54, 148, 144, 123, 160, 78, 121, 251, 221, 231, 23, 183, 51, 31, 140, 34, 163, 62, 143, 146, 51, 146,
        252, 109, 236,
    ];
    assert_eq!(key1, expected_key, "invalid key by ring");
    assert_eq!(key2, expected_key, "invalid key by fast");
}
