use super::FairPlayError;

#[allow(dead_code)]
mod generated {
    include!("./generated.rs");
}

const INITIAL_SESSION_KEY: [u8; 16] = [
    0xDC, 0xDC, 0xF3, 0xB9, 0x0B, 0x74, 0xDC, 0xFB, 0x86, 0x7F, 0xF7, 0x60, 0x16, 0x72, 0x90, 0x51,
];

const INDEX_MANGLE: [u8; 11] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1B, 0x36, 0x6C];

const STATIC_SOURCE_1: [u8; 17] = [
    0xFA, 0x9C, 0xAD, 0x4D, 0x4B, 0x68, 0x26, 0x8C, 0x7F, 0xF3, 0x88, 0x99, 0xDE, 0x92, 0x2E, 0x95, 0x1E,
];

const STATIC_SOURCE_2: [u8; 47] = [
    0xEC, 0x4E, 0x27, 0x5E, 0xFD, 0xF2, 0xE8, 0x30, 0x97, 0xAE, 0x70, 0xFB, 0xE0, 0x00, 0x3F, 0x1C, 0x39, 0x80, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x09, 0x00, 0x0, 0x00,
    0x00, 0x00, 0x00,
];

const MD5_SHIFT: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4,
    11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const DEFAULT_SAP: [u8; 276] = [
    0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79,
    0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79, 0x79,
    0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79,
    0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x79, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x03, 0x02, 0x53, 0x00, 0x01, 0xcc, 0x34,
    0x2a, 0x5e, 0x5b, 0x1a, 0x67, 0x73, 0xc2, 0x0e, 0x21, 0xb8, 0x22, 0x4d, 0xf8, 0x62, 0x48, 0x18, 0x64, 0xef, 0x81, 0x0a, 0xae, 0x2e,
    0x37, 0x03, 0xc8, 0x81, 0x9c, 0x23, 0x53, 0x9d, 0xe5, 0xf5, 0xd7, 0x49, 0xbc, 0x5b, 0x7a, 0x26, 0x6c, 0x49, 0x62, 0x83, 0xce, 0x7f,
    0x03, 0x93, 0x7a, 0xe1, 0xf6, 0x16, 0xde, 0x0c, 0x15, 0xff, 0x33, 0x8c, 0xca, 0xff, 0xb0, 0x9e, 0xaa, 0xbb, 0xe4, 0x0f, 0x5d, 0x5f,
    0x55, 0x8f, 0xb9, 0x7f, 0x17, 0x31, 0xf8, 0xf7, 0xda, 0x60, 0xa0, 0xec, 0x65, 0x79, 0xc3, 0x3e, 0xa9, 0x83, 0x12, 0xc3, 0xb6, 0x71,
    0x35, 0xa6, 0x69, 0x4f, 0xf8, 0x23, 0x05, 0xd9, 0xba, 0x5c, 0x61, 0x5f, 0xa2, 0x54, 0xd2, 0xb1, 0x83, 0x45, 0x83, 0xce, 0xe4, 0x2d,
    0x44, 0x26, 0xc8, 0x35, 0xa7, 0xa5, 0xf6, 0xc8, 0x42, 0x1c, 0x0d, 0xa3, 0xf1, 0xc7, 0x00, 0x50, 0xf2, 0xe5, 0x17, 0xf8, 0xd0, 0xfa,
    0x77, 0x8d, 0xfb, 0x82, 0x8d, 0x40, 0xc7, 0x8e, 0x94, 0x1e, 0x1e, 0x1e,
];

#[inline]
fn f(b: u32, c: u32, d: u32) -> u32 {
    (b & c) | (!b & d)
}

#[inline]
fn g(b: u32, c: u32, d: u32) -> u32 {
    (b & d) | (c & !d)
}

#[inline]
fn h(b: u32, c: u32, d: u32) -> u32 {
    b ^ c ^ d
}

#[inline]
fn i_fn(b: u32, c: u32, d: u32) -> u32 {
    c ^ (b | !d)
}

#[inline]
fn md5_k(round: usize) -> u32 {
    let sin_term = ((1u64 << 32) as f64 * (round.wrapping_add(1) as f64).sin().abs()) as u64;
    sin_term as u32
}

#[inline]
fn ne_word(bytes: &[u8]) -> u32 {
    u32::from_ne_bytes(bytes.try_into().expect("4 bytes"))
}

#[inline]
fn set_ne_word(dst: &mut [u8], value: u32) {
    dst.copy_from_slice(&value.to_ne_bytes());
}

#[inline]
fn swap_words(words: &mut [u32; 16], a: usize, b: usize) {
    words.swap(a, b);
}

#[inline]
fn rol8(input: u8, count: u32) -> u8 {
    input.rotate_left(count & 7)
}

#[inline]
fn rol8x(input: u8, count: u32) -> u32 {
    (u32::from(input) << count) | (u32::from(input) >> (8 - count))
}

#[inline]
fn weird_ror8(input: u8, count: u32) -> u32 {
    if count == 0 {
        0
    } else {
        ((u32::from(input) >> count) & 0xff) | ((u32::from(input) & 0xff) << (8 - count))
    }
}

#[inline]
fn weird_rol8(input: u8, count: u32) -> u32 {
    if count == 0 {
        0
    } else {
        ((u32::from(input) << count) & 0xff) | ((u32::from(input) & 0xff) >> (8 - count))
    }
}

#[inline]
fn weird_rol32(input: u8, count: u32) -> u32 {
    if count == 0 {
        0
    } else {
        (u32::from(input) << count) ^ (u32::from(input) >> (8 - count))
    }
}

#[inline]
fn load_word_ne(bytes: &[u8], offset: usize) -> u32 {
    u32::from_ne_bytes(bytes[offset..offset + 4].try_into().expect("word"))
}

#[inline]
fn store_word_ne(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
}

#[inline]
fn xor_word_ne(dst: &mut [u8], dst_off: usize, src: &[u8], src_off: usize) {
    store_word_ne(dst, dst_off, load_word_ne(dst, dst_off) ^ load_word_ne(src, src_off));
}

#[inline]
fn table_index(i: usize) -> &'static [u8] {
    &generated::TABLE_S1[((31 * i) % 0x28) << 8..]
}

#[inline]
fn message_table_index(i: usize) -> &'static [u8] {
    &generated::TABLE_S2[((97 * i) % 144) << 8..]
}

#[inline]
fn permute_table_2(i: usize) -> &'static [u8] {
    &generated::TABLE_S4[((71 * i) % 144) << 8..]
}

#[inline]
fn xor_blocks(a: &[u8; 16], b: &[u8; 16], out: &mut [u8; 16]) {
    for i in 0..16 {
        out[i] = a[i] ^ b[i];
    }
}

#[inline]
fn z_xor_rust(input: &[u8], output: &mut [u8], blocks: usize) {
    for j in 0..blocks {
        for i in 0..16 {
            output[j * 16 + i] = input[j * 16 + i] ^ generated::Z_KEY[i];
        }
    }
}

#[inline]
fn x_xor_rust(input: &[u8], output: &mut [u8], blocks: usize) {
    for j in 0..blocks {
        for i in 0..16 {
            output[j * 16 + i] = input[j * 16 + i] ^ generated::X_KEY[i];
        }
    }
}

#[inline]
fn t_xor_rust(input: &[u8], output: &mut [u8]) {
    for i in 0..16 {
        output[i] = input[i] ^ generated::T_KEY[i];
    }
}

#[inline]
fn permute_block_1_rust(block: &mut [u8; 16]) {
    block[0] = generated::TABLE_S3[block[0] as usize];
    block[4] = generated::TABLE_S3[0x400 + block[4] as usize];
    block[8] = generated::TABLE_S3[0x800 + block[8] as usize];
    block[12] = generated::TABLE_S3[0xc00 + block[12] as usize];

    let mut tmp = block[13];
    block[13] = generated::TABLE_S3[0x100 + block[9] as usize];
    block[9] = generated::TABLE_S3[0xd00 + block[5] as usize];
    block[5] = generated::TABLE_S3[0x900 + block[1] as usize];
    block[1] = generated::TABLE_S3[0x500 + tmp as usize];

    tmp = block[2];
    block[2] = generated::TABLE_S3[0xa00 + block[10] as usize];
    block[10] = generated::TABLE_S3[0x200 + tmp as usize];
    tmp = block[6];
    block[6] = generated::TABLE_S3[0xe00 + block[14] as usize];
    block[14] = generated::TABLE_S3[0x600 + tmp as usize];

    tmp = block[3];
    block[3] = generated::TABLE_S3[0xf00 + block[7] as usize];
    block[7] = generated::TABLE_S3[0x300 + block[11] as usize];
    block[11] = generated::TABLE_S3[0x700 + block[15] as usize];
    block[15] = generated::TABLE_S3[0xb00 + tmp as usize];
}

