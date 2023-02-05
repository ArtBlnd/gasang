pub mod aarch64;
pub mod instr;
pub mod utils;

mod bit_patterns;

use std::iter::Iterator;
use utility::*;

use crate::instr::NativeInstr;

pub trait MachineInstrParserRule: Clone {
    type MachineInstr;

    fn parse<I>(&mut self, buf: &mut BitReader<I>) -> Option<NativeInstr<Self::MachineInstr>>
    where
        I: Iterator<Item = u8>;
}

pub struct MachineInstParser<I, R> {
    buf: BitReader<I>,
    rule: R,
}

impl<I, R> MachineInstParser<I, R> {
    pub fn new(buf: BitReader<I>, rule: R) -> Self {
        Self { buf, rule }
    }
}

impl<I, R> Iterator for MachineInstParser<I, R>
where
    I: Iterator<Item = u8>,
    R: MachineInstrParserRule,
{
    type Item = NativeInstr<R::MachineInstr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rule.parse(&mut self.buf)
    }
}
