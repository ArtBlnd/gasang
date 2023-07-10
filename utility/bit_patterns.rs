use std::marker::PhantomData;

use crate::FixedBytesArray;

pub fn ones_u8(len: usize) -> u8 {
    assert!(len <= 8);
    (0..len).fold(0u8, |state, _| {
        let state = state << 1;
        state | 1
    })
}

#[derive(Debug)]
pub struct MatchTester {
    pattern_and_mask: Vec<(u8, u8)>,
}

impl MatchTester {
    /// Create a new `MatchTester` from a pattern string.
    ///
    /// If the pattern is not a multiple of 8, which is not aligned. this function will panic.
    pub fn new(pattern: &str) -> Self {
        let mut pattern_and_mask = Vec::new();

        fn is_pattern_char(c: &char) -> bool {
            *c == 'x' || *c == '0' || *c == '1'
        }

        let pattern = pattern.chars().filter(is_pattern_char).collect::<String>();
        assert!(pattern.len() % 8 == 0);

        let mut pat = 0u8;
        let mut mas = 0u8;
        for (i, c) in pattern.chars().enumerate() {
            let (p, m) = match c {
                'x' => (0, 0),
                '1' => (1, 1),
                '0' => (0, 1),
                _ => unreachable!(),
            };

            pat <<= 1;
            pat |= p;
            mas <<= 1;
            mas |= m;

            if (i + 1) % 8 == 0 {
                pattern_and_mask.push((pat, mas));

                pat = 0;
                mas = 0;
            }
        }

        Self { pattern_and_mask }
    }

    pub fn test(&self, target: &[u8]) -> bool {
        self.pattern_and_mask
            .iter()
            .zip(target.iter())
            .all(|((p, m), v)| v & m == *p)
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

    pub fn bind<H, A>(&mut self, pattern: impl AsRef<str>, handler: H) -> &mut Self
    where
        H: Send + Sync + 'static,
        A: Send + Sync + 'static,
        H: Handler<A, Output = O>,
    {
        self.matches
            .push(Box::new(Match::new(pattern.as_ref(), handler)));
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
    pub fn new(pattern: &str, handler: H) -> Self {
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

trait SetBit: Default {
    fn set_bit(&mut self, position: usize, bit: bool);
}

macro_rules! impl_set_bit {
    ($ty:ty) => {
        impl SetBit for $ty {
            fn set_bit(&mut self, position: usize, bit: bool) {
                if bit {
                    *self |= 1 << position;
                } else {
                    *self &= !(1 << position);
                }
            }
        }
    };
}

impl_set_bit!(u8);
impl_set_bit!(u16);
impl_set_bit!(u32);
impl_set_bit!(u64);
impl_set_bit!(u128);
impl_set_bit!(i8);
impl_set_bit!(i16);
impl_set_bit!(i32);
impl_set_bit!(i64);
impl_set_bit!(i128);

#[derive(Clone, Copy)]
pub struct Extract<T, const L: usize, const R: usize>(pub T);
impl<T, const L: usize, const R: usize> ExtractFromBytes for Extract<T, L, R>
where
    T: Clone + Copy + SetBit,
{
    fn extract(from: &[u8]) -> Self {
        let mut result = T::default();
        for (i, v) in (L..R).enumerate() {
            result.set_bit(i, from[v / 8] & (1 << (v % 8)) != 0);
        }

        Self(result)
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

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14>
    Handler<(
        O,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
    )> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14) -> O,
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
    A14: ExtractFromBytes,
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
            A14::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15>
    Handler<(
        O,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
    )> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15) -> O,
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
    A14: ExtractFromBytes,
    A15: ExtractFromBytes,
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
            A14::extract(bytes),
            A15::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16>
    Handler<(
        O,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
    )> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16) -> O,
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
    A14: ExtractFromBytes,
    A15: ExtractFromBytes,
    A16: ExtractFromBytes,
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
            A14::extract(bytes),
            A15::extract(bytes),
            A16::extract(bytes),
        )
    }
}

impl<F, O, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17>
    Handler<(
        O,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
    )> for F
where
    F: Fn(&[u8], A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17) -> O,
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
    A14: ExtractFromBytes,
    A15: ExtractFromBytes,
    A16: ExtractFromBytes,
    A17: ExtractFromBytes,
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
            A14::extract(bytes),
            A15::extract(bytes),
            A16::extract(bytes),
            A17::extract(bytes),
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

    #[test]
    fn test_ones_u8() {
        assert_eq!(ones_u8(0), 0b0);
        assert_eq!(ones_u8(1), 0b1);
        assert_eq!(ones_u8(2), 0b11);
        assert_eq!(ones_u8(3), 0b111);
        assert_eq!(ones_u8(4), 0b1111);
        assert_eq!(ones_u8(5), 0b11111);
        assert_eq!(ones_u8(6), 0b111111);
        assert_eq!(ones_u8(7), 0b1111111);
        assert_eq!(ones_u8(8), 0b11111111);
    }
}