#[inline]
fn permute_block_2_rust(block: &mut [u8; 16], round: usize) {
    block[0] = permute_table_2(round * 16)[block[0] as usize];
    block[4] = permute_table_2(round * 16 + 4)[block[4] as usize];
    block[8] = permute_table_2(round * 16 + 8)[block[8] as usize];
    block[12] = permute_table_2(round * 16 + 12)[block[12] as usize];

    let mut tmp = block[13];
    block[13] = permute_table_2(round * 16 + 13)[block[9] as usize];
    block[9] = permute_table_2(round * 16 + 9)[block[5] as usize];
    block[5] = permute_table_2(round * 16 + 5)[block[1] as usize];
    block[1] = permute_table_2(round * 16 + 1)[tmp as usize];

    tmp = block[2];
    block[2] = permute_table_2(round * 16 + 2)[block[10] as usize];
    block[10] = permute_table_2(round * 16 + 10)[tmp as usize];
    tmp = block[6];
    block[6] = permute_table_2(round * 16 + 6)[block[14] as usize];
    block[14] = permute_table_2(round * 16 + 14)[tmp as usize];

    tmp = block[3];
    block[3] = permute_table_2(round * 16 + 3)[block[7] as usize];
    block[7] = permute_table_2(round * 16 + 7)[block[11] as usize];
    block[11] = permute_table_2(round * 16 + 11)[block[15] as usize];
    block[15] = permute_table_2(round * 16 + 15)[tmp as usize];
}

fn generate_key_schedule_rust(key_material: &[u8; 16]) -> [[u32; 4]; 11] {
    let mut key_data = [0u8; 16];
    t_xor_rust(key_material, &mut key_data);

    let mut key_schedule = [[0u32; 4]; 11];
    let mut ti = 0usize;

    for round in 0..11usize {
        key_schedule[round][0] = load_word_ne(&key_data, 0);

        let table1 = table_index(ti);
        let table2 = table_index(ti + 1);
        let table3 = table_index(ti + 2);
        let table4 = table_index(ti + 3);
        ti += 4;

        key_data[0] ^= table1[key_data[13] as usize] ^ INDEX_MANGLE[round];
        key_data[1] ^= table2[key_data[14] as usize];
        key_data[2] ^= table3[key_data[15] as usize];
        key_data[3] ^= table4[key_data[12] as usize];

        key_schedule[round][1] = load_word_ne(&key_data, 4);
        let k0 = load_word_ne(&key_data, 0);
        xor_word_ne(&mut key_data, 4, &k0.to_ne_bytes(), 0);

        key_schedule[round][2] = load_word_ne(&key_data, 8);
        let k1 = load_word_ne(&key_data, 4);
        xor_word_ne(&mut key_data, 8, &k1.to_ne_bytes(), 0);

        key_schedule[round][3] = load_word_ne(&key_data, 12);
        let k2 = load_word_ne(&key_data, 8);
        xor_word_ne(&mut key_data, 12, &k2.to_ne_bytes(), 0);
    }

    key_schedule
}

#[allow(clippy::needless_range_loop)]
fn cycle_rust(block: &mut [u8; 16], key_schedule: &[[u32; 4]; 11]) {
    let mut b_words = [0u8; 16];
    b_words.copy_from_slice(block);

    for i in 0..4usize {
        let word = load_word_ne(&b_words, i * 4) ^ key_schedule[10][i];
        store_word_ne(&mut b_words, i * 4, word);
    }

    permute_block_1_rust(&mut b_words);

    for round in 0..9usize {
        let key = &key_schedule[9 - round];
        let key0 = key[0].to_ne_bytes();
        let key1 = key[1].to_ne_bytes();
        let key2 = key[2].to_ne_bytes();
        let key3 = key[3].to_ne_bytes();

        let ptr1 = generated::TABLE_S5[(b_words[3] ^ key0[3]) as usize];
        let ptr2 = generated::TABLE_S6[(b_words[2] ^ key0[2]) as usize];
        let ptr3 = generated::TABLE_S8[(b_words[0] ^ key0[0]) as usize];
        let ptr4 = generated::TABLE_S7[(b_words[1] ^ key0[1]) as usize];
        store_word_ne(&mut b_words, 0, ptr1 ^ ptr2 ^ ptr3 ^ ptr4);

        let ptr2 = generated::TABLE_S5[(b_words[7] ^ key1[3]) as usize];
        let ptr1 = generated::TABLE_S6[(b_words[6] ^ key1[2]) as usize];
        let ptr4 = generated::TABLE_S7[(b_words[5] ^ key1[1]) as usize];
        let ptr3 = generated::TABLE_S8[(b_words[4] ^ key1[0]) as usize];
        store_word_ne(&mut b_words, 4, ptr1 ^ ptr2 ^ ptr3 ^ ptr4);

        let w2 = generated::TABLE_S5[(b_words[11] ^ key2[3]) as usize]
            ^ generated::TABLE_S6[(b_words[10] ^ key2[2]) as usize]
            ^ generated::TABLE_S7[(b_words[9] ^ key2[1]) as usize]
            ^ generated::TABLE_S8[(b_words[8] ^ key2[0]) as usize];
        store_word_ne(&mut b_words, 8, w2);

        let w3 = generated::TABLE_S5[(b_words[15] ^ key3[3]) as usize]
            ^ generated::TABLE_S6[(b_words[14] ^ key3[2]) as usize]
            ^ generated::TABLE_S7[(b_words[13] ^ key3[1]) as usize]
            ^ generated::TABLE_S8[(b_words[12] ^ key3[0]) as usize];
        store_word_ne(&mut b_words, 12, w3);

        permute_block_2_rust(&mut b_words, 8 - round);
    }

    for i in 0..4usize {
        let word = load_word_ne(&b_words, i * 4) ^ key_schedule[0][i];
        store_word_ne(&mut b_words, i * 4, word);
    }

    block.copy_from_slice(&b_words);
}

