use crate::fast_chacha::init_cpu_caps;

#[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
#[derive(Clone)]
#[repr(C, align(4))]
struct Aligned16([u8; 16]);

#[derive(Clone)]
pub struct FastPoly1305 {
    backend: FastPoly1305Backend,
}

#[derive(Clone)]
enum FastPoly1305Backend {
    Fallback {
        state: FallbackPoly1305State,
        buffer: [u8; 16],
        leftover: usize,
    },
    #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
    ScalarAsm {
        ctx: [u32; 12],
        pad: Aligned16,
        buffer: Aligned16,
        leftover: usize,
    },
}

impl FastPoly1305 {
    pub fn new(key: &[u8; 32]) -> Self {
        init_cpu_caps();

        #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
        {
            let mut ctx = [0u32; 12];
            let mut pad = Aligned16([0u8; 16]);
            pad.0.copy_from_slice(&key[16..]);
            unsafe {
                poly1305_init(ctx.as_mut_ptr(), key.as_ptr());
            }
            return Self {
                backend: FastPoly1305Backend::ScalarAsm {
                    ctx,
                    pad,
                    buffer: Aligned16([0; 16]),
                    leftover: 0,
                },
            };
        }

        #[allow(unreachable_code)]
        Self::fallback(key)
    }

    pub fn fallback(key: &[u8; 32]) -> Self {
        Self {
            backend: FastPoly1305Backend::Fallback {
                state: FallbackPoly1305State::new(key),
                buffer: [0u8; 16],
                leftover: 0,
            },
        }
    }

    pub fn is_fallback(&self) -> bool {
        matches!(self.backend, FastPoly1305Backend::Fallback { .. })
    }

    pub fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        match &mut self.backend {
            FastPoly1305Backend::Fallback { state, buffer, leftover } => {
                let mut input = data;

                if *leftover != 0 {
                    let want = (16 - *leftover).min(input.len());
                    buffer[*leftover..*leftover + want].copy_from_slice(&input[..want]);
                    *leftover += want;
                    input = &input[want..];

                    if *leftover == 16 {
                        state.compute_block(buffer, false);
                        *leftover = 0;
                    }
                }

                let full_len = input.len() & !0xf;
                if full_len != 0 {
                    for chunk in input[..full_len].chunks_exact(16) {
                        state.compute_block(chunk.try_into().unwrap(), false);
                    }
                    input = &input[full_len..];
                }

                if !input.is_empty() {
                    buffer[..input.len()].copy_from_slice(input);
                    *leftover = input.len();
                }
            }
            #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
            FastPoly1305Backend::ScalarAsm { ctx, buffer, leftover, .. } => {
                let mut input = data;

                if *leftover != 0 {
                    let want = (16 - *leftover).min(input.len());
                    buffer.0[*leftover..*leftover + want].copy_from_slice(&input[..want]);
                    *leftover += want;
                    input = &input[want..];

                    if *leftover == 16 {
                        unsafe {
                            poly1305_blocks(ctx.as_mut_ptr(), buffer.0.as_ptr(), 16, 1);
                        }
                        *leftover = 0;
                    }
                }

                let full_len = input.len() & !0xf;
                if full_len != 0 {
                    unsafe {
                        poly1305_blocks(ctx.as_mut_ptr(), input.as_ptr(), full_len, 1);
                    }
                    input = &input[full_len..];
                }

                if !input.is_empty() {
                    buffer.0[..input.len()].copy_from_slice(input);
                    *leftover = input.len();
                }
            }
        }
    }

    pub fn finalize(self) -> [u8; 16] {
        match self.backend {
            FastPoly1305Backend::Fallback {
                mut state,
                buffer,
                leftover,
            } => {
                if leftover != 0 {
                    let mut block = [0u8; 16];
                    block[..leftover].copy_from_slice(&buffer[..leftover]);
                    block[leftover] = 1;
                    state.compute_block(&block, true);
                }

                state.finalize()
            }
            #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
            FastPoly1305Backend::ScalarAsm {
                mut ctx,
                pad,
                mut buffer,
                leftover,
            } => {
                if leftover != 0 {
                    buffer.0[leftover] = 1;
                    buffer.0[leftover + 1..].fill(0);
                    unsafe {
                        poly1305_blocks(ctx.as_mut_ptr(), buffer.0.as_ptr(), 16, 0);
                    }
                }

                let mut out = Aligned16([0u8; 16]);
                unsafe {
                    poly1305_emit(ctx.as_mut_ptr(), out.0.as_mut_ptr(), pad.0.as_ptr());
                }
                out.0
            }
        }
    }

    pub fn compute(key: &[u8; 32], data: &[u8]) -> [u8; 16] {
        let mut poly = Self::new(key);
        poly.update(data);
        poly.finalize()
    }

    pub fn reset(&mut self, key: &[u8; 32]) {
        match &mut self.backend {
            FastPoly1305Backend::Fallback { state, buffer, leftover } => {
                *state = FallbackPoly1305State::new(key);
                *buffer = [0u8; 16];
                *leftover = 0;
            }
            #[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
            FastPoly1305Backend::ScalarAsm { ctx, pad, leftover, .. } => {
                *ctx = [0u32; 12];
                pad.0.copy_from_slice(&key[16..]);
                *leftover = 0;
                unsafe {
                    poly1305_init(ctx.as_mut_ptr(), key.as_ptr());
                }
            }
        }
    }
}

