/*
 * Copyright 2025 sh0rch <sh0rch@iwl.dev>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! FastChaCha20: High-performance ChaCha20 stream cipher implementation with optional assembly optimizations.
//!
//! This module provides a ChaCha20 implementation that can use optimized assembly routines if available,
//! and falls back to a pure Rust implementation otherwise. It exposes a simple API for encryption and decryption
//! using the ChaCha20 stream cipher, suitable for cryptographic applications.
//!
//! # Examples
//!
//! Basic usage with a 32-byte key and 12-byte nonce:
//!
//! ```rust
//! use catplay_hap::fast_chacha::FastChaCha20;
//!
//! let key = [0u8; 32];
//! let nonce = [0u8; 12];
//! let mut data = b"plaintext data to encrypt".to_vec();
//!
//! let mut cipher = FastChaCha20::new(&key, &nonce);
//! cipher.apply_keystream(&mut data);
//! // `data` now contains the encrypted bytes
//!
//! // To decrypt, reinitialize with the same key/nonce and apply again:
//! let mut cipher = FastChaCha20::new(&key, &nonce);
//! cipher.apply_keystream(&mut data);
//! // `data` is now the original plaintext
//! ```
//!
//! To use the pure Rust fallback implementation explicitly:
//!
//! ```rust
//! use catplay_hap::fast_chacha::FastChaCha20;
//!
//! let key = [0u8; 32];
//! let nonce = [0u8; 12];
//! let mut data = b"some data".to_vec();
//! let mut cipher = FastChaCha20::new(&key, &nonce);
//! cipher.apply_keystream_pure(&mut data, 10); // 10 double rounds
//! // `data` is now encrypted using the pure Rust implementation
//! ```

// #![no_std]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(unused)]

extern crate alloc;

use crate::fast_chacha::{fallback_chacha20, init_cpu_caps};
use alloc::vec::Vec;
use core::{
    slice,
    sync::atomic::{AtomicBool, Ordering},
};

/// Atomic flag indicating whether the fallback (pure Rust) implementation was triggered.
static FALLBACK_TRIGGERED: AtomicBool = AtomicBool::new(false);

/// Fallback ChaCha20 implementation using pure Rust.
///
/// # Arguments
/// * `out` - Output buffer for the keystream XOR result.
/// * `inp` - Input buffer to be encrypted/decrypted.
/// * `key` - 256-bit key as 8 u32 words.
/// * `counter` - 128-bit counter as 4 u32 words.
#[inline(always)]
fn fallback(out: &mut [u8], len: usize, keystream_only: bool, key: &[u32; 8], counter: &mut [u32; 4], double_rounds: usize) {
    let mut offset = 0;
    while offset < len {
        let block_len = (len - offset).min(64);
        fallback_chacha20::xor(&mut out[offset..offset + block_len], keystream_only, key, counter, double_rounds);
        counter[0] = counter[0].wrapping_add(1);
        offset += block_len;
    }
}

/// C-compatible ChaCha20 function, used as a fallback or as the main implementation if assembly is not available.
///
/// # Safety
/// This function is unsafe because it operates on raw pointers.
///
/// # Arguments
/// * `out` - Pointer to output buffer.
/// * `inp` - Pointer to input buffer.
/// * `len` - Length of the input/output buffers.
/// * `key` - Pointer to the key (8 u32 words).
/// * `counter` - Pointer to the counter (4 u32 words).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ChaCha20_ctr32_c(out: *mut u8, inp: *const u8, len: usize, key: *const u32, counter: *const u32) {
    FALLBACK_TRIGGERED.store(true, Ordering::SeqCst);

    unsafe {
        let out = slice::from_raw_parts_mut(out, len);
        let keystream_only = inp.is_null();
        let key = &*(key as *const [u32; 8]);
        let ctr = &mut *(counter as *mut [u32; 4]);
        fallback(out, len, keystream_only, key, ctr, 10);
    }
}

#[cfg(fast_chacha_asm)]
unsafe extern "C" {
    /// External assembly-optimized ChaCha20 function.
    unsafe fn ChaCha20_ctr32(out: *mut u8, inp: *const u8, len: usize, key: *const u32, counter: *const u32);
}

#[cfg(not(fast_chacha_asm))]
/// Pure Rust fallback for ChaCha20_ctr32 if assembly is not enabled.
///
/// # Safety
/// This function is unsafe because it operates on raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ChaCha20_ctr32(out: *mut u8, inp: *const u8, len: usize, key: *const u32, counter: *const u32) {
    unsafe { ChaCha20_ctr32_c(out, inp, len, key, counter) };
}

/// FastChaCha20: Main struct representing a ChaCha20 cipher instance.
///
/// Holds the key and counter state for encryption/decryption.
///
///
#[derive(Clone)]
pub struct FastChaCha20 {
    /// 256-bit key as 8 u32 words.
    key_words: [u32; 8],
    /// 128-bit counter as 4 u32 words.
    counter: [u32; 4],
}

impl FastChaCha20 {
    /// Creates a new FastChaCha20 instance with the given key and nonce.
    ///
    /// # Arguments
    /// * `key` - 32-byte (256-bit) key.
    /// * `nonce` - 12-byte (96-bit) nonce.
    ///
    /// # Panics
    /// Panics if the key or nonce are not the correct length.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let key = [0u8; 32];
    /// let nonce = [0u8; 12];
    /// let cipher = FastChaCha20::new(&key, &nonce);
    /// ```
    pub fn new(key: &[u8; 32], nonce: &[u8; 12]) -> Self {
        debug_assert!(key.len() == 32, "Key must be 32 bytes");
        debug_assert!(nonce.len() == 12, "Nonce must be 12 bytes");

        init_cpu_caps();

        let key_words = [
            u32::from_le_bytes([key[0], key[1], key[2], key[3]]),
            u32::from_le_bytes([key[4], key[5], key[6], key[7]]),
            u32::from_le_bytes([key[8], key[9], key[10], key[11]]),
            u32::from_le_bytes([key[12], key[13], key[14], key[15]]),
            u32::from_le_bytes([key[16], key[17], key[18], key[19]]),
            u32::from_le_bytes([key[20], key[21], key[22], key[23]]),
            u32::from_le_bytes([key[24], key[25], key[26], key[27]]),
            u32::from_le_bytes([key[28], key[29], key[30], key[31]]),
        ];

        let counter = [
            0,
            u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]),
            u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]),
            u32::from_le_bytes([nonce[8], nonce[9], nonce[10], nonce[11]]),
        ];

        Self { key_words, counter }
    }

    /// Applies the ChaCha20 keystream to the given data in-place, using the fastest available implementation.
    ///
    /// # Arguments
    /// * `data` - Mutable buffer to encrypt/decrypt.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let key = [0u8; 32];
    /// let nonce = [0u8; 12];
    /// let mut cipher = FastChaCha20::new(&key, &nonce);
    /// let mut data = [1u8, 2, 3, 4, 5];
    /// cipher.apply_keystream(&mut data);
    /// ```
    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        if data.is_empty() {
            return;
        }

        unsafe {
            ChaCha20_ctr32(
                data.as_mut_ptr(),
                data.as_mut_ptr(),
                data.len(),
                self.key_words.as_ptr(),
                self.counter.as_mut_ptr(),
            )
        }
    }

    /// Applies the ChaCha20 keystream using the pure Rust fallback implementation.
    ///
    /// # Arguments
    /// * `data` - Mutable buffer to encrypt/decrypt.
    /// * `double_rounds` - Number of double rounds to apply (default is 10).
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let key = [0u8; 32];
    /// let nonce = [0u8; 12];
    /// let mut cipher = FastChaCha20::new(&key, &nonce);
    /// let mut data = [1u8, 2, 3, 4, 5];
    /// cipher.apply_keystream_pure(&mut data, 10);
    /// ```
    pub fn apply_keystream_pure(&mut self, data: &mut [u8], double_rounds: usize) {
        if data.is_empty() {
            return;
        }
        // Avoid aliasing mutable and immutable borrows by splitting the slice
        fallback(data, data.len(), false, &self.key_words, &mut self.counter, double_rounds);
    }

    pub fn keystream_only(&mut self, data: &mut [u8]) {
        if data.is_empty() {
            return;
        }
        // The MIPS o32 and RV32 backends accept null input for raw keystream.
        #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
        unsafe {
            ChaCha20_ctr32(
                data.as_mut_ptr(),
                core::ptr::null(),
                data.len(),
                self.key_words.as_ptr(),
                self.counter.as_ptr(),
            );
            // ASM keeps its counter local, unlike the Rust implementation.
            self.counter[0] = self.counter[0].wrapping_add(data.len().div_ceil(64) as u32);
        }
        #[cfg(not(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32"))))]
        fallback(data, data.len(), true, &self.key_words, &mut self.counter, 10);
    }

    /// Resets the internal counter to zero.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let mut cipher = FastChaCha20::new(&[0u8; 32], &[0u8; 12]);
    /// cipher.reset();
    /// ```
    pub fn reset(&mut self) {
        self.counter[0] = 0;
    }

    pub fn set_counter(&mut self, counter: u32) {
        self.counter[0] = counter;
    }

    /// Sets the internal counter to the position corresponding to the given byte offset.
    ///
    /// # Arguments
    /// * `pos` - Byte position to seek to.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let mut cipher = FastChaCha20::new(&[0u8; 32], &[0u8; 12]);
    /// cipher.seek(128);
    /// ```
    pub fn seek(&mut self, pos: u64) {
        self.counter[0] = (pos / 64) as u32;
    }

    /// Returns the current byte position in the stream.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let cipher = FastChaCha20::new(&[0u8; 32], &[0u8; 12]);
    /// let pos = cipher.current_pos();
    /// ```
    pub fn current_pos(&self) -> u64 {
        (self.counter[0] as u64) * 64
    }

    /// Creates a new FastChaCha20 instance with a custom initial counter value.
    ///
    /// # Arguments
    /// * `key` - 32-byte key.
    /// * `nonce` - 12-byte nonce.
    /// * `counter` - Initial counter value.
    ///
    /// # Example
    /// ```
    /// use catplay_hap::fast_chacha::FastChaCha20;
    ///
    /// let key = [0u8; 32];
    /// let nonce = [0u8; 12];
    /// let cipher = FastChaCha20::new_with_counter(key, nonce, 42);
    /// ```
    pub fn new_with_counter(key: [u8; 32], nonce: [u8; 12], counter: u32) -> Self {
        let mut s = Self::new(&key, &nonce);
        s.counter[0] = counter;

        s
    }
}

/// Dummy static used for feature detection.
#[repr(align(16))]
struct Dummy([u8; 16]);
static mut DUMMY: Dummy = Dummy([0; 16]);

/// Checks if the assembly-optimized implementation is available at runtime.
///
/// Returns `true` if the assembly implementation is available, `false` otherwise.
///
/// # Example
/// ```
/// if catplay_hap::fast_chacha::is_asm_available_chacha20() {
///     // Use the fastest implementation
/// }
/// ```
pub fn is_asm_available_chacha20() -> bool {
    FALLBACK_TRIGGERED.store(false, Ordering::SeqCst);
    unsafe {
        init_cpu_caps();
        let dummy_ptr = core::ptr::addr_of_mut!(DUMMY.0) as *mut u8;
        ChaCha20_ctr32(dummy_ptr, dummy_ptr, 0, dummy_ptr as *const u32, dummy_ptr as *const u32);
    }
    !FALLBACK_TRIGGERED.load(Ordering::SeqCst)
}

#[cfg(test)]
mod keystream_tests {
    use super::*;
    use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

    #[test]
    fn direct_keystream_overwrites_aligned_unaligned_and_partial_buffers() {
        // RustCrypto reserves its terminal block; stay below that boundary.
        for counter in [0, 1, 71, u32::MAX - 2] {
            for len in [0usize, 1, 3, 4, 63, 64, 65, 127] {
                for align in 0..4 {
                    let key = [0x37; 32];
                    let nonce = [0x65; 12];
                    let mut direct = FastChaCha20::new_with_counter(key, nonce, counter);
                    let mut pure = direct.clone();
                    let mut guarded = vec![0xaa; len + 8];
                    direct.keystream_only(&mut guarded[align..align + len]);
                    let mut expected = vec![0; len];
                    let mut reference = chacha20::ChaCha20::new((&key).into(), (&nonce).into());
                    reference.seek(u64::from(counter) * 64);
                    reference.apply_keystream(&mut expected);
                    assert_eq!(&guarded[align..align + len], expected);
                    assert!(
                        guarded[..align]
                            .iter()
                            .chain(&guarded[align + len..])
                            .all(|&b| b == 0xaa)
                    );
                    let mut soft = vec![0x55; len];
                    fallback(&mut soft, len, true, &pure.key_words, &mut pure.counter, 10);
                    assert_eq!(soft, expected);
                    assert_eq!(direct.counter, pure.counter);
                }
            }
        }
    }
}
