use std::marker::PhantomData;
use std::ops::Range;

use crate::FixedBytesArray;

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

pub fn right_shift_vec(shift: usize, vec: Vec<u8>) -> Vec<u8> {
    todo!();
}

pub const fn extract_slice(range: Range<usize>, slice: &[u8]) -> Vec<u8> {
    todo!();
}

#[derive(Debug)]
pub struct MatchTester {
    pattern: Vec<u8>,
    mask: Vec<u8>,
}

impl MatchTester {
    pub fn new(pattern: &str) -> Self {
        let pattern: String = pattern
            .chars()
            .filter(|c| *c == 'x' || *c == '0' || *c == '1')
            .collect();
        let padding: String = (0..(8 - pattern.len() % 8) % 8).map(|_| '0').collect();
        let match_string = format!("{padding}{pattern}");

        println!("{match_string}");

        let mut pattern_result = Vec::new();
        let mut mask_result = Vec::new();

        let mut pat = 0u8;
        let mut mas = 0u8;

        for (i, c) in match_string.chars().enumerate() {
            let (p, m) = match c {
                'x' => (0, 0),
                '1' => (1, 1),
                '0' => (0, 1),
                ' ' | '_' => continue,
                _ => unreachable!(),
            };

            pat <<= 1;
            pat |= p;
            mas <<= 1;
            mas |= m;

            if (i + 1) % 8 == 0 {
                pattern_result.push(pat);
                mask_result.push(mas);

                pat = 0;
                mas = 0;
            }
        }

        Self {
            pattern: pattern_result,
            mask: mask_result,
        }
    }
    pub fn test(&self, target: &[u8]) -> bool {
        self.pattern
            .iter()
            .zip(self.mask.iter())
            .zip(target.iter())
            .all(|((p, m), v)| v & m == *p)
    }

    pub fn debug(&self) {
        for p in self.pattern.iter() {
            println!("{:08b}", p);
        }

        println!("----------");

        for m in self.mask.iter() {
            println!("{:08b}", m);
        }
    }
}

pub struct BitPatternMatcher<O> {
    matches: Vec<Box<dyn MatchHelper<Output = Option<O>> + Send + Sync + 'static>>,
}

impl<O> BitPatternMatcher<O> {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }

    pub fn bind<H, A>(&mut self, pattern: &str, handler: H) -> &mut Self
    where
        H: Send + Sync + 'static,
        A: Send + Sync + 'static,
        H: Handler<A, Output = O>,
    {
        self.matches.push(Box::new(Match::new(pattern, handler)));
        self
    }

    pub fn try_match(&self, raw_instr: &[u8]) -> Option<O> {
        for matcher in &self.matches {
            if let Some(result) = matcher.handle(raw_instr) {
                return Some(result);
            }
        }
        None
    }
}

trait MatchHelper {
    type Output;
    fn handle(&self, raw_instr: &[u8]) -> Self::Output;
}

impl<H, A> MatchHelper for Match<H, A>
where
    H: Handler<A>,
{
    type Output = Option<H::Output>;

    fn handle(&self, raw_instr: &[u8]) -> Self::Output {
        self.handle(raw_instr)
    }
}

pub struct Match<H, A> {
    pattern: MatchTester,
    handler: H,
    __p: PhantomData<A>,
}

impl<H, A> Match<H, A>
where
    H: Handler<A>,
{
    pub fn new<'s>(pattern: &'s str, handler: H) -> Self {
        let pattern = MatchTester::new(pattern);

        Self {
            pattern,
            handler,
            __p: PhantomData,
        }
    }

    pub fn handle(&self, raw_instr: &[u8]) -> Option<H::Output> {
        if self.pattern.test(raw_instr) {
            Some(self.handler.handle(raw_instr))
        } else {
            None
        }
    }
}

pub trait ExtractFromBytes: Copy {
    fn extract(from: &[u8]) -> Self;
}

impl<T> ExtractFromBytes for T
where
    T: FixedBytesArray + Copy,
{
    fn extract(from: &[u8]) -> Self {
        T::from_bytes(from)
    }
}

impl ExtractFromBytes for u8 {
    fn extract(from: &[u8]) -> Self {
        from[0]
    }
}

#[derive(Clone, Copy)]
pub struct Byte<const L: usize, const R: usize>(pub u8);
impl<const L: usize, const R: usize> ExtractFromBytes for Byte<L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(from[0])
    }
}

#[derive(Clone, Copy)]
pub struct Le<T, const L: usize, const R: usize>(pub T);

impl<const L: usize, const R: usize> ExtractFromBytes for Le<u16, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u16::from_le_bytes(from[..2].try_into().unwrap()))
    }
}

