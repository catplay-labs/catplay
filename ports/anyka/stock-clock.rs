// SPDX-License-Identifier: GPL-2.0-only
// The vendor clock_gettime64 syscall returns success without filling CLOCK_MONOTONIC.
// Override libc's clock functions for stock-kernel ARMv5 programs.
// The legacy syscall works; convert its 32-bit fields to the target timespec ABI.
#[repr(C)]
struct LegacyTimespec {
    seconds: i32,
    nanoseconds: i32,
}

#[unsafe(no_mangle)]
unsafe extern "C" fn clock_gettime(clock: libc::clockid_t, output: *mut libc::timespec) -> libc::c_int {
    if output.is_null() {
        unsafe { *libc::__errno_location() = libc::EFAULT };
        return -1;
    }
    let mut legacy = LegacyTimespec {
        seconds: 0,
        nanoseconds: 0,
    };
    // ARM EABI syscall number from arch/arm/include/uapi/asm/unistd.h.
    let result = unsafe { libc::syscall(263, clock, &mut legacy as *mut LegacyTimespec) };
    if result != 0 {
        return -1;
    }
    unsafe {
        (*output).tv_sec = legacy.seconds.into();
        (*output).tv_nsec = legacy.nanoseconds.into();
    }
    0
}

// New GNU Rust standard libraries also probe this glibc time64 entry point.
// ARM glibc uses a 64-bit seconds field followed by 32-bit nanoseconds and
// padding; writing a zero-extended 64-bit nanoseconds slot matches that layout.
#[cfg(target_env = "gnu")]
#[repr(C)]
struct GnuTime64 {
    seconds: i64,
    nanoseconds: i64,
}

#[cfg(target_env = "gnu")]
#[unsafe(no_mangle)]
unsafe extern "C" fn __clock_gettime64(clock: libc::clockid_t, output: *mut GnuTime64) -> libc::c_int {
    if output.is_null() {
        unsafe { *libc::__errno_location() = libc::EFAULT };
        return -1;
    }
    let mut legacy = LegacyTimespec {
        seconds: 0,
        nanoseconds: 0,
    };
    if unsafe { libc::syscall(263, clock, &mut legacy as *mut LegacyTimespec) } != 0 {
        return -1;
    }
    unsafe {
        (*output).seconds = legacy.seconds.into();
        (*output).nanoseconds = legacy.nanoseconds.into();
    }
    0
}
