// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Writing and reading codes bit by bit, the most significant bit first.

/// Writes codes into bytes.
#[derive(Default)]
pub(super) struct BitWriter {
    bytes: Vec<u8>,
    /// The number of bits used of the last byte, 0 for a full or no byte.
    used: u8,
}

impl BitWriter {
    /// Write the lowest `bits` bits of `code`.
    pub fn write(&mut self, code: u32, bits: u8) {
        for bit in (0..bits).rev() {
            if self.used == 0 {
                self.bytes.push(0);
            }
            if code >> bit & 1 == 1 {
                *self.bytes.last_mut().expect("A byte") |= 0x80 >> self.used;
            }
            self.used = (self.used + 1) % 8;
        }
    }

    /// Write the number `n` > 0 in Elias gamma code: the number of its bits - 1 as 0 bits,
    /// followed by its bits, e.g. `1` for 1 and `00101` for 5.
    pub fn write_gamma(&mut self, n: u32) {
        debug_assert!(n > 0, "Elias gamma code needs a positive number");
        let bits = 32 - n.leading_zeros() as u8;
        self.write(0, bits - 1);
        self.write(n, bits);
    }

    /// The written bytes, the last byte is filled with 1 bits.
    pub fn finish(mut self) -> Vec<u8> {
        if self.used > 0 {
            *self.bytes.last_mut().expect("A byte") |= 0xff >> self.used;
        }
        self.bytes
    }
}

/// Reads bits from bytes.
pub(super) struct BitReader<'a> {
    bytes: &'a [u8],
    /// The position of the next bit.
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    /// The next bit, `None` at the end.
    pub fn read(&mut self) -> Option<bool> {
        let byte = self.bytes.get(self.position / 8)?;
        let bit = byte >> (7 - self.position % 8) & 1 == 1;
        self.position += 1;
        Some(bit)
    }

    /// The next `bits` bits as number, `None` at the end.
    pub fn read_bits(&mut self, bits: u8) -> Option<u32> {
        (0..bits).try_fold(0, |code, _| Some(code << 1 | self.read()? as u32))
    }

    /// The next number in Elias gamma code, see [`BitWriter::write_gamma`], `None` at the end
    /// or for a number with more than 32 bits.
    pub fn read_gamma(&mut self) -> Option<u32> {
        let mut zeros = 0;
        while !self.read()? {
            zeros += 1;
            if zeros >= 32 {
                return None;
            }
        }
        Some(1 << zeros | self.read_bits(zeros)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Codes are written and read the most significant bit first, filled with 1 bits.
    #[test]
    fn write_and_read() {
        let mut writer = BitWriter::default();
        writer.write(0b101, 3);
        writer.write(0b0000_0001, 8);
        let bytes = writer.finish();
        assert_eq!(bytes, [0b1010_0000, 0b0011_1111]);

        let mut reader = BitReader::new(&bytes);
        assert_eq!(reader.read_bits(3), Some(0b101));
        assert_eq!(reader.read_bits(8), Some(1));
        assert_eq!(reader.read_bits(5), Some(0b11111));
        assert_eq!(reader.read(), None);
        assert_eq!(BitWriter::default().finish(), Vec::<u8>::new());
    }

    /// Numbers are written and read in Elias gamma code.
    #[test]
    fn gamma() {
        let mut writer = BitWriter::default();
        for n in [1, 2, 5, 1000, u32::MAX] {
            writer.write_gamma(n);
        }
        let bytes = writer.finish();
        // 1, 010 and 00101.
        assert_eq!(bytes[0], 0b1010_0010);
        let mut reader = BitReader::new(&bytes);
        for n in [1, 2, 5, 1000, u32::MAX] {
            assert_eq!(reader.read_gamma(), Some(n));
        }
        assert_eq!(BitReader::new(&[0, 0, 0, 0, 0]).read_gamma(), None);
    }
}