#[derive(Clone, Default)]
struct FallbackPoly1305State {
    r: [u32; 5],
    h: [u32; 5],
    pad: [u32; 4],
}

impl FallbackPoly1305State {
    fn new(key: &[u8; 32]) -> Self {
        let mut poly = Self::default();

        // Clamp r to 0xffffffc0ffffffc0ffffffc0fffffff.
        poly.r[0] = (u32::from_le_bytes(key[0..4].try_into().unwrap())) & 0x3ff_ffff;
        poly.r[1] = (u32::from_le_bytes(key[3..7].try_into().unwrap()) >> 2) & 0x3ff_ff03;
        poly.r[2] = (u32::from_le_bytes(key[6..10].try_into().unwrap()) >> 4) & 0x3ff_c0ff;
        poly.r[3] = (u32::from_le_bytes(key[9..13].try_into().unwrap()) >> 6) & 0x3f0_3fff;
        poly.r[4] = (u32::from_le_bytes(key[12..16].try_into().unwrap()) >> 8) & 0x00f_ffff;

        poly.pad[0] = u32::from_le_bytes(key[16..20].try_into().unwrap());
        poly.pad[1] = u32::from_le_bytes(key[20..24].try_into().unwrap());
        poly.pad[2] = u32::from_le_bytes(key[24..28].try_into().unwrap());
        poly.pad[3] = u32::from_le_bytes(key[28..32].try_into().unwrap());

        poly
    }

