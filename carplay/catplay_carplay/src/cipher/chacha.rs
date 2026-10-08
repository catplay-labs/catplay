use catplay_hap::cipher::HomeKitCipher;

pub enum AirPlayCipherSaltType {
    /// iPhone -> AirPlay server over RTSP
    Control,
    /// AirPlay server -> iPhone over RTSP
    Events,
    /// Audio or video frames
    DataStream { stream_connection_id: u64 },
}

fn compute_keys(shared_secret: &[u8; 32], t: AirPlayCipherSaltType) -> ([u8; 32], [u8; 32]) {
    let (keyword, read_keyword, write_keyword) = match t {
        AirPlayCipherSaltType::Control => ("Control", "Read", "Write"),
        AirPlayCipherSaltType::Events => ("Events", "Read", "Write"),
        AirPlayCipherSaltType::DataStream { .. } => ("DataStream", "Input", "Output"),
    };

    let salt = match t {
        AirPlayCipherSaltType::DataStream { stream_connection_id } => format!("{}-Salt{}", keyword, stream_connection_id),
        _ => format!("{}-Salt", keyword),
    };

    let read_key = HomeKitCipher::compute_key(
        shared_secret,
        salt.as_bytes(),
        format!("{}-{}-Encryption-Key", keyword, read_keyword).as_bytes(),
    );
    let write_key = HomeKitCipher::compute_key(
        shared_secret,
        salt.as_bytes(),
        format!("{}-{}-Encryption-Key", keyword, write_keyword).as_bytes(),
    );

    (read_key, write_key)
}

pub fn create_chacha_ciphers(shared_secret: &[u8; 32], t: AirPlayCipherSaltType, server: bool) -> (HomeKitCipher, HomeKitCipher) {
    create_chacha_ciphers_with(shared_secret, t, server, HomeKitCipher::new)
}

pub(crate) fn create_chacha_ciphers_with<C>(
    shared_secret: &[u8; 32],
    t: AirPlayCipherSaltType,
    server: bool,
    mut new: impl FnMut([u8; 32]) -> C,
) -> (C, C) {
    let (read_key, write_key) = compute_keys(shared_secret, t);

    let read_cipher = new(read_key);
    let write_cipher = new(write_key);

    match server {
        false => (read_cipher, write_cipher),
        true => (write_cipher, read_cipher),
    }
}