fn decrypt_message_rust(message_in: &[u8; 164]) -> [u8; 128] {
    let mode = message_in[12] as usize;
    // let key_schedule = generate_key_schedule_rust(&INITIAL_SESSION_KEY);
    let mut decrypted_message = [0u8; 128];

    for i in 0..8usize {
        let mut buffer = [0u8; 16];
        let src_off = if mode == 3 {
            0x80usize.wrapping_sub(0x10 * i)
        } else {
            0x10 * (i + 1)
        };
        buffer.copy_from_slice(&message_in[src_off..src_off + 16]);

        for j in 0..9usize {
            let base = 0x80usize.wrapping_sub(0x10 * j);

            buffer[0x0] = message_table_index(base)[buffer[0x0] as usize] ^ generated::MESSAGE_KEY[mode][base];
            buffer[0x4] = message_table_index(base + 0x4)[buffer[0x4] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x4];
            buffer[0x8] = message_table_index(base + 0x8)[buffer[0x8] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x8];
            buffer[0xC] = message_table_index(base + 0xC)[buffer[0xC] as usize] ^ generated::MESSAGE_KEY[mode][base + 0xC];

            let tmp = buffer[0x0D];
            buffer[0x0D] = message_table_index(base + 0x0D)[buffer[0x09] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x0D];
            buffer[0x09] = message_table_index(base + 0x09)[buffer[0x05] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x09];
            buffer[0x05] = message_table_index(base + 0x05)[buffer[0x01] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x05];
            buffer[0x01] = message_table_index(base + 0x01)[tmp as usize] ^ generated::MESSAGE_KEY[mode][base + 0x01];

            let tmp = buffer[0x02];
            buffer[0x02] = message_table_index(base + 0x02)[buffer[0x0A] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x02];
            buffer[0x0A] = message_table_index(base + 0x0A)[tmp as usize] ^ generated::MESSAGE_KEY[mode][base + 0x0A];
            let tmp = buffer[0x06];
            buffer[0x06] = message_table_index(base + 0x06)[buffer[0x0E] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x06];
            buffer[0x0E] = message_table_index(base + 0x0E)[tmp as usize] ^ generated::MESSAGE_KEY[mode][base + 0x0E];

            let tmp = buffer[0x03];
            buffer[0x03] = message_table_index(base + 0x03)[buffer[0x07] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x03];
            buffer[0x07] = message_table_index(base + 0x07)[buffer[0x0B] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x07];
            buffer[0x0B] = message_table_index(base + 0x0B)[buffer[0x0F] as usize] ^ generated::MESSAGE_KEY[mode][base + 0x0B];
            buffer[0x0F] = message_table_index(base + 0x0F)[tmp as usize] ^ generated::MESSAGE_KEY[mode][base + 0x0F];

            let word0 = generated::TABLE_S9[buffer[0x0] as usize]
                ^ generated::TABLE_S9[0x100 + buffer[0x1] as usize]
                ^ generated::TABLE_S9[0x200 + buffer[0x2] as usize]
                ^ generated::TABLE_S9[0x300 + buffer[0x3] as usize];
            let word1 = generated::TABLE_S9[buffer[0x4] as usize]
                ^ generated::TABLE_S9[0x100 + buffer[0x5] as usize]
                ^ generated::TABLE_S9[0x200 + buffer[0x6] as usize]
                ^ generated::TABLE_S9[0x300 + buffer[0x7] as usize];
            let word2 = generated::TABLE_S9[buffer[0x8] as usize]
                ^ generated::TABLE_S9[0x100 + buffer[0x9] as usize]
                ^ generated::TABLE_S9[0x200 + buffer[0xA] as usize]
                ^ generated::TABLE_S9[0x300 + buffer[0xB] as usize];
            let word3 = generated::TABLE_S9[buffer[0xC] as usize]
                ^ generated::TABLE_S9[0x100 + buffer[0xD] as usize]
                ^ generated::TABLE_S9[0x200 + buffer[0xE] as usize]
                ^ generated::TABLE_S9[0x300 + buffer[0xF] as usize];

            let mut word_buf = [0u8; 16];
            store_word_ne(&mut word_buf, 0, word0);
            store_word_ne(&mut word_buf, 4, word1);
            store_word_ne(&mut word_buf, 8, word2);
            store_word_ne(&mut word_buf, 12, word3);
            buffer.copy_from_slice(&word_buf);
        }

        buffer[0x0] = generated::TABLE_S10[buffer[0x0] as usize];
        buffer[0x4] = generated::TABLE_S10[(0x4 << 8) + buffer[0x4] as usize];
        buffer[0x8] = generated::TABLE_S10[(0x8 << 8) + buffer[0x8] as usize];
        buffer[0xC] = generated::TABLE_S10[(0xC << 8) + buffer[0xC] as usize];

        let tmp = buffer[0x0D];
        buffer[0x0D] = generated::TABLE_S10[(0x0D << 8) + buffer[0x09] as usize];
        buffer[0x09] = generated::TABLE_S10[(0x09 << 8) + buffer[0x05] as usize];
        buffer[0x05] = generated::TABLE_S10[(0x05 << 8) + buffer[0x01] as usize];
        buffer[0x01] = generated::TABLE_S10[(0x01 << 8) + tmp as usize];

        let tmp = buffer[0x02];
        buffer[0x02] = generated::TABLE_S10[(0x02 << 8) + buffer[0x0A] as usize];
        buffer[0x0A] = generated::TABLE_S10[(0x0A << 8) + tmp as usize];
        let tmp = buffer[0x06];
        buffer[0x06] = generated::TABLE_S10[(0x06 << 8) + buffer[0x0E] as usize];
        buffer[0x0E] = generated::TABLE_S10[(0x0E << 8) + tmp as usize];

        let tmp = buffer[0x03];
        buffer[0x03] = generated::TABLE_S10[(0x03 << 8) + buffer[0x07] as usize];
        buffer[0x07] = generated::TABLE_S10[(0x07 << 8) + buffer[0x0B] as usize];
        buffer[0x0B] = generated::TABLE_S10[(0x0B << 8) + buffer[0x0F] as usize];
        buffer[0x0F] = generated::TABLE_S10[(0x0F << 8) + tmp as usize];

        if mode == 0 || mode == 1 || mode == 2 {
            let mut out = [0u8; 16];
            if i > 0 {
                xor_blocks(&buffer, array_ref_16(&message_in[0x10 * i..0x10 * i + 16]), &mut out);
            } else {
                xor_blocks(&buffer, &generated::MESSAGE_IV[mode], &mut out);
            }
            decrypted_message[0x10 * i..0x10 * i + 16].copy_from_slice(&out);
        } else {
            let dst_off = 0x70usize.wrapping_sub(0x10 * i);
            let mut out = [0u8; 16];
            if i < 7 {
                xor_blocks(&buffer, array_ref_16(&message_in[dst_off..dst_off + 16]), &mut out);
            } else {
                xor_blocks(&buffer, &generated::MESSAGE_IV[mode], &mut out);
            }
            decrypted_message[dst_off..dst_off + 16].copy_from_slice(&out);
        }
    }

    decrypted_message
}

fn generate_session_key(old_sap: &[u8], message_in: &[u8; 164]) -> [u8; 16] {
    let decrypted_message = decrypt_message_rust(message_in);
    let mut new_sap = [0u8; 320];
    new_sap[0x000..0x011].copy_from_slice(&STATIC_SOURCE_1);
    new_sap[0x011..0x091].copy_from_slice(&decrypted_message);
    new_sap[0x091..0x111].copy_from_slice(&old_sap[0x80..0x100]);
    new_sap[0x111..0x140].copy_from_slice(&STATIC_SOURCE_2);

    let mut session_key = INITIAL_SESSION_KEY;

    for round in 0..5usize {
        let base = &new_sap[round * 64..round * 64 + 64];
        let md5 = modified_md5_rust(base.try_into().expect("64 bytes"), &session_key);
        session_key = sap_hash_rust(base.try_into().expect("64 bytes"), &session_key);
        for i in 0..4usize {
            let sum = load_word_ne(&session_key, i * 4).wrapping_add(load_word_ne(&md5, i * 4));
            store_word_ne(&mut session_key, i * 4, sum);
        }
    }

    for i in (0..16usize).step_by(4) {
        session_key.swap(i, i + 3);
        session_key.swap(i + 1, i + 2);
    }
    for b in &mut session_key {
        *b ^= 121;
    }

    session_key
}

pub fn playfair_decrypt(message3: &[u8], input72: &[u8]) -> Result<[u8; 16], FairPlayError> {
    let message3: &[u8; 164] = message3
        .try_into()
        .map_err(|_| FairPlayError::UnsupportedSize(message3.len()))?;
    let input72: &[u8; 72] = input72
        .try_into()
        .map_err(|_| FairPlayError::UnsupportedSize(input72.len()))?;

    let sap_key = generate_session_key(&DEFAULT_SAP, message3);
    let mut block_in = [0u8; 16];
    let mut key_out = [0u8; 16];
    let key_schedule = generate_key_schedule_rust(&sap_key);

    z_xor_rust(&input72[56..72], &mut block_in, 1);
    cycle_rust(&mut block_in, &key_schedule);
    for i in 0..16usize {
        key_out[i] = block_in[i] ^ input72[16 + i];
    }

    let mut tmp = key_out;
    x_xor_rust(&tmp, &mut key_out, 1);
    tmp = key_out;
    z_xor_rust(&tmp, &mut key_out, 1);
    Ok(key_out)
}

#[inline]
fn array_ref_16(slice: &[u8]) -> &[u8; 16] {
    slice.try_into().expect("16 bytes")
}

