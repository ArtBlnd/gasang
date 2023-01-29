use std::ops::Range;

pub struct BitReader<I> {
    iter: I,
}

impl<I> BitReader<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(iter: I) -> Self {
        Self { iter }
    }

    pub fn read32(&mut self) -> Option<u32> {
        let mut inst: u32 = 0;
        for i in 0..4 {
            let byte = self.iter.next()?;
            inst |= (byte as u32) << (i * 8);
        }

        Some(inst)
    }

    pub fn read64(&mut self) -> Option<u64> {
        let mut inst: u64 = 0;
        for i in 0..8 {
            let byte = self.iter.next()?;
            inst |= (byte as u64) << (i * 8);
        }

        Some(inst)
    }
}

pub const fn extract_bits32(range: Range<usize>, value: u32) -> u32 {
    let lshift = 32 - range.end;
    let left_shifted = value << lshift;

    left_shifted >> (range.start + lshift)
}

pub const fn extract_bits16(range: Range<usize>, value: u16) -> u16 {
    let lshift = 16 - range.end;
    let left_shifted = value << lshift;

    left_shifted >> (range.start + lshift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits32() {
        let bits: u32 = 0b1111_0000_1010_0101_1100_0011_1001_0110;

        assert_eq!(0b1100, extract_bits32(8..12, bits))
    }
}
