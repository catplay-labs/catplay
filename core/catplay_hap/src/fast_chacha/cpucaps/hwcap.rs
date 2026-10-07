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

/*!
 * Module: cpucaps::hwcap
 *
 * Provides low-level access to hardware capability information by reading
 * the auxiliary vector (/proc/self/auxv) on Linux systems.
 */

/// Reads the value associated with a given key from the auxiliary vector (/proc/self/auxv).
///
/// # Arguments
///
/// * `target` - The key (type) to search for in the auxiliary vector.
///
/// # Returns
///
/// * `Some(usize)` - The value associated with the given key, if found.
/// * `None` - If the key is not found or an error occurs.
///
/// # Safety
///
/// This function uses unsafe code to call libc functions (`open`, `read`, `close`)
/// and to interpret raw bytes as `usize`.
pub fn get_auxv(target: usize) -> Option<usize> {
    use core::ffi::{c_char, c_void};
    use core::mem::size_of;

    // Path to the auxiliary vector file for the current process.
    let path = b"/proc/self/auxv\0";
    unsafe extern "C" {
        /// Opens a file and returns a file descriptor.
        fn open(pathname: *const c_char, flags: i32, ...) -> i32;
        /// Reads data from a file descriptor into a buffer.
        fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
        /// Closes a file descriptor.
        fn close(fd: i32) -> i32;
    }

    // Open the auxv file for reading.
    let fd = unsafe { open(path.as_ptr().cast(), 0) };
    if fd < 0 {
        return None;
    }

    // Buffer to hold one (key, value) pair from auxv.
    let mut buf = [0u8; size_of::<usize>() * 2];
    // Read each (key, value) pair and check if the key matches the target.
    while unsafe { read(fd, buf.as_mut_ptr().cast(), buf.len()) } == buf.len() as isize {
        let key = usize::from_ne_bytes(buf[..size_of::<usize>()].try_into().unwrap());
        let val = usize::from_ne_bytes(buf[size_of::<usize>()..].try_into().unwrap());
        if key == target {
            unsafe { close(fd) };
            return Some(val);
        }
    }

    unsafe { close(fd) };
    None
}
