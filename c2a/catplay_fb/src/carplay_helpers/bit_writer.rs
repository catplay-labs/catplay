use exp_golomb::ExpGolombEncoder;

#[derive(Default)]
pub(super) struct BitWriter {
    pub(super) data: Vec<u8>,
    len: usize,
}

impl BitWriter {
    pub(super) fn bit(&mut self, bit: u8) {
        if self.len % 8 == 0 {
            self.data.push(0);
        }
        if bit != 0 {
            let last = self.data.len() - 1;
            self.data[last] |= 1 << (7 - self.len % 8);
        }
        self.len += 1;
    }

    pub(super) fn bits(&mut self, value: u32, count: usize) {
        for shift in (0..count).rev() {
            self.bit(((value >> shift) & 1) as u8);
        }
    }

    pub(super) fn ue(&mut self, value: u32) {
        // A u32 Exp-Golomb code occupies at most 65 bits.
        let mut encoded = [0u8; 9];
        let encoded_bits = {
            let mut encoder = ExpGolombEncoder::new(&mut encoded, 0).expect("non-empty Exp-Golomb buffer");
            encoder
                .put_unsigned(value as u64)
                .expect("u32 Exp-Golomb value fits in the temporary buffer");
            let (byte, bit) = encoder.close();
            byte * 8 + bit as usize
        };
        for pos in 0..encoded_bits {
            self.bit((encoded[pos / 8] >> (7 - pos % 8)) & 1);
        }
    }

    pub(super) fn se(&mut self, value: i32) {
        let code_num = if value <= 0 {
            value.unsigned_abs() * 2
        } else {
            value as u32 * 2 - 1
        };
        self.ue(code_num);
    }

    pub(super) fn align_one(&mut self) {
        while self.len % 8 != 0 {
            self.bit(1);
        }
    }

    fn align_zero(&mut self) {
        while self.len % 8 != 0 {
            self.bit(0);
        }
    }

    pub(super) fn rbsp_trailing_bits(&mut self) {
        self.bit(1);
        self.align_zero();
    }
}