#[allow(unused_mut)]
fn garble_rust(buffer0: &mut [u8; 20], buffer1: &mut [u8; 210], buffer2: &mut [u8; 35], buffer3: &mut [u8; 132], buffer4: &mut [u8; 21]) {
    let mut tmp: u32;
    let mut tmp2: u32;
    let mut tmp3: u32;
    let (
        mut a,
        mut b,
        mut c,
        mut d,
        mut e,
        mut m,
        mut j,
        mut g,
        mut f,
        mut h,
        mut k,
        mut r,
        mut s,
        mut t,
        mut u,
        mut v,
        mut w,
        mut x,
        mut y,
        mut z,
    );

    buffer2[12] = (0x14u32.wrapping_add(
        ((u32::from(buffer1[64]) & 92) | ((u32::from(buffer1[99]) / 3) & 35))
            & u32::from(buffer4[(rol8x(buffer4[usize::from(buffer1[206]) % 21], 4) as usize) % 21]),
    )) as u8;
    buffer1[4] = ((u32::from(buffer1[99]) / 5)
        .wrapping_mul(u32::from(buffer1[99]) / 5)
        .wrapping_mul(2)) as u8;
    buffer2[34] = 0xb8;
    buffer1[153] ^= (u32::from(buffer2[usize::from(buffer1[203]) % 35])
        .wrapping_mul(u32::from(buffer2[usize::from(buffer1[203]) % 35]))
        .wrapping_mul(u32::from(buffer1[190]))) as u8;
    buffer0[3] = buffer0[3].wrapping_sub((((u32::from(buffer4[usize::from(buffer1[205]) % 21]) >> 1) & 80) | 0x0e6440) as u8);
    buffer0[16] = 0x93;
    buffer0[13] = 0x62;
    buffer1[33] = buffer1[33].wrapping_sub((u32::from(buffer4[usize::from(buffer1[36]) % 21]) & 0xf6) as u8);
    tmp2 = u32::from(buffer2[usize::from(buffer1[67]) % 35]);
    buffer2[12] = 0x07;
    tmp = u32::from(buffer0[usize::from(buffer1[181]) % 20]);
    buffer1[2] = buffer1[2].wrapping_sub(3136u32 as u8);
    buffer0[19] = buffer4[usize::from(buffer1[58]) % 21];
    buffer3[0] = 92u8.wrapping_sub(buffer2[usize::from(buffer1[32]) % 35]);
    buffer3[4] = buffer2[usize::from(buffer1[15]) % 35].wrapping_add(0x9e);
    buffer1[34] = buffer1[34].wrapping_add((u32::from(buffer4[usize::from(buffer3[4]) % 21]) / 5) as u8);
    buffer0[19] =
        buffer0[19].wrapping_add((0xfffffee6u32.wrapping_sub((u32::from(buffer0[usize::from(buffer3[4]) % 20]) >> 1) & 102)) as u8);
    let rshift = u32::from(buffer4[usize::from(buffer1[190]) % 21]) & 7;
    let term = (u32::from(buffer1[72]) >> rshift)
        ^ (u32::from(buffer1[72]) << ((7u32.wrapping_sub(u32::from(buffer4[usize::from(buffer1[190]) % 21]).wrapping_sub(1))) & 7));
    let sub = 3u32.wrapping_mul(u32::from(buffer4[usize::from(buffer1[126]) % 21]));
    buffer1[15] = ((3u32.wrapping_mul(term.wrapping_sub(sub)) ^ u32::from(buffer1[15])) & 0xff) as u8;
    buffer0[15] ^= (u32::from(buffer2[usize::from(buffer1[181]) % 35])
        .wrapping_mul(u32::from(buffer2[usize::from(buffer1[181]) % 35]))
        .wrapping_mul(u32::from(buffer2[usize::from(buffer1[181]) % 35]))) as u8;
    buffer2[4] ^= (u32::from(buffer1[202]) / 3) as u8;
    a = 92u32.wrapping_sub(u32::from(buffer0[usize::from(buffer3[0]) % 20]));
    e = (a & 0xc6) | ((!u32::from(buffer1[105])) & 0xc6) | (a & !u32::from(buffer1[105]));
    buffer2[1] = buffer2[1].wrapping_add(e.wrapping_mul(e).wrapping_mul(e) as u8);
    buffer0[19] ^=
        (((224 | (u32::from(buffer4[usize::from(buffer1[92]) % 21]) & 27)) * u32::from(buffer2[usize::from(buffer1[41]) % 35])) / 3) as u8;
    buffer1[140] = buffer1[140].wrapping_add(weird_ror8(92, u32::from(buffer1[5]) & 7) as u8);
    buffer2[12] = buffer2[12].wrapping_add(
        ((((!u32::from(buffer1[4])) ^ u32::from(buffer2[usize::from(buffer1[12]) % 35])) | u32::from(buffer1[182])) & 192
            | (((!u32::from(buffer1[4])) ^ u32::from(buffer2[usize::from(buffer1[12]) % 35])) & u32::from(buffer1[182]))) as u8,
    );
    buffer1[36] = buffer1[36].wrapping_add(125);
    buffer1[124] = rol8(
        (((74 & u32::from(buffer1[138])) | ((74 | u32::from(buffer1[138])) & u32::from(buffer0[15])))
            & u32::from(buffer0[usize::from(buffer1[43]) % 20])
            | (((74 & u32::from(buffer1[138]))
                | ((74 | u32::from(buffer1[138])) & u32::from(buffer0[15]))
                | u32::from(buffer0[usize::from(buffer1[43]) % 20]))
                & 95)) as u8,
        4,
    );
    buffer3[8] =
        ((((u32::from(buffer0[usize::from(buffer3[4]) % 20]) & 95) & ((u32::from(buffer4[usize::from(buffer1[68]) % 21]) & 46) << 1)) | 16)
            ^ 92) as u8;
    a = u32::from(buffer1[177]).wrapping_add(u32::from(buffer4[usize::from(buffer1[79]) % 21]));
    d = (((a >> 1) | ((3 * u32::from(buffer1[148])) / 5)) & u32::from(buffer2[1])) | ((a >> 1) & ((3 * u32::from(buffer1[148])) / 5));
    buffer3[12] = (0u32.wrapping_sub(34).wrapping_sub(d)) as u8;
    a = 8u32.wrapping_sub(u32::from(buffer2[22]) & 7);
    b = u32::from(buffer1[33]) >> (a & 7);
    c = u32::from(buffer1[33]) << (u32::from(buffer2[22]) & 7);
    buffer2[16] = buffer2[16].wrapping_add(
        (((u32::from(buffer2[usize::from(buffer3[0]) % 35]) & 159) | u32::from(buffer0[usize::from(buffer3[4]) % 20]) | 8)
            .wrapping_sub((b ^ c) | 128)) as u8,
    );
    buffer0[14] ^= buffer2[usize::from(buffer3[12]) % 35];
    a = weird_rol8(
        buffer4[usize::from(buffer0[usize::from(buffer1[201]) % 20]) % 21],
        (u32::from(buffer2[usize::from(buffer1[112]) % 35]) << 1) & 7,
    );
    d = (u32::from(buffer0[usize::from(buffer1[208]) % 20]) & 131) | (u32::from(buffer0[usize::from(buffer1[164]) % 20]) & 124);
    buffer1[19] = buffer1[19].wrapping_add(((a & (d / 5)) | ((a | (d / 5)) & 37)) as u8);
    buffer2[8] = weird_ror8(
        140,
        (u32::from(buffer4[usize::from(buffer1[45]) % 21]).wrapping_add(92)
            * u32::from(buffer4[usize::from(buffer1[45]) % 21]).wrapping_add(92))
            & 7,
    ) as u8;
    buffer1[190] = 56;
    buffer2[8] ^= buffer3[0];
    buffer1[53] = (!((u32::from(buffer0[usize::from(buffer1[83]) % 20]) | 204) / 5)) as u8;
    buffer0[13] = buffer0[13].wrapping_add(buffer0[usize::from(buffer1[41]) % 20]);
    buffer0[10] = (((u32::from(buffer2[usize::from(buffer3[0]) % 35]) & u32::from(buffer1[2]))
        | ((u32::from(buffer2[usize::from(buffer3[0]) % 35]) | u32::from(buffer1[2])) & u32::from(buffer3[12])))
        / 15) as u8;
    a = (((56 | (u32::from(buffer4[usize::from(buffer1[2]) % 21]) & 68)) | u32::from(buffer2[usize::from(buffer3[8]) % 35])) & 42)
        | (((u32::from(buffer4[usize::from(buffer1[2]) % 21]) & 68) | 56) & u32::from(buffer2[usize::from(buffer3[8]) % 35]));
    buffer3[16] = a.wrapping_mul(a).wrapping_add(110) as u8;
    buffer3[20] = 202u8.wrapping_sub(buffer3[16]);
    buffer3[24] = buffer1[151];
    buffer2[13] ^= buffer4[usize::from(buffer3[0]) % 21];
    b = ((u32::from(buffer2[usize::from(buffer1[179]) % 35]).wrapping_sub(38)) & 177) | (u32::from(buffer3[12]) & 177);
    c = (u32::from(buffer2[usize::from(buffer1[179]) % 35]).wrapping_sub(38)) & u32::from(buffer3[12]);
    buffer3[28] = 30u32.wrapping_add((b | c).wrapping_mul(b | c)) as u8;
    buffer3[32] = buffer3[28].wrapping_add(62);
    a = ((u32::from(buffer3[20]).wrapping_add(u32::from(buffer3[0]) & 74)) | !u32::from(buffer4[usize::from(buffer3[0]) % 21])) & 121;
    b = (u32::from(buffer3[20]).wrapping_add(u32::from(buffer3[0]) & 74)) & !u32::from(buffer4[usize::from(buffer3[0]) % 21]);
    tmp3 = a | b;
    c = ((((a | b) ^ 0xffffffa6) | u32::from(buffer3[0])) & 4) | (((a | b) ^ 0xffffffa6) & u32::from(buffer3[0]));
    buffer1[47] ^= (u32::from(buffer2[usize::from(buffer1[89]) % 35]).wrapping_add(c)) as u8;
    buffer3[36] = ((rol8(((tmp & 179).wrapping_add(68)) as u8, 2) & buffer0[3]) | (tmp2 as u8 & !buffer0[3])).wrapping_sub(15);
    buffer1[123] ^= 221;
    a = (u32::from(buffer4[usize::from(buffer3[0]) % 21]) / 3).wrapping_sub(u32::from(buffer2[usize::from(buffer3[4]) % 35]));
    c = (((u32::from(buffer3[0]) & 163).wrapping_add(92)) & 246) | (u32::from(buffer3[0]) & 92);
    e = ((c | u32::from(buffer3[24])) & 54) | (c & u32::from(buffer3[24]));
    buffer3[40] = a.wrapping_sub(e) as u8;
    buffer3[44] = (tmp3 ^ 81 ^ (((u32::from(buffer3[0]) >> 1) & 101).wrapping_add(26))) as u8;
    buffer3[48] = (u32::from(buffer2[usize::from(buffer3[4]) % 35]) & 27) as u8;
    buffer3[52] = 27;
    buffer3[56] = 199;
    let b40 = u32::from(buffer3[40]);
    let b24 = u32::from(buffer3[24]);
    let b4_20 = u32::from(buffer4[usize::from(buffer3[0]) % 20]);
    let b4_21 = u32::from(buffer4[usize::from(buffer3[0]) % 21]);
    let p = ((b40 | b24) & 177) | (b40 & b24);
    let q = (b4_20 & 177) | 176 | (b4_21 & !3);
    let r64 = ((b40 & b24) | ((b40 | b24) & 177)) & 199;
    let s64 = (((b4_21 & 1).wrapping_add(176)) | (b4_21 & !3)) & u32::from(buffer3[56]);
    let t64 = ((p & q) | (r64 | s64)) & !u32::from(buffer3[52]);
    buffer3[64] = buffer3[4].wrapping_add((t64 | u32::from(buffer3[48])) as u8);
    buffer2[33] ^= buffer1[26];
    buffer1[106] ^= buffer3[20] ^ 133;
    buffer2[30] = ((((u32::from(buffer3[64]) / 3).wrapping_sub(275 | (u32::from(buffer3[0]) & 247)))
        ^ u32::from(buffer0[usize::from(buffer1[122]) % 20]))
        & 0xff) as u8;
    buffer1[22] = ((u32::from(buffer2[usize::from(buffer1[90]) % 35]) & 95) | 68) as u8;
    a = (u32::from(buffer4[usize::from(buffer3[36]) % 21]) & 184) | (u32::from(buffer2[usize::from(buffer3[44]) % 35]) & !184);
    buffer2[18] = buffer2[18].wrapping_add(((a.wrapping_mul(a).wrapping_mul(a)) >> 1) as u8);
    buffer2[5] = buffer2[5].wrapping_sub(buffer4[usize::from(buffer1[92]) % 21]);
    a = (((u32::from(buffer1[41]) & !24) | (u32::from(buffer2[usize::from(buffer1[183]) % 35]) & 24))
        & (u32::from(buffer3[16]).wrapping_add(53)))
        | (u32::from(buffer3[20]) & u32::from(buffer2[usize::from(buffer3[20]) % 35]));
    b = (u32::from(buffer1[17]) & !u32::from(buffer3[44])) | (u32::from(buffer0[usize::from(buffer1[59]) % 20]) & u32::from(buffer3[44]));
    buffer2[18] ^= a.wrapping_mul(b) as u8;
    a = weird_ror8(buffer1[11], u32::from(buffer2[usize::from(buffer1[28]) % 35]) & 7) & 7;
    b = (((u32::from(buffer0[usize::from(buffer1[93]) % 20]) & !u32::from(buffer0[14])) | (u32::from(buffer0[14]) & 150)) & !28)
        | (u32::from(buffer1[7]) & 28);
    buffer2[22] = ((((b | weird_rol8(buffer2[usize::from(buffer3[0]) % 35], a)) & u32::from(buffer2[33])
        | (b & weird_rol8(buffer2[usize::from(buffer3[0]) % 35], a)))
    .wrapping_add(74))
        & 0xff) as u8;
    a = u32::from(buffer4[(usize::from(buffer0[usize::from(buffer1[39]) % 20] ^ 217)) % 21]);
    buffer0[15] = buffer0[15].wrapping_sub(
        ((((u32::from(buffer3[20]) | u32::from(buffer3[0])) & 214) | (u32::from(buffer3[20]) & u32::from(buffer3[0]))) & a
            | ((((u32::from(buffer3[20]) | u32::from(buffer3[0])) & 214) | (u32::from(buffer3[20]) & u32::from(buffer3[0])) | a)
                & u32::from(buffer3[32]))) as u8,
    );
    let bc_lhs = (u32::from(buffer2[usize::from(buffer1[57]) % 35]) & u32::from(buffer0[usize::from(buffer3[64]) % 20]))
        | ((u32::from(buffer0[usize::from(buffer3[64]) % 20]) | u32::from(buffer2[usize::from(buffer1[57]) % 35])) & 95);
    b = (bc_lhs | (u32::from(buffer3[64]) & 45) | 82) & 32;
    c = bc_lhs & ((u32::from(buffer3[64]) & 45) | 82);
    d = ((u32::from(buffer3[0]) / 3).wrapping_sub(u32::from(buffer3[64]) | u32::from(buffer1[22])))
        ^ u32::from(buffer3[28]).wrapping_add(62)
        ^ (b | c);
    t = u32::from(buffer0[(d & 0xff) as usize % 20]);
    buffer3[68] =
        ((u32::from(buffer0[usize::from(buffer1[99]) % 20]).wrapping_pow(4)) | u32::from(buffer2[usize::from(buffer3[64]) % 35])) as u8;
    u = u32::from(buffer0[usize::from(buffer1[50]) % 20]);
    w = u32::from(buffer2[usize::from(buffer1[138]) % 35]);
    x = u32::from(buffer4[usize::from(buffer1[39]) % 21]);
    y = u32::from(buffer0[usize::from(buffer1[4]) % 20]);
    z = u32::from(buffer4[usize::from(buffer1[202]) % 21]);
    v = u32::from(buffer0[usize::from(buffer1[151]) % 20]);
    s = u32::from(buffer2[usize::from(buffer1[14]) % 35]);
    r = u32::from(buffer0[usize::from(buffer1[145]) % 20]);
    a = (u32::from(buffer2[usize::from(buffer3[68]) % 35]) & u32::from(buffer0[usize::from(buffer1[209]) % 20]))
        | ((u32::from(buffer2[usize::from(buffer3[68]) % 35]) | u32::from(buffer0[usize::from(buffer1[209]) % 20])) & 24);
    b = weird_rol8(
        buffer4[usize::from(buffer1[127]) % 21],
        u32::from(buffer2[usize::from(buffer3[68]) % 35]) & 7,
    );
    c = (a & u32::from(buffer0[10])) | (b & !u32::from(buffer0[10]));
    d = 7 ^ (u32::from(buffer4[usize::from(buffer2[usize::from(buffer3[36]) % 35]) % 21]) << 1);
    buffer3[72] = ((c & 71) | (d & !71)) as u8;
    buffer2[2] = buffer2[2].wrapping_add(
        ((((u32::from(buffer0[usize::from(buffer3[20]) % 20]) << 1) & 159) | (u32::from(buffer4[usize::from(buffer1[190]) % 21]) & !159))
            & ((((u32::from(buffer4[usize::from(buffer3[64]) % 21]) & 110) | (u32::from(buffer0[usize::from(buffer1[25]) % 20]) & !110))
                & !150)
                | (u32::from(buffer1[25]) & 150))) as u8,
    );
    buffer2[14] = buffer2[14].wrapping_sub(
        (((u32::from(buffer2[usize::from(buffer3[20]) % 35])
            & (u32::from(buffer3[72]) ^ u32::from(buffer2[usize::from(buffer1[100]) % 35])))
            & !34)
            | (u32::from(buffer1[97]) & 34)) as u8,
    );
    buffer0[17] = 115;
    let q = ((u32::from(buffer4[usize::from(buffer1[17]) % 21]) | u32::from(buffer0[usize::from(buffer3[20]) % 20]))
        & u32::from(buffer3[72]))
        | (u32::from(buffer4[usize::from(buffer1[17]) % 21]) & u32::from(buffer0[usize::from(buffer3[20]) % 20]));
    buffer1[23] ^= (((q & (u32::from(buffer1[50]) / 3)) | ((q | (u32::from(buffer1[50]) / 3)) & 246)) << 1) as u8;
    buffer0[13] = ((((((u32::from(buffer0[usize::from(buffer3[40]) % 20]) | u32::from(buffer1[10])) & 82)
        | (u32::from(buffer0[usize::from(buffer3[40]) % 20]) & u32::from(buffer1[10])))
        & 209)
        | ((u32::from(buffer0[usize::from(buffer1[39]) % 20]) << 1) & 46))
        >> 1) as u8;
    buffer2[33] = buffer2[33].wrapping_sub((u32::from(buffer1[113]) & 9) as u8);
    buffer2[28] = buffer2[28].wrapping_sub(((((2 | (u32::from(buffer1[110]) & 222)) >> 1) & !223) | (u32::from(buffer3[20]) & 223)) as u8);
    j = weird_rol8((v | z) as u8, u & 7);
    a = (u32::from(buffer2[16]) & t) | (w & !u32::from(buffer2[16]));
    b = (u32::from(buffer1[33]) & 17) | (x & !17);
    e = ((y | ((a.wrapping_add(b)) / 5)) & 147) | (y & ((a.wrapping_add(b)) / 5));
    m = (u32::from(buffer3[40]) & u32::from(buffer4[((u32::from(buffer3[8]).wrapping_add(j).wrapping_add(e)) & 0xff) as usize % 21]))
        | ((u32::from(buffer3[40]) | u32::from(buffer4[((u32::from(buffer3[8]).wrapping_add(j).wrapping_add(e)) & 0xff) as usize % 21]))
            & u32::from(buffer2[23]));
    buffer0[15] = (((u32::from(buffer4[usize::from(buffer3[20]) % 21]).wrapping_sub(48) & !u32::from(buffer1[184]))
        | (u32::from(buffer4[usize::from(buffer3[20]) % 21]).wrapping_sub(48) & 189)
        | (189 & !u32::from(buffer1[184])))
        & m.wrapping_mul(m).wrapping_mul(m)) as u8;
    buffer2[22] = buffer2[22].wrapping_add(buffer1[183]);
    buffer3[76] = (3u32.wrapping_mul(u32::from(buffer4[usize::from(buffer1[1]) % 21])) ^ u32::from(buffer3[0])) as u8;
    a = u32::from(buffer2[((u32::from(buffer3[8]).wrapping_add(j.wrapping_add(e))) & 0xff) as usize % 35]);
    f = ((u32::from(buffer4[usize::from(buffer1[178]) % 21]) & a) | ((u32::from(buffer4[usize::from(buffer1[178]) % 21]) | a) & 209))
        .wrapping_mul(u32::from(buffer0[usize::from(buffer1[13]) % 20]))
        .wrapping_mul(u32::from(buffer4[usize::from(buffer1[26]) % 21]) >> 1);
    g = (f.wrapping_add(0x733ffff9))
        .wrapping_mul(198)
        .wrapping_sub(
            ((f.wrapping_add(0x733ffff9))
                .wrapping_mul(396)
                .wrapping_add(212))
                & 212,
        )
        .wrapping_add(85);
    buffer3[80] = (u32::from(buffer3[36])
        .wrapping_add(g ^ 148)
        .wrapping_add((g ^ 107) << 1)
        .wrapping_sub(127)) as u8;
    buffer3[84] =
        ((u32::from(buffer2[usize::from(buffer3[64]) % 35]) & 245) | (u32::from(buffer2[usize::from(buffer3[20]) % 35]) & 10)) as u8;
    a = u32::from(buffer0[usize::from(buffer3[68]) % 20]) | 81;
    buffer2[18] = buffer2[18].wrapping_sub(
        ((a.wrapping_mul(a).wrapping_mul(a) & !u32::from(buffer0[15])) | ((u32::from(buffer3[80]) / 15) & u32::from(buffer0[15]))) as u8,
    );
    buffer3[88] = (u32::from(buffer3[8])
        .wrapping_add(j)
        .wrapping_add(e)
        .wrapping_sub(u32::from(buffer0[usize::from(buffer1[160]) % 20]))
        .wrapping_add(
            u32::from(buffer4[usize::from(buffer0[((u32::from(buffer3[8]).wrapping_add(j).wrapping_add(e)) & 255) as usize % 20]) % 21])
                / 3,
        )) as u8;
    b = ((r ^ u32::from(buffer3[72])) & !198) | ((s.wrapping_mul(s)) & 198);
    f = (u32::from(buffer4[usize::from(buffer1[69]) % 21]) & u32::from(buffer1[172]))
        | ((u32::from(buffer4[usize::from(buffer1[69]) % 21]) | u32::from(buffer1[172]))
            & (u32::from(buffer3[12]).wrapping_sub(b).wrapping_add(77)));
    buffer0[16] = (147u32.wrapping_sub((u32::from(buffer3[72]) & (f & 251 | 1)) | (((f & 250) | u32::from(buffer3[72])) & 198))) as u8;
    c = (u32::from(buffer4[usize::from(buffer1[168]) % 21]) & u32::from(buffer0[usize::from(buffer1[29]) % 20]) & 7)
        | ((u32::from(buffer4[usize::from(buffer1[168]) % 21]) | u32::from(buffer0[usize::from(buffer1[29]) % 20])) & 6);
    f = (u32::from(buffer4[usize::from(buffer1[155]) % 21]) & u32::from(buffer1[105]))
        | ((u32::from(buffer4[usize::from(buffer1[155]) % 21]) | u32::from(buffer1[105])) & 141);
    buffer0[3] = buffer0[3].wrapping_sub(buffer4[(weird_rol32(f as u8, c) % 21) as usize]);
    buffer1[5] = (weird_ror8(buffer0[12], (u32::from(buffer0[usize::from(buffer1[61]) % 20]) / 5) & 7)
        ^ ((!u32::from(buffer2[usize::from(buffer3[84]) % 35])) / 5)) as u8;
    buffer1[198] = buffer1[198].wrapping_add(buffer1[3]);
    a = 162 | u32::from(buffer2[usize::from(buffer3[64]) % 35]);
    buffer1[164] = buffer1[164].wrapping_add(((a.wrapping_mul(a)) / 5) as u8);
    g = weird_ror8(139, u32::from(buffer3[80]) & 7);
    c = ((u32::from(buffer4[usize::from(buffer3[64]) % 21])
        .wrapping_mul(u32::from(buffer4[usize::from(buffer3[64]) % 21]))
        .wrapping_mul(u32::from(buffer4[usize::from(buffer3[64]) % 21])))
        & 95)
        | (u32::from(buffer0[usize::from(buffer3[40]) % 20]) & !95);
    buffer3[92] =
        ((g & 12) | (u32::from(buffer0[usize::from(buffer3[20]) % 20]) & 12) | (g & u32::from(buffer0[usize::from(buffer3[20]) % 20])) | c)
            as u8;
    buffer2[12] = buffer2[12]
        .wrapping_add((((u32::from(buffer1[103]) & 32) | (u32::from(buffer3[92]) & (u32::from(buffer1[103]) | 60)) | 16) / 3) as u8);
    buffer3[96] = buffer1[143];
    buffer3[100] = 27;
    buffer3[104] = (((u32::from(buffer3[40]) & !u32::from(buffer2[8])) | (u32::from(buffer1[35]) & u32::from(buffer2[8])))
        & u32::from(buffer3[64])
        ^ 119) as u8;
    buffer3[108] = (238
        & ((((u32::from(buffer3[40]) & !u32::from(buffer2[8])) | (u32::from(buffer1[35]) & u32::from(buffer2[8])))
            & u32::from(buffer3[64]))
            << 1)) as u8;
    buffer3[112] = (((!u32::from(buffer3[64])) & (u32::from(buffer3[84]) / 3)) ^ 49) as u8;
    buffer3[116] = (98 & (((!u32::from(buffer3[64])) & (u32::from(buffer3[84]) / 3)) << 1)) as u8;
    a = (u32::from(buffer1[35]) & u32::from(buffer2[8])) | (u32::from(buffer3[40]) & !u32::from(buffer2[8]));
    b = (a & u32::from(buffer3[64])) | ((u32::from(buffer3[84]) / 3) & !u32::from(buffer3[64]));
    buffer1[143] = buffer3[96].wrapping_sub(
        ((b & (86 + ((u32::from(buffer1[172]) & 64) >> 1)))
            | (((((u32::from(buffer1[172]) & 65) >> 1) ^ 86)
                | (((!u32::from(buffer3[64])) & (u32::from(buffer3[84]) / 3))
                    | (((u32::from(buffer3[40]) & !u32::from(buffer2[8])) | (u32::from(buffer1[35]) & u32::from(buffer2[8])))
                        & u32::from(buffer3[64]))))
                & u32::from(buffer3[100]))) as u8,
    );
    buffer2[29] = 162;
    a = ((u32::from(buffer4[usize::from(buffer3[88]) % 21]) & 160) | (u32::from(buffer0[usize::from(buffer1[125]) % 20]) & 95)) >> 1;
    b = u32::from(buffer2[usize::from(buffer1[149]) % 35]) ^ u32::from(buffer1[43]).wrapping_mul(u32::from(buffer1[43]));
    buffer0[15] = buffer0[15].wrapping_add(((b & a) | ((a | b) & 115)) as u8);
    buffer3[120] = buffer3[64].wrapping_sub(buffer0[usize::from(buffer3[40]) % 20]);
    buffer1[95] = buffer4[usize::from(buffer3[20]) % 21];
    a = weird_ror8(
        buffer2[usize::from(buffer3[80]) % 35],
        (u32::from(buffer2[usize::from(buffer1[17]) % 35])
            .wrapping_mul(u32::from(buffer2[usize::from(buffer1[17]) % 35]))
            .wrapping_mul(u32::from(buffer2[usize::from(buffer1[17]) % 35])))
            & 7,
    );
    buffer0[7] = buffer0[7].wrapping_sub(a.wrapping_mul(a) as u8);
    buffer2[8] = buffer2[8].wrapping_sub(buffer1[184]).wrapping_add(
        (u32::from(buffer4[usize::from(buffer1[202]) % 21])
            .wrapping_mul(u32::from(buffer4[usize::from(buffer1[202]) % 21]))
            .wrapping_mul(u32::from(buffer4[usize::from(buffer1[202]) % 21]))) as u8,
    );
    buffer0[16] = ((u32::from(buffer2[usize::from(buffer1[102]) % 35]) << 1) & 132) as u8;
    buffer3[124] = ((u32::from(buffer4[usize::from(buffer3[40]) % 21]) >> 1) ^ u32::from(buffer3[68])) as u8;
    buffer0[7] = buffer0[7].wrapping_sub(buffer0[usize::from(buffer1[191]) % 20].wrapping_sub(
        (((u32::from(buffer4[usize::from(buffer1[80]) % 21]) << 1) & !177)
            | (u32::from(buffer4[usize::from(buffer4[usize::from(buffer3[88]) % 21]) % 21]) & 177)) as u8,
    ));
    buffer0[6] = buffer0[usize::from(buffer1[119]) % 20];
    a = (u32::from(buffer4[usize::from(buffer1[190]) % 21]) & !209) | (u32::from(buffer1[118]) & 209);
    b = u32::from(buffer0[usize::from(buffer3[120]) % 20]).wrapping_mul(u32::from(buffer0[usize::from(buffer3[120]) % 20]));
    buffer0[12] = (u32::from(buffer0[usize::from(buffer3[84]) % 20])
        ^ (u32::from(buffer2[usize::from(buffer1[71]) % 35]).wrapping_add(u32::from(buffer2[usize::from(buffer1[15]) % 35]))))
        as u8
        & ((a & b) | ((a | b) & 27)) as u8;
    b = (u32::from(buffer1[32]) & u32::from(buffer2[usize::from(buffer3[88]) % 35]))
        | ((u32::from(buffer1[32]) | u32::from(buffer2[usize::from(buffer3[88]) % 35])) & 23);
    d = (u32::from(buffer4[usize::from(buffer1[57]) % 21]).wrapping_mul(231) & 169) | (b & 86);
    f = (((u32::from(buffer0[usize::from(buffer1[82]) % 20]) & !29) | (u32::from(buffer4[usize::from(buffer3[124]) % 21]) & 29)) & 190)
        | (u32::from(buffer4[((d / 5) % 21) as usize]) & !190);
    h = u32::from(buffer0[usize::from(buffer3[40]) % 20])
        .wrapping_mul(u32::from(buffer0[usize::from(buffer3[40]) % 20]))
        .wrapping_mul(u32::from(buffer0[usize::from(buffer3[40]) % 20]));
    k = (h & u32::from(buffer1[82])) | (h & 92) | (u32::from(buffer1[82]) & 92);
    buffer3[128] = (((f & k) | ((f | k) & 192)) ^ (d / 5)) as u8;
    buffer2[25] ^= (u32::from(buffer0[usize::from(buffer3[120]) % 20] << 1)
        .wrapping_mul(u32::from(buffer1[5]))
        .wrapping_sub(
            weird_rol8(buffer3[76], u32::from(buffer4[usize::from(buffer3[124]) % 21]) & 7) & u32::from(buffer3[20]).wrapping_add(110),
        )) as u8;
}

