mod operands;
pub use operands::*;
mod instructions;
pub use instructions::*;
mod executer;
pub use executer::*;
mod printer;
pub use printer::*;

use std::fmt::{Display, Formatter, Result as FmtResult};
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

impl<'r> Display for VmIr<'r> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let real_size = self.real_size();
        let curr_size = self.curr_size();

        f.write_fmt(format_args!("{:02x}:{:02x}", real_size, curr_size))?;
        let mut offset = 2;

        let mut l = f.debug_list();
        while let Some(opcode) = self.opcode(&mut offset) {
            l.entry(&print_irop(opcode, &mut offset, self));
        }

        l.finish()?;

        Ok(())
    }
}

impl<'r> VmIr<'r> {
    pub fn from_ref(slice: &'r [u8]) -> Self {
        Self::Ref(slice)
    }

    pub fn real_size(&self) -> u8 {
        (self.get(0).unwrap() & 0b1111_0000) >> 4
    }

    pub fn curr_size(&self) -> u8 {
        ((self.get(0).unwrap() & 0b0000_1111) << 2) | ((self.get(1).unwrap() & 0b1100_0000) >> 6)
    }

    pub fn prev_size(&self) -> u8 {
        self.get(1).unwrap() & 0b0011_1111
    }

    pub fn opcode(&self, offset: &mut usize) -> Option<u8> {
        if *offset >= self.curr_size() as usize {
            return None;
        }

        let v = self.get(*offset).cloned();

        *offset += 1;
        v
    }

    pub fn reg1(&self, offset: &mut usize) -> Reg1 {
        let v = Reg1 {
            op1: RegId(*self.get(*offset).unwrap()),
        };

        *offset += 1;
        v
    }

    pub fn reg2(&self, offset: &mut usize) -> Reg2 {
        let v = Reg2 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
        };

        *offset += 2;
        v
    }

    pub fn reg3(&self, offset: &mut usize) -> Reg3 {
        let v = Reg3 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            op3: RegId(*self.get(2 + *offset).unwrap()),
        };

        *offset += 3;
        v
    }

    pub fn reg1u8(&self, offset: &mut usize) -> Reg1U8 {
        let v = Reg1U8 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm8: *self.get(1 + *offset).unwrap(),
        };

        *offset += 2;
        v
    }

    pub fn reg2u8(&self, offset: &mut usize) -> Reg2U8 {
        let v = Reg2U8 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm8: *self.get(2 + *offset).unwrap(),
        };

        *offset += 3;
        v
    }

    pub fn reg1u32(&self, offset: &mut usize) -> Reg1U32 {
        let v = Reg1U32 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm32: u32::from_le_bytes([
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
            ]),
        };

        *offset += 5;

        v
    }

    pub fn reg1u64(&self, offset: &mut usize) -> Reg1U64 {
        let v = Reg1U64 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm64: u64::from_le_bytes([
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
            ]),
        };

        *offset += 9;
        v
    }

    pub fn reg1u16(&self, offset: &mut usize) -> Reg1U16 {
        let v = Reg1U16 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm16: u16::from_le_bytes([
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
            ]),
        };

        *offset += 3;
        v
    }

    pub fn u16(&self, offset: &mut usize) -> U16 {
        let v = U16 {
            imm16: u16::from_le_bytes([
                *self.get(*offset).unwrap(),
                *self.get(1 + *offset).unwrap(),
            ]),
        };

        *offset += 2;
        v
    }

    pub fn reg2i64(&self, offset: &mut usize) -> Reg2I64 {
        let v = Reg2I64 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm64: i64::from_le_bytes([
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
                *self.get(9 + *offset).unwrap(),
            ]),
        };

        *offset += 10;
        v
    }

    pub fn reg1i64(&self, offset: &mut usize) -> Reg1I64 {
        let v = Reg1I64 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm64: i64::from_le_bytes([
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
                *self.get(6 + *offset).unwrap(),
                *self.get(7 + *offset).unwrap(),
                *self.get(8 + *offset).unwrap(),
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
    let a1 = orgn_size << 4 | ((curr_size & 0b0011_1100) >> 2);
    let a2 = ((curr_size & 0b0000_0011) << 6) | prev_size;

    out.push(a1);
    out.push(a2);
}
