// Copyright 2025 sh0rch <sh0rch@iwl.dev>
// Licensed under the MIT License.
// You may obtain a copy of the License at
//
//     https://opensource.org/licenses/MIT
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Fallback ChaCha20 Implementation (Pure Rust)
//!
//! This module provides a minimal, dependency-free implementation of the ChaCha20
//! stream cipher. It is intended as a fallback when hardware-accelerated or
//! optimized versions are unavailable. The implementation is based on the
//! original ChaCha20 specification and operates on 256-bit keys and 96-bit nonces.
//!
//! ## Usage
//!
//! The main function is [`xor`], which applies the ChaCha20 keystream to a buffer
//! in-place for encryption or decryption. The function expects a 256-bit key and
//! a 128-bit counter/nonce (as four `u32` words).
//!
//! ## Example
//!
//! ```rust
//! use catplay_hap::fast_chacha::fallback_chacha20::xor;
//! let key = [0u32; 8];
//! let counter = [0u32; 4];
//! let mut data = [0u8; 64];
//! xor(&mut data, false, &key, &counter, 10);
//! ```
//!
//! # Security
//!
//! This implementation is intended for fallback or testing purposes. For
//! production use, prefer well-audited and hardware-accelerated libraries.

//! ## Module Overview
//!
//! - `quarter_round`: Core ChaCha20 operation mixing four words of the state.
//! - `xor`: Applies ChaCha20 keystream to a buffer for encryption/decryption or outputs keystream.
//! - Tests: Verifies correct encryption/decryption round-trip.

/// Performs a single ChaCha20 quarter round operation on the cipher state.
///
/// The quarter round is the core operation of the ChaCha20 algorithm. It mixes
/// four 32-bit words of the state using addition, XOR, and bit rotation. This
/// function is called multiple times per block to provide diffusion and security.
///
/// # Arguments
/// * `state` - Mutable reference to the cipher state array (16 words).
/// * `a`, `b`, `c`, `d` - Indices of the state words to operate on.
///
/// # Panics
/// Panics if any index is out of bounds for the state array.
#[inline(always)]
fn quarter_round(state: &mut [u32], a: usize, b: usize, c: usize, d: usize) {
    // Add, XOR, and rotate operations as specified by ChaCha20
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

/// XORs the given data in-place with a ChaCha20 keystream generated from the provided key and counter.
///
/// This function generates a single ChaCha20 block (64 bytes) using the provided key and counter,
/// and XORs it with the input data buffer. The buffer can be up to 64 bytes in length. This function
/// can be used for both encryption and decryption, as ChaCha20 is a symmetric stream cipher.
/// If `keystream_only` is true, the function writes the raw keystream to `data` instead of XORing.
///
/// # Arguments
/// * `data` - Mutable byte slice to encrypt, decrypt, or fill with keystream (up to 64 bytes).
/// * `keystream_only` - If true, output the keystream directly to `data` (must be 64 bytes).
/// * `key` - Reference to an array of 8 u32 words (256-bit key).
/// * `counter` - Reference to an array of 4 u32 words (block counter and nonce).
/// * `double_rounds` - Number of double rounds (each double round is 2 rounds, standard is 10).
///
/// # Panics
/// Panics if `data` is longer than 64 bytes, (both XOR and direct keystream output support partial blocks).
///
/// # Example
/// ```rust
/// use catplay_hap::fast_chacha::fallback_chacha20::xor;
/// let key = [0u32; 8];
/// let counter = [0u32; 4];
/// let mut data = [0u8; 64];
/// xor(&mut data, false, &key, &counter, 10);
/// ```
pub fn xor(data: &mut [u8], keystream_only: bool, key: &[u32; 8], counter: &[u32; 4], double_rounds: usize) {
    debug_assert!(data.len() <= 64, "Data length must not exceed 64 bytes");

    // State initialization: 4 constant words, 8 key words, 4 counter/nonce words
    let state = [
        0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, // constants
        key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7], // key
        counter[0], counter[1], counter[2], counter[3], // counter+nonce
    ];

    let mut working = state;

    // Apply the specified number of double rounds (each double round = 2 rounds)
    for _ in 0..double_rounds {
        // Column rounds
        quarter_round(&mut working, 0, 4, 8, 12);
        quarter_round(&mut working, 1, 5, 9, 13);
        quarter_round(&mut working, 2, 6, 10, 14);
        quarter_round(&mut working, 3, 7, 11, 15);
        // Diagonal rounds
        quarter_round(&mut working, 0, 5, 10, 15);
        quarter_round(&mut working, 1, 6, 11, 12);
        quarter_round(&mut working, 2, 7, 8, 13);
        quarter_round(&mut working, 3, 4, 9, 14);
    }

    // Add the original state to the working state, serialize, and XOR with data in one pass
    if keystream_only {
        // Output the keystream directly to data
        let mut i = 0;
        while i < data.len() {
            let word = working[i / 4].wrapping_add(state[i / 4]).to_le_bytes();
            let j = i % 4;
            data[i] = word[j];
            i += 1;
        }
    } else {
        // XOR the keystream with the input data
        let mut i = 0;
        while i < data.len() {
            let word = working[i / 4].wrapping_add(state[i / 4]).to_le_bytes();
            let j = i % 4;
            data[i] ^= word[j];
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fast_chacha::fallback_chacha20::xor;

    /// Tests that encrypting and then decrypting with the same key and counter
    /// returns the original plaintext, and that the ciphertext differs from the plaintext.
    #[test]
    fn test_chacha20_xor_encrypt_decrypt() {
        // Example key and nonce (all zeros for simplicity)
        let key = [0u8; 32];
        let nonce = [0u8; 12];

        // Convert key to u32 words (little-endian)
        let mut key_words = [0u32; 8];
        for i in 0..8 {
            key_words[i] = u32::from_le_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
        }

        // Counter: 4 words, first is 0, next 3 from nonce (little-endian)
        let mut counter = [0u32; 4];
        for i in 0..3 {
            counter[i + 1] = u32::from_le_bytes([nonce[i * 4], nonce[i * 4 + 1], nonce[i * 4 + 2], nonce[i * 4 + 3]]);
        }

        // Plaintext to encrypt
        let plaintext = b"Hello, ChaCha20 fallback test!";
        let mut buffer = [0u8; 32];
        buffer[..plaintext.len()].copy_from_slice(plaintext);
        // Encrypt in place
        xor(&mut buffer[..plaintext.len()], false, &key_words, &counter, 10);

        // Save ciphertext for assertion
        let mut ciphertext = [0u8; 64];
        ciphertext[..plaintext.len()].copy_from_slice(&buffer[..plaintext.len()]);
        // Decrypt in place
        xor(&mut buffer[..plaintext.len()], false, &key_words, &counter, 10);

        // Check that decryption yields the original plaintext
        assert_eq!(&buffer[..plaintext.len()], plaintext);

        // Check that ciphertext is not equal to plaintext
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);
    }
}
