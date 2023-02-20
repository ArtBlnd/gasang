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

pub fn parse_pattern(pattern: &str) -> Pattern {
    let mut pattern_result = 0b0;
    let mut mask_result = 0b0;

    for char in pattern.chars() {
        let (pat, mask) = match char {
            'x' => (0, 0),
            '0' => (0, 1),
            '1' => (1, 1),
            '_' | ' ' => {
                continue;
            }
            _ => unreachable!("Bad parse pattern!"),
        };
        pattern_result <<= 1;
        pattern_result |= pat;

        mask_result <<= 1;
        mask_result |= mask;
    }

    Pattern {
        pattern: pattern_result,
        mask: mask_result,
    }
}

pub struct Pattern {
    pattern: u32,
    mask: u32,
}

impl Pattern {
    pub fn test_u32(&self, target: u32) -> bool {
        (!(target ^ self.pattern) & self.mask) == self.mask
    }

    pub fn test_u8(&self, target: u8) -> bool {
        let target = target as u32;
        (!(target ^ self.pattern) & self.mask) == self.mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits32() {
        let bits: u32 = 0b1111_0000_1010_0101_1100_0011_1001_0110;

        assert_eq!(0b0011, extract_bits32(8..12, bits))
    }
}