pub fn modified_md5_rust(block_in: &[u8; 64], key_in: &[u8; 16]) -> [u8; 16] {
    let mut block_words = [0u32; 16];
    for (idx, chunk) in block_in.chunks_exact(4).enumerate() {
        block_words[idx] = ne_word(chunk);
    }

    let key_words = [
        ne_word(&key_in[0..4]),
        ne_word(&key_in[4..8]),
        ne_word(&key_in[8..12]),
        ne_word(&key_in[12..16]),
    ];

    let mut a = key_words[0];
    let mut b = key_words[1];
    let mut c = key_words[2];
    let mut d = key_words[3];

    #[allow(clippy::needless_range_loop)]
    for round in 0..64usize {
        let j = if round < 16 {
            round
        } else if round < 32 {
            round.wrapping_mul(5).wrapping_add(1) % 16
        } else if round < 48 {
            round.wrapping_mul(3).wrapping_add(5) % 16
        } else {
            round.wrapping_mul(7) % 16
        };

        let input = u32::from_be_bytes(block_words[j].to_ne_bytes());
        let mut z = a.wrapping_add(input).wrapping_add(md5_k(round));
        z = if round < 16 {
            z.wrapping_add(f(b, c, d)).rotate_left(MD5_SHIFT[round])
        } else if round < 32 {
            z.wrapping_add(g(b, c, d)).rotate_left(MD5_SHIFT[round])
        } else if round < 48 {
            z.wrapping_add(h(b, c, d)).rotate_left(MD5_SHIFT[round])
        } else {
            z.wrapping_add(i_fn(b, c, d)).rotate_left(MD5_SHIFT[round])
        };
        z = z.wrapping_add(b);

        let tmp = d;
        d = c;
        c = b;
        b = z;
        a = tmp;

        if round == 31 {
            swap_words(&mut block_words, (a & 15) as usize, (b & 15) as usize);
            swap_words(&mut block_words, (c & 15) as usize, (d & 15) as usize);
            swap_words(&mut block_words, ((a >> 4) & 15) as usize, ((b >> 4) & 15) as usize);
            swap_words(&mut block_words, ((a >> 8) & 15) as usize, ((b >> 8) & 15) as usize);
            swap_words(&mut block_words, ((a >> 12) & 15) as usize, ((b >> 12) & 15) as usize);
        }
    }

    let out_words = [
        key_words[0].wrapping_add(a),
        key_words[1].wrapping_add(b),
        key_words[2].wrapping_add(c),
        key_words[3].wrapping_add(d),
    ];
    let mut out = [0u8; 16];
    set_ne_word(&mut out[0..4], out_words[0]);
    set_ne_word(&mut out[4..8], out_words[1]);
    set_ne_word(&mut out[8..12], out_words[2]);
    set_ne_word(&mut out[12..16], out_words[3]);
    out
}