impl<const L: usize, const R: usize> ExtractFromBytes for Le<u32, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u32::from_le_bytes(from[..4].try_into().unwrap()))
    }
}

impl<const L: usize, const R: usize> ExtractFromBytes for Le<u64, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u64::from_le_bytes(from[..8].try_into().unwrap()))
    }
}

#[derive(Clone, Copy)]
pub struct Be<T, const L: usize, const R: usize>(pub T);

impl<const L: usize, const R: usize> ExtractFromBytes for Be<u16, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u16::from_be_bytes(from[..2].try_into().unwrap()))
    }
}

impl<const L: usize, const R: usize> ExtractFromBytes for Be<u32, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u32::from_be_bytes(from[..4].try_into().unwrap()))
    }
}

impl<const L: usize, const R: usize> ExtractFromBytes for Be<u64, L, R> {
    fn extract(from: &[u8]) -> Self {
        Self(u64::from_be_bytes(from[..8].try_into().unwrap()))
    }
}

pub trait Handler<T> {
    type Output;

    fn handle(&self, bytes: &[u8]) -> Self::Output;
}

impl<F, O> Handler<()> for F
where
    F: Fn(&[u8]) -> O,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(bytes)
    }
}

impl<F, O, A1> Handler<(O, A1)> for F
where
    F: Fn(&[u8], A1) -> O,
    A1: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(bytes, A1::extract(bytes))
    }
}

impl<F, O, A1, A2> Handler<(O, A1, A2)> for F
where
    F: Fn(&[u8], A1, A2) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(bytes, A1::extract(bytes), A2::extract(bytes))
    }
}

impl<F, O, A1, A2, A3> Handler<(O, A1, A2, A3)> for F
where
    F: Fn(&[u8], A1, A2, A3) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4> Handler<(O, A1, A2, A3, A4)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5> Handler<(O, A1, A2, A3, A4, A5)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6> Handler<(O, A1, A2, A3, A4, A5, A6)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7> Handler<(O, A1, A2, A3, A4, A5, A6, A7)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8> Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9> Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8, A9)>
    for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
    A9: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
            A9::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10>
    Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
    A9: ExtractFromBytes,
    A10: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
            A9::extract(bytes),
            A10::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11>
    Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
    A9: ExtractFromBytes,
    A10: ExtractFromBytes,
    A11: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
            A9::extract(bytes),
            A10::extract(bytes),
            A11::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12>
    Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
    A9: ExtractFromBytes,
    A10: ExtractFromBytes,
    A11: ExtractFromBytes,
    A12: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
            A9::extract(bytes),
            A10::extract(bytes),
            A11::extract(bytes),
            A12::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13>
    Handler<(O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13) -> O,
    A1: ExtractFromBytes,
    A2: ExtractFromBytes,
    A3: ExtractFromBytes,
    A4: ExtractFromBytes,
    A5: ExtractFromBytes,
    A6: ExtractFromBytes,
    A7: ExtractFromBytes,
    A8: ExtractFromBytes,
    A9: ExtractFromBytes,
    A10: ExtractFromBytes,
    A11: ExtractFromBytes,
    A12: ExtractFromBytes,
    A13: ExtractFromBytes,
{
    type Output = O;

    fn handle(&self, bytes: &[u8]) -> Self::Output {
        (self)(
            bytes,
            A1::extract(bytes),
            A2::extract(bytes),
            A3::extract(bytes),
            A4::extract(bytes),
            A5::extract(bytes),
            A6::extract(bytes),
            A7::extract(bytes),
            A8::extract(bytes),
            A9::extract(bytes),
            A10::extract(bytes),
            A11::extract(bytes),
            A12::extract(bytes),
            A13::extract(bytes),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_tester1() {
        let tester = MatchTester::new("x1x1_00xx");

        assert_eq!(tester.test(&vec![0b01010011u8]), true);
        assert_eq!(tester.test(&vec![0b1010_1100u8]), false);
    }

    #[test]
    fn match_tester2() {
        let tester = MatchTester::new("1111_x1x1_00xx_000x");

        assert_eq!(tester.test(&vec![0b1111_0101, 0b0011_0001]), true);
        assert_eq!(tester.test(&vec![0b1111_0111, 0b0010_0001]), true);
        assert_eq!(tester.test(&vec![0b1111_1111, 0b0011_0000]), true);
        assert_eq!(tester.test(&vec![0b1111_1111, 0b0011_0001]), true);

        assert_eq!(tester.test(&vec![0b1110_0101, 0b0011_0001]), false);
        assert_eq!(tester.test(&vec![0b1110_1111, 0b0011_0001]), false);
        assert_eq!(tester.test(&vec![0b1110_1111, 0b0011_0000]), false);
        assert_eq!(tester.test(&vec![0b1110_1111, 0b1000_0000]), false);
    }
}
