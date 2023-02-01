#![allow(unused_variables)]

mod operands;
pub use operands::*;
mod instructions;
pub use instructions::*;
mod executer;
pub use executer::*;
mod printer;
pub use printer::*;

use crate::SlotId;

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
                // 1 REGISTER OPERANDS
                MOV_IPR_REG | PSH_REG | POP_REG | BR_IPR_REG1 => {
                    visitor.visit_reg1(opcode, self.reg1(&mut offset))
                }
                MOV_REG2 | NOT_REG2 => visitor.visit_reg2(opcode, self.reg2(&mut offset)),

                UADD_REG3 | USUB_REG3 | UMUL_REG3 | UDIV_REG3 | OR_REG3 | AND_REG3 | XOR_REG3 => {
                    visitor.visit_reg3(opcode, self.reg3(&mut offset))
                }

                // 1 REGISTE AND 1 IMMEDIATE OPERANDS
                MOV_REG1IMM16 => visitor.visit_reg1imm16(opcode, self.reg1u16(&mut offset)),
                MOV_REG1IMM64 | ULOAD_REG1IMM64 | USTORE_REG1IMM64 | NOT_REG1IMM64 => {
                    visitor.visit_reg1imm64(opcode, self.reg1u64(&mut offset))
                }

                // 2 REGISTER AND 1 IMMEDIATE OPERANDS
                LSHL_REG2IMM8 | LSHR_REG2IMM8 | RROT_REG2IMM8 | ASHR_REG2IMM8 | UADD_REG2IMM8
                | USUB_REG2IMM8 | UMUL_REG2IMM8 | UDIV_REG2IMM8 | MOV_BIT_REG2IMM8 => {
                    visitor.visit_reg2imm8(opcode, self.reg2u8(&mut offset))
                }
                REPL_REG2IMM16 => visitor.visit_reg2imm16(opcode, self.reg2u16(&mut offset)),
                UADD_REG2IMM32 | USUB_REG2IMM32 | UMUL_REG2IMM32 | UDIV_REG2IMM32
                | IADD_REG2IMM32 | ISUB_REG2IMM32 | IMUL_REG2IMM32 | IDIV_REG2IMM32
                | SLOAD_REL_REG2IMM32 | SSTORE_REL_REG2IMM32 | ULOAD_REL_REG2IMM32
                | USTORE_REL_REG2IMM32 => {
                    visitor.visit_reg2imm32(opcode, self.reg2u32(&mut offset))
                }
                UADD_REG2IMM64 | USUB_REG2IMM64 | UMUL_REG2IMM64 | UDIV_REG2IMM64
                | IADD_REG2IMM64 | ISUB_REG2IMM64 | IMUL_REG2IMM64 | IDIV_REG2IMM64
                | OR_REG2IMM64 | AND_REG2IMM64 | XOR_REG2IMM64 => {
                    visitor.visit_reg2imm64(opcode, self.reg2u64(&mut offset))
                }

                // 1 IMMEDIATE OPERANDS
                SVC_IMM16 | BRK_IMM16 => visitor.visit_u16(opcode, self.u16(&mut offset)),
                BR_IPV_IMM32 | BR_IPR_IMM32 | BR_IPR_IMM32_REL => {
                    visitor.visit_u32(opcode, self.u32(&mut offset))
                }

                // NO OPERANDS
                NOP => visitor.visit_no_operand(opcode),

                BR_IPR_IMM32_REL_IF_SLOT_ZERO => {
                    visitor.visit_slot1imm32(opcode, self.slot1imm32(&mut offset))
                }

                LOAD_SLOT_REG | STORE_SLOT_REG | UADD_REG1SLOT1 | USUB_REG1SLOT1
                | UMUL_REG1SLOT1 | UDIV_REG1SLOT1 | OR_REG1SLOT1 | AND_REG1SLOT1
                | XOR_REG1SLOT1 => visitor.visit_reg1slot1(opcode, self.reg1slot1(&mut offset)),

                UADD_SLOT3 | USUB_SLOT3 | UMUL_SLOT3 | UDIV_SLOT3 | OR_SLOT3 | AND_SLOT3
                | XOR_SLOT3 => visitor.visit_slot3(opcode, self.slot3(&mut offset)),

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

    pub fn reg1u8(&self, offset: &mut usize) -> Reg1Imm8 {
        let v = Reg1Imm8 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm8: *self.get(1 + *offset).unwrap(),
        };

        *offset += 2;
        v
    }

    pub fn reg2u8(&self, offset: &mut usize) -> Reg2Imm8 {
        let v = Reg2Imm8 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm8: *self.get(2 + *offset).unwrap(),
        };

        *offset += 3;
        v
    }

    pub fn reg2u16(&self, offset: &mut usize) -> Reg2Imm16 {
        let v = Reg2Imm16 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm16: u16::from_be_bytes([
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
            ]),
        };

        *offset += 4;
        v
    }

    pub fn reg1u32(&self, offset: &mut usize) -> Reg1Imm32 {
        let v = Reg1Imm32 {
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

    pub fn reg1u64(&self, offset: &mut usize) -> Reg1Imm64 {
        let v = Reg1Imm64 {
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

    pub fn reg1u16(&self, offset: &mut usize) -> Reg1Imm16 {
        let v = Reg1Imm16 {
            op1: RegId(*self.get(*offset).unwrap()),
            imm16: u16::from_le_bytes([
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
            ]),
        };

        *offset += 3;
        v
    }

    pub fn reg2u32(&self, offset: &mut usize) -> Reg2Imm32 {
        let v = Reg2Imm32 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm32: u32::from_le_bytes([
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
                *self.get(4 + *offset).unwrap(),
                *self.get(5 + *offset).unwrap(),
            ]),
        };

        *offset += 6;
        v
    }

    pub fn reg2u64(&self, offset: &mut usize) -> Reg2Imm64 {
        let v = Reg2Imm64 {
            op1: RegId(*self.get(*offset).unwrap()),
            op2: RegId(*self.get(1 + *offset).unwrap()),
            imm64: u64::from_le_bytes([
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

    pub fn u16(&self, offset: &mut usize) -> Imm16 {
        let v = Imm16 {
            imm16: u16::from_le_bytes([
                *self.get(*offset).unwrap(),
                *self.get(1 + *offset).unwrap(),
            ]),
        };

        *offset += 2;
        v
    }

    pub fn u32(&self, offset: &mut usize) -> Imm32 {
        let v = Imm32 {
            imm32: u32::from_le_bytes([
                *self.get(*offset).unwrap(),
                *self.get(1 + *offset).unwrap(),
                *self.get(2 + *offset).unwrap(),
                *self.get(3 + *offset).unwrap(),
            ]),
        };

        *offset += 4;
        v
    }

    pub fn reg1slot1(&self, offset: &mut usize) -> Reg1Slot1 {
        let v = Reg1Slot1 {
            op1: RegId(*self.get(*offset).unwrap()),
            slot_id: SlotId(*self.get(1 + *offset).unwrap()),
        };

        *offset += 2;
        v
    }

    pub fn slot1imm32(&self, offset: &mut usize) -> SlotImm32 {
        let v = SlotImm32 {
            slot_id: SlotId(*self.get(*offset).unwrap()),
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

    pub fn slot3(&self, offset: &mut usize) -> Slot3 {
        let v = Slot3 {
            slot1: SlotId(*self.get(*offset).unwrap()),
            slot2: SlotId(*self.get(1 + *offset).unwrap()),
            slot3: SlotId(*self.get(2 + *offset).unwrap()),
        };

        *offset += 3;
        v
    }
}

pub trait InstrVisitor {
    fn visit_no_operand(&mut self, op: u8) {}
    fn visit_reg1(&mut self, op: u8, operand: Reg1) {}
    fn visit_reg2(&mut self, op: u8, operand: Reg2) {}
    fn visit_reg3(&mut self, op: u8, operand: Reg3) {}
    fn visit_reg1imm8(&mut self, op: u8, operand: Reg1Imm8) {}
    fn visit_reg1imm16(&mut self, op: u8, operand: Reg1Imm16) {}
    fn visit_reg1imm32(&mut self, op: u8, operand: Reg1Imm32) {}
    fn visit_reg1imm64(&mut self, op: u8, operand: Reg1Imm64) {}
    fn visit_reg2imm8(&mut self, op: u8, operand: Reg2Imm8) {}
    fn visit_reg2imm16(&mut self, op: u8, operand: Reg2Imm16) {}
    fn visit_reg2imm32(&mut self, op: u8, operand: Reg2Imm32) {}
    fn visit_reg2imm64(&mut self, op: u8, operand: Reg2Imm64) {}
    fn visit_u16(&mut self, op: u8, operand: Imm16) {}
    fn visit_u32(&mut self, op: u8, operand: Imm32) {}

    fn visit_reg1slot1(&mut self, op: u8, operand: Reg1Slot1) {}
    fn visit_slot1imm32(&mut self, op: u8, operand: SlotImm32) {}

    fn visit_slot3(&mut self, op: u8, operand: Slot3) {}
}

pub fn build_instr_sig<A>(out: &mut SmallVec<A>, orgn_size: u8, curr_size: u8, prev_size: u8)
where
    A: Array<Item = u8>,
{
    assert!(curr_size <= 63, "curr_size: {}", curr_size);
    assert!(prev_size <= 63, "prev_size: {}", prev_size);
    let a1 = orgn_size << 4 | ((curr_size & 0b0011_1100) >> 2);
    let a2 = ((curr_size & 0b0000_0011) << 6) | prev_size;

    out.push(a1);
    out.push(a2);
}