    fn compute_block(&mut self, block: &[u8; 16], partial: bool) {
        let hibit = if partial { 0 } else { 1 << 24 };

        let r0 = self.r[0];
        let r1 = self.r[1];
        let r2 = self.r[2];
        let r3 = self.r[3];
        let r4 = self.r[4];

        let s1 = r1 * 5;
        let s2 = r2 * 5;
        let s3 = r3 * 5;
        let s4 = r4 * 5;

        let mut h0 = self.h[0];
        let mut h1 = self.h[1];
        let mut h2 = self.h[2];
        let mut h3 = self.h[3];
        let mut h4 = self.h[4];

        h0 += (u32::from_le_bytes(block[0..4].try_into().unwrap())) & 0x3ff_ffff;
        h1 += (u32::from_le_bytes(block[3..7].try_into().unwrap()) >> 2) & 0x3ff_ffff;
        h2 += (u32::from_le_bytes(block[6..10].try_into().unwrap()) >> 4) & 0x3ff_ffff;
        h3 += (u32::from_le_bytes(block[9..13].try_into().unwrap()) >> 6) & 0x3ff_ffff;
        h4 += (u32::from_le_bytes(block[12..16].try_into().unwrap()) >> 8) | hibit;

        let d0 = (u64::from(h0) * u64::from(r0))
            + (u64::from(h1) * u64::from(s4))
            + (u64::from(h2) * u64::from(s3))
            + (u64::from(h3) * u64::from(s2))
            + (u64::from(h4) * u64::from(s1));

        let mut d1 = (u64::from(h0) * u64::from(r1))
            + (u64::from(h1) * u64::from(r0))
            + (u64::from(h2) * u64::from(s4))
            + (u64::from(h3) * u64::from(s3))
            + (u64::from(h4) * u64::from(s2));

        let mut d2 = (u64::from(h0) * u64::from(r2))
            + (u64::from(h1) * u64::from(r1))
            + (u64::from(h2) * u64::from(r0))
            + (u64::from(h3) * u64::from(s4))
            + (u64::from(h4) * u64::from(s3));

        let mut d3 = (u64::from(h0) * u64::from(r3))
            + (u64::from(h1) * u64::from(r2))
            + (u64::from(h2) * u64::from(r1))
            + (u64::from(h3) * u64::from(r0))
            + (u64::from(h4) * u64::from(s4));

        let mut d4 = (u64::from(h0) * u64::from(r4))
            + (u64::from(h1) * u64::from(r3))
            + (u64::from(h2) * u64::from(r2))
            + (u64::from(h3) * u64::from(r1))
            + (u64::from(h4) * u64::from(r0));

        let mut c: u32;
        c = (d0 >> 26) as u32;
        h0 = d0 as u32 & 0x3ff_ffff;
        d1 += u64::from(c);

        c = (d1 >> 26) as u32;
        h1 = d1 as u32 & 0x3ff_ffff;
        d2 += u64::from(c);

        c = (d2 >> 26) as u32;
        h2 = d2 as u32 & 0x3ff_ffff;
        d3 += u64::from(c);

        c = (d3 >> 26) as u32;
        h3 = d3 as u32 & 0x3ff_ffff;
        d4 += u64::from(c);

        c = (d4 >> 26) as u32;
        h4 = d4 as u32 & 0x3ff_ffff;
        h0 += c * 5;

        c = h0 >> 26;
        h0 &= 0x3ff_ffff;
        h1 += c;

        self.h[0] = h0;
        self.h[1] = h1;
        self.h[2] = h2;
        self.h[3] = h3;
        self.h[4] = h4;
    }