pub fn sap_hash_rust(block_in: &[u8; 64], key_in_out: &[u8; 16]) -> [u8; 16] {
    let (mut buffer0, mut buffer1, mut buffer2, mut buffer3, mut buffer4) = sap_buffers_from_block(block_in);

    garble_rust(&mut buffer0, &mut buffer1, &mut buffer2, &mut buffer3, &mut buffer4);

    sap_finalize(key_in_out, &buffer0, &buffer1, &buffer2, &buffer3)
}

fn sap_finalize(_key_in_out: &[u8; 16], buffer0: &[u8; 20], buffer1: &[u8; 210], buffer2: &[u8; 35], buffer3: &[u8; 132]) -> [u8; 16] {
    let i0_index = [18usize, 22, 23, 0, 5, 19, 32, 31, 10, 21, 30];
    let mut key_out = [0u8; 16];
    key_out.fill(0xE1);

    for i in 0..11usize {
        key_out[i] = if i == 3 {
            0x3d
        } else {
            key_out[i].wrapping_add(buffer3[i0_index[i].wrapping_mul(4)])
        };
    }

    for i in 0..20usize {
        key_out[i % 16] ^= buffer0[i];
    }
    for i in 0..35usize {
        key_out[i % 16] ^= buffer2[i];
    }
    for i in 0..210usize {
        key_out[i % 16] ^= buffer1[i];
    }

    for _ in 0..16usize {
        for i in 0..16usize {
            let x = key_out[(i + 9) & 15];
            let y = key_out[i];
            let z = key_out[(i + 11) & 15];
            let w = key_out[(i + 15) & 15];
            key_out[i] = rol8(x, 1) ^ y ^ rol8(z, 6) ^ rol8(w, 5);
        }
    }

    key_out
}

