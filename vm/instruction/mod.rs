mod operands;
pub use operands::*;
mod instructions;
pub use instructions::*;
mod executer;
pub use executer::*;

use std::ops::Deref;

use crate::register::RegId;

use smallvec::{Array, SmallVec};

pub enum VmIr<'r> {
    Ref(&'r [u8]),
}

impl<'r> Deref for VmIr<'r> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ref(inst) => inst,
        }
    }
}

impl<'r> VmIr<'r> {
    pub fn from_ref(slice: &'r [u8]) -> Self {
        Self::Ref(slice)
    }

    pub fn real_size(&self) -> u8 {
        self.get(0).unwrap() & 0b1111_0000 >> 4
    }

    pub fn curr_size(&self) -> u8 {
        (self.get(0).unwrap() & 0b0000_1111 << 2) | (self.get(1).unwrap() & 0b1100_0000 >> 6)
    }

    pub fn prev_size(&self) -> u8 {
        self.get(1).unwrap() & 0b0011_1111
    }

    pub fn opcode(&self, offset: &mut usize) -> Option<u8> {
        let v = self.get(2 + *offset).cloned();

        *offset += 1;
        v
    }

    pub fn reg1(&self, offset: &mut usize) -> Reg1 {
        let v = Reg1 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
        };

        *offset += 1;
        v
    }

    pub fn reg2(&self, offset: &mut usize) -> Reg2 {
        let v = Reg2 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            op2: RegId(*self.get(3 + *offset).unwrap()),
        };

        *offset += 2;
        v
    }

    pub fn reg3(&self, offset: &mut usize) -> Reg3 {
        let v = Reg3 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            op2: RegId(*self.get(3 + *offset).unwrap()),
            op3: RegId(*self.get(4 + *offset).unwrap()),
        };

        *offset += 3;
        v
    }

    pub fn reg1u8(&self, offset: &mut usize) -> Reg1U8 {
        let v = Reg1U8 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            imm8: *self.get(3 + *offset).unwrap(),
        };

        *offset += 2;
        v
    }

    pub fn reg2u8(&self, offset: &mut usize) -> Reg2U8 {
        let v = Reg2U8 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            op2: RegId(*self.get(3 + *offset).unwrap()),
            imm8: *self.get(4 + *offset).unwrap(),
        };

        *offset += 3;
        v
    }

    pub fn reg1u32(&self, offset: &mut usize) -> Reg1U32 {
        let v = Reg1U32 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            imm32: u32::from_le_bytes([
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
            ]),
        };

        *offset += 5;

        v
    }

    pub fn reg1u64(&self, offset: &mut usize) -> Reg1U64 {
        let v = Reg1U64 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            imm64: u64::from_le_bytes([
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
                *self.get(9 + *offset).unwrap(),
                *self.get(10 + *offset).unwrap(),
            ]),
        };

        *offset += 9;
        v
    }

    pub fn reg1u16(&self, offset: &mut usize) -> Reg1U16 {
        let v = Reg1U16 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            imm16: u16::from_le_bytes([
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
            ]),
        };

        *offset += 3;
        v
    }

    pub fn u16(&self, offset: &mut usize) -> U16 {
        let v = U16 {
            imm16: u16::from_le_bytes([
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
            ]),
        };

        *offset += 2;
        v
    }

    pub fn reg2i64(&self, offset: &mut usize) -> Reg2I64 {
        let v = Reg2I64 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            op2: RegId(*self.get(3 + *offset).unwrap()),
            imm64: i64::from_le_bytes([
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
                *self.get(9 + *offset).unwrap(),
                *self.get(10 + *offset).unwrap(),
                *self.get(11 + *offset).unwrap(),
            ]),
        };

        *offset += 10;
        v
    }

    pub fn reg1i64(&self, offset: &mut usize) -> Reg1I64 {
        let v = Reg1I64 {
            op1: RegId(*self.get(2 + *offset).unwrap()),
            imm64: i64::from_le_bytes([
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
                *self.get(9 + *offset).unwrap(),
                *self.get(10 + *offset).unwrap(),
            ]),
        };

        *offset += 9;
        v
    }
}

pub fn build_instr_sig<A>(out: &mut SmallVec<A>, orgn_size: u8, curr_size: u8, prev_size: u8)
where
    A: Array<Item = u8>,
{
    let a1 = orgn_size << 4 | (curr_size & 0b0011_1100 >> 2);
    let a2 = (curr_size & 0b0000_0011 << 6) | prev_size;

    out.push(a1);
    out.push(a2);
}
