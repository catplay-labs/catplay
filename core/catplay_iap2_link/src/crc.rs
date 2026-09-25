/// Accepts data but without the checksum byte
pub fn iap2_gen_checksum(data: &[u8]) -> u8 {
    let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    (!sum).wrapping_add(1)
}

fn sum_block_u32(data: &[u8]) -> u32 {
    let mut acc0 = 0u32;
    let mut acc1 = 0u32;
    let mut acc2 = 0u32;
    let mut acc3 = 0u32;

    let mut chunks = data.chunks_exact(16);
    for chunk in &mut chunks {
        acc0 += chunk[0] as u32 + chunk[4] as u32 + chunk[8] as u32 + chunk[12] as u32;
        acc1 += chunk[1] as u32 + chunk[5] as u32 + chunk[9] as u32 + chunk[13] as u32;
        acc2 += chunk[2] as u32 + chunk[6] as u32 + chunk[10] as u32 + chunk[14] as u32;
        acc3 += chunk[3] as u32 + chunk[7] as u32 + chunk[11] as u32 + chunk[15] as u32;
    }

    let mut sum = acc0 + acc1 + acc2 + acc3;
    for &b in chunks.remainder() {
        sum += b as u32;
    }

    sum
}

/// Same checksum as `iap2_gen_checksum`, but optimized for larger payloads by
/// summing independent fixed-size blocks first and then reducing block sums.
pub fn iap2_gen_checksum_fast(data: &[u8]) -> u8 {
    const BLOCK_SIZE: usize = 1024;

    let mut sum_mod_256 = 0u8;
    for block in data.chunks(BLOCK_SIZE) {
        sum_mod_256 = sum_mod_256.wrapping_add(sum_block_u32(block) as u8);
    }

    (!sum_mod_256).wrapping_add(1)
}

/// Accepts full data including checksum byte
pub fn iap2_check_checksum(data: &[u8]) -> bool {
    iap2_gen_checksum(data) == 0
}

/// Accepts full data including checksum byte
pub fn iap2_check_checksum_fast(data: &[u8]) -> bool {
    iap2_gen_checksum_fast(data) == 0
}

#[test]
fn fast_checksum_matches_baseline() {
    for len in [0usize, 1, 7, 8, 15, 16, 31, 32, 127, 128, 255, 256, 511, 512, 1023, 1024, 4096] {
        let payload: Vec<u8> = (0..len)
            .map(|i| (i as u8).wrapping_mul(37).wrapping_add(11))
            .collect();
        assert_eq!(
            iap2_gen_checksum(&payload),
            iap2_gen_checksum_fast(&payload),
            "mismatch for len={len}"
        );
    }
}