#[allow(clippy::type_complexity)]
fn sap_buffers_from_block(block_in: &[u8; 64]) -> ([u8; 20], [u8; 210], [u8; 35], [u8; 132], [u8; 21]) {
    let buffer0 = [
        0x96, 0x5F, 0xC6, 0x53, 0xF8, 0x46, 0xCC, 0x18, 0xDF, 0xBE, 0xB2, 0xF8, 0x38, 0xD7, 0xEC, 0x22, 0x03, 0xD1, 0x20, 0x8F,
    ];
    let mut buffer1 = [0u8; 210];
    let buffer2 = [
        0x43, 0x54, 0x62, 0x7A, 0x18, 0xC3, 0xD6, 0xB3, 0x9A, 0x56, 0xF6, 0x1C, 0x14, 0x3F, 0x0C, 0x1D, 0x3B, 0x36, 0x83, 0xB1, 0x39, 0x51,
        0x4A, 0xAA, 0x09, 0x3E, 0xFE, 0x44, 0xAF, 0xDE, 0xC3, 0x20, 0x9D, 0x42, 0x3A,
    ];
    let buffer3 = [0u8; 132];
    let buffer4 = [
        0xED, 0x25, 0xD1, 0xBB, 0xBC, 0x27, 0x9F, 0x02, 0xA2, 0xA9, 0x11, 0x00, 0x0C, 0xB3, 0x52, 0xC0, 0xBD, 0xE3, 0x1B, 0x49, 0xC7,
    ];

    #[allow(clippy::needless_range_loop)]
    for i in 0..210usize {
        let idx = i % 64;
        let src = (idx & !3).wrapping_add(3usize.wrapping_sub(idx & 3));
        buffer1[i] = block_in[src];
    }

    for i in 0..840u32 {
        let x = buffer1[i.wrapping_sub(155) as usize % 210];
        let y = buffer1[i.wrapping_sub(57) as usize % 210];
        let z = buffer1[i.wrapping_sub(13) as usize % 210];
        let w = buffer1[i as usize % 210];
        buffer1[i as usize % 210] = rol8(y, 5)
            .wrapping_add(rol8(z, 3) ^ w)
            .wrapping_sub(rol8(x, 7));
    }

    (buffer0, buffer1, buffer2, buffer3, buffer4)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{modified_md5_rust, playfair_decrypt, sap_buffers_from_block, sap_hash_rust};

    include!("scrambler_expected.rs");

    fn next_seed(seed: &mut u32) -> u8 {
        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (*seed >> 24) as u8
    }

    fn make_message3(seed: &mut u32, iter: usize) -> [u8; 164] {
        let mut message3 = [0u8; 164];
        for b in &mut message3 {
            *b = next_seed(seed);
        }
        message3[4] = 0x03;
        message3[12] = if iter & 1 == 0 { 0x03 } else { 0x01 };
        message3
    }

    fn make_input72(seed: &mut u32) -> [u8; 72] {
        let mut input72 = [0u8; 72];
        for b in &mut input72 {
            *b = next_seed(seed);
        }
        input72[4] = 0x01;
        input72
    }

    fn make_block(seed: &mut u32) -> [u8; 64] {
        let mut block = [0u8; 64];
        for b in &mut block {
            *b = next_seed(seed);
        }
        block
    }

    fn make_key(seed: &mut u32) -> [u8; 16] {
        let mut key = [0u8; 16];
        for b in &mut key {
            *b = next_seed(seed);
        }
        key
    }

    #[test]
    fn playfair_decrypt_matches_expected_vector() {
        assert_eq!(playfair_decrypt(&TEST_MESSAGE_3, &TEST_EKEY).unwrap(), TEST_OUTPUT);
        assert_eq!(playfair_decrypt(&TEST_MESSAGE_3, &TEST_EKEY).unwrap(), EXPECTED_DECRYPT);
    }

    #[test]
    fn playfair_decrypt_matches_expected_table() {
        let mut seed = 0x2468_acf0_u32;
        let mut table = Vec::with_capacity(EXPECTED_DECRYPT_TABLE_64.len());
        for iter in 0..EXPECTED_DECRYPT_TABLE_64.len() {
            let message3 = make_message3(&mut seed, iter);
            let input72 = make_input72(&mut seed);
            table.push(playfair_decrypt(&message3, &input72).unwrap());
        }
        assert_eq!(table, EXPECTED_DECRYPT_TABLE_64);
    }

    #[test]
    fn modified_md5_matches_expected_vector() {
        assert_eq!(modified_md5_rust(&TEST_MD5_BLOCK, &TEST_MD5_KEY), EXPECTED_MD5);
    }

    #[test]
    fn sap_hash_matches_expected_vector() {
        assert_eq!(sap_hash_rust(&TEST_MD5_BLOCK, &TEST_MD5_KEY), EXPECTED_SAP);
    }

    #[test]
    fn sap_hash_matches_expected_table() {
        let mut seed = 0x1357_9bdf_u32;
        let mut table = Vec::with_capacity(EXPECTED_SAP_TABLE_128.len());
        for _ in 0..EXPECTED_SAP_TABLE_128.len() {
            let block = make_block(&mut seed);
            let key = make_key(&mut seed);
            table.push(sap_hash_rust(&block, &key));
        }
        assert_eq!(table, EXPECTED_SAP_TABLE_128);
    }

    #[test]
    fn garble_rust_matches_expected_case() {
        let block = [
            0x20, 0xec, 0xde, 0xae, 0xff, 0xee, 0x5a, 0x72, 0x66, 0x84, 0xf8, 0x59, 0x9f, 0xab, 0xe8, 0xac, 0xdc, 0xe2, 0xaf, 0x06, 0x7c,
            0x26, 0x70, 0x49, 0xe3, 0xf7, 0x69, 0xb6, 0x7d, 0xcf, 0x5a, 0x6e, 0xb6, 0xe4, 0xcb, 0xa2, 0x07, 0x29, 0x62, 0x2a, 0x64, 0xe2,
            0xbd, 0x5e, 0xa2, 0x3f, 0x87, 0x59, 0xa3, 0xc8, 0xbb, 0x06, 0x6e, 0xa9, 0x44, 0xa1, 0x6b, 0x0d, 0x79, 0x84, 0x70, 0x20, 0x1b,
            0x78,
        ];
        let (mut r0, mut r1, mut r2, mut r3, mut r4) = sap_buffers_from_block(&block);
        super::garble_rust(&mut r0, &mut r1, &mut r2, &mut r3, &mut r4);
        assert_eq!(r0, EXPECTED_GARBLE_0);
        assert_eq!(r1, EXPECTED_GARBLE_1);
        assert_eq!(r2, EXPECTED_GARBLE_2);
        assert_eq!(r3, EXPECTED_GARBLE_3);
        assert_eq!(r4, EXPECTED_GARBLE_4);
    }
}