    fn finalize(self) -> [u8; 16] {
        let mut h0 = self.h[0];
        let mut h1 = self.h[1];
        let mut h2 = self.h[2];
        let mut h3 = self.h[3];
        let mut h4 = self.h[4];

        let mut c: u32;
        c = h1 >> 26;
        h1 &= 0x3ff_ffff;
        h2 += c;

        c = h2 >> 26;
        h2 &= 0x3ff_ffff;
        h3 += c;

        c = h3 >> 26;
        h3 &= 0x3ff_ffff;
        h4 += c;

        c = h4 >> 26;
        h4 &= 0x3ff_ffff;
        h0 += c * 5;

        c = h0 >> 26;
        h0 &= 0x3ff_ffff;
        h1 += c;

        let mut g0 = h0.wrapping_add(5);
        c = g0 >> 26;
        g0 &= 0x3ff_ffff;

        let mut g1 = h1.wrapping_add(c);
        c = g1 >> 26;
        g1 &= 0x3ff_ffff;

        let mut g2 = h2.wrapping_add(c);
        c = g2 >> 26;
        g2 &= 0x3ff_ffff;

        let mut g3 = h3.wrapping_add(c);
        c = g3 >> 26;
        g3 &= 0x3ff_ffff;

        let mut g4 = h4.wrapping_add(c).wrapping_sub(1 << 26);

        let mut mask = (g4 >> 31).wrapping_sub(1);
        g0 &= mask;
        g1 &= mask;
        g2 &= mask;
        g3 &= mask;
        g4 &= mask;

        mask = !mask;
        h0 = (h0 & mask) | g0;
        h1 = (h1 & mask) | g1;
        h2 = (h2 & mask) | g2;
        h3 = (h3 & mask) | g3;
        h4 = (h4 & mask) | g4;

        h0 |= h1 << 26;
        h1 = (h1 >> 6) | (h2 << 20);
        h2 = (h2 >> 12) | (h3 << 14);
        h3 = (h3 >> 18) | (h4 << 8);

        let mut f = u64::from(h0) + u64::from(self.pad[0]);
        h0 = f as u32;

        f = u64::from(h1) + u64::from(self.pad[1]) + (f >> 32);
        h1 = f as u32;

        f = u64::from(h2) + u64::from(self.pad[2]) + (f >> 32);
        h2 = f as u32;

        f = u64::from(h3) + u64::from(self.pad[3]) + (f >> 32);
        h3 = f as u32;

        let mut tag = [0u8; 16];
        tag[0..4].copy_from_slice(&h0.to_le_bytes());
        tag[4..8].copy_from_slice(&h1.to_le_bytes());
        tag[8..12].copy_from_slice(&h2.to_le_bytes());
        tag[12..16].copy_from_slice(&h3.to_le_bytes());
        tag
    }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", any(target_arch = "mips", target_arch = "riscv32")))]
unsafe extern "C" {
    fn poly1305_init(ctx: *mut u32, key: *const u8) -> i32;
    fn poly1305_blocks(ctx: *mut u32, inp: *const u8, len: usize, padbit: u32);
    fn poly1305_emit(ctx: *mut u32, mac: *mut u8, nonce: *const u8);
}

#[cfg(all(
    test,
    not(any(target_arch = "mips", target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6"))
))]
mod tests {
    use super::FastPoly1305;

    #[test]
    fn poly1305_rfc7539_vector() {
        let key = [
            0x85, 0xd6, 0xbe, 0x78, 0x57, 0x55, 0x6d, 0x33, 0x7f, 0x44, 0x52, 0xfe, 0x42, 0xd5, 0x06, 0xa8, 0x01, 0x03, 0x80, 0x8a, 0xfb,
            0x0d, 0xb2, 0xfd, 0x4a, 0xbf, 0xf6, 0xaf, 0x41, 0x49, 0xf5, 0x1b,
        ];
        let msg = b"Cryptographic Forum Research Group";
        let expected = [
            0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, 0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9,
        ];

        let mut poly = FastPoly1305::new(&key);
        poly.update(msg);

        assert_eq!(poly.finalize(), expected);
        assert_eq!(FastPoly1305::compute(&key, msg), expected);
    }

    #[test]
    fn poly1305_rfc7539_vector_fallback() {
        let key = [
            0x85, 0xd6, 0xbe, 0x78, 0x57, 0x55, 0x6d, 0x33, 0x7f, 0x44, 0x52, 0xfe, 0x42, 0xd5, 0x06, 0xa8, 0x01, 0x03, 0x80, 0x8a, 0xfb,
            0x0d, 0xb2, 0xfd, 0x4a, 0xbf, 0xf6, 0xaf, 0x41, 0x49, 0xf5, 0x1b,
        ];
        let msg = b"Cryptographic Forum Research Group";
        let expected = [
            0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, 0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9,
        ];

        let mut poly = FastPoly1305::fallback(&key);
        poly.update(msg);

        assert_eq!(poly.finalize(), expected);
        assert_eq!(FastPoly1305::compute(&key, msg), expected);
    }
}
