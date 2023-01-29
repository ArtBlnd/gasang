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

        let mut p = InstrPrinter::new();
        self.visit(&mut p);
        f.debug_list().entries(p.into_inner()).finish()?;

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

    pub fn visit<V>(&self, visitor: &mut V)
    where
        V: InstrVisitor,
    {
        let mut offset = 2;
        while let Some(opcode) = self.opcode(&mut offset) {
            match opcode {
                IROP_MOV_IPR2REG | IROP_PUSH_REG | IROP_POP_REG => {
                    visitor.visit_reg1(opcode, self.reg1(&mut offset))
                }

                IROP_MOV_REG2MEM_REG | IROP_MOV_REG2REG => {
                    visitor.visit_reg2(opcode, self.reg2(&mut offset))
                }

                IROP_UADD_REG3 | IROP_USUB_REG3 | IROP_UMUL_REG3 | IROP_UDIV_REG3
                | IROP_OR_REG3 | IROP_AND_REG3 | IROP_XOR_REG3 => {
                    visitor.visit_reg3(opcode, self.reg3(&mut offset))
                }

                IROP_UADD_CST8 | IROP_USUB_CST8 | IROP_UMUL_CST8 | IROP_UDIV_CST8 => {
                    visitor.visit_reg1u8(opcode, self.reg1u8(&mut offset))
                }

                IROP_LLEFT_SHIFT_IMM8
                | IROP_LRIGHT_SHIFT_IMM8
                | IROP_ROTATE_IMM8
                | IROP_ARIGHT_SHIFT_IMM8 => visitor.visit_reg2u8(opcode, self.reg2u8(&mut offset)),

                IROP_UADD_CST32 | IROP_USUB_CST32 | IROP_UMUL_CST32 | IROP_UDIV_CST32
                | IROP_MOV_REG2MEM_CST => visitor.visit_reg1u32(opcode, self.reg1u32(&mut offset)),

                IROP_IADD_CST32 | IROP_ISUB_CST32 | IROP_IMUL_CST32 | IROP_IDIV_CST32 => {
                    visitor.visit_reg1i32(opcode, self.reg1i32(&mut offset))
                }

                IROP_UADD_CST64 | IROP_USUB_CST64 | IROP_UMUL_CST64 | IROP_UDIV_CST64 => {
                    visitor.visit_reg1u64(opcode, self.reg1u64(&mut offset))
                }

                IROP_MOV_16CST2REG => visitor.visit_reg1u16(opcode, self.reg1u16(&mut offset)),

                IROP_SVC | IROP_BRK => visitor.visit_u16(opcode, self.u16(&mut offset)),

                IROP_NOP => visitor.visit_no_operand(opcode),
                _ => unimplemented!("opcode: {:02x}", opcode),
            }
        }
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

    pub fn reg1i32(&self, offset: &mut usize) -> Reg1I32 {
        let v = Reg1I32 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm32: i32::from_le_bytes([
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

pub trait InstrVisitor {
    fn visit_no_operand(&mut self, op: u8) {}
    fn visit_reg1(&mut self, op: u8, operand: Reg1) {}
    fn visit_reg2(&mut self, op: u8, operand: Reg2) {}
    fn visit_reg3(&mut self, op: u8, operand: Reg3) {}
    fn visit_reg1u8(&mut self, op: u8, operand: Reg1U8) {}
    fn visit_reg2u8(&mut self, op: u8, operand: Reg2U8) {}
    fn visit_reg1u32(&mut self, op: u8, operand: Reg1U32) {}
    fn visit_reg1i32(&mut self, op: u8, operand: Reg1I32) {}
    fn visit_reg1u64(&mut self, op: u8, operand: Reg1U64) {}
    fn visit_reg1u16(&mut self, op: u8, operand: Reg1U16) {}
    fn visit_u16(&mut self, op: u8, operand: U16) {}
    fn visit_reg2i64(&mut self, op: u8, operand: Reg2I64) {}
    fn visit_reg1i64(&mut self, op: u8, operand: Reg1I64) {}
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
