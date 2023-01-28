use crate::instruction::*;

use std::fmt::{Display, Formatter, Result as FmtResult};

pub fn print_irop(op: u8, offset: &mut usize, instr: &VmIr<'_>) -> String {
    match op {
        IROP_UADD_REG3 => {
            let op1 = instr.reg3(offset);
            format!("uadd {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }
        IROP_USUB_REG3 => {
            let op1 = instr.reg3(offset);
            format!("usub {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }
        IROP_UMUL_REG3 => {
            let op1 = instr.reg3(offset);
            format!("umul {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }
        IROP_UDIV_REG3 => {
            let op1 = instr.reg3(offset);
            format!("udiv {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }

        IROP_UADD_CST8 => {
            let op1 = instr.reg1u8(offset);
            format!("uadd {}, {}", op1.op1, op1.imm8)
        }

        IROP_USUB_CST8 => {
            let op1 = instr.reg1u8(offset);
            format!("usub {}, {}", op1.op1, op1.imm8)
        }

        IROP_UMUL_CST8 => {
            let op1 = instr.reg1u8(offset);
            format!("umul {}, {}", op1.op1, op1.imm8)
        }

        IROP_UDIV_CST8 => {
            let op1 = instr.reg1u8(offset);
            format!("udiv {}, {}", op1.op1, op1.imm8)
        }

        IROP_UADD_CST32 => {
            let op1 = instr.reg1u32(offset);
            format!("uadd {}, {}", op1.op1, op1.imm32)
        }

        IROP_IADD_CST32 => {
            let op1 = instr.reg1i32(offset);
            format!("iadd {}, {}", op1.op1, op1.imm32)
        }

        IROP_USUB_CST32 => {
            let op1 = instr.reg1u32(offset);
            format!("usub {}, {}", op1.op1, op1.imm32)
        }

        IROP_UMUL_CST32 => {
            let op1 = instr.reg1u32(offset);
            format!("umul {}, {}", op1.op1, op1.imm32)
        }

        IROP_UDIV_CST32 => {
            let op1 = instr.reg1u32(offset);
            format!("udiv {}, {}", op1.op1, op1.imm32)
        }

        IROP_UADD_CST64 => {
            let op1 = instr.reg1u64(offset);
            format!("umul {}, {}", op1.op1, op1.imm64)
        }

        IROP_USUB_CST64 => {
            let op1 = instr.reg1u64(offset);
            format!("usub {}, {}", op1.op1, op1.imm64)
        }

        IROP_UMUL_CST64 => {
            let op1 = instr.reg1u64(offset);
            format!("umul {}, {}", op1.op1, op1.imm64)
        }

        IROP_UDIV_CST64 => {
            let op1 = instr.reg1u64(offset);
            format!("udiv {}, {}", op1.op1, op1.imm64)
        }

        IROP_OR_REG3 => {
            let op1 = instr.reg3(offset);
            format!("or {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }
        IROP_AND_REG3 => {
            let op1 = instr.reg3(offset);
            format!("and {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }

        IROP_XOR_REG3 => {
            let op1 = instr.reg3(offset);
            format!("xor {}, {}, {}", op1.op1, op1.op2, op1.op3)
        }

        IROP_LLEFT_SHIFT_IMM8 => {
            let op1 = instr.reg2u8(offset);
            format!("lshl {}, {} {}", op1.op1, op1.op2, op1.imm8)
        }
        IROP_LRIGHT_SHIFT_IMM8 => {
            let op1 = instr.reg2u8(offset);
            format!("lshr {}, {} {}", op1.op1, op1.op2, op1.imm8)
        }

        IROP_ROTATE_IMM8 => {
            let op1 = instr.reg2u8(offset);
            format!("rot {}, {} {}", op1.op1, op1.op2, op1.imm8)
        }

        IROP_ARIGHT_SHIFT_IMM8 => {
            let op1 = instr.reg2u8(offset);
            format!("ashr {}, {}", op1.op1, op1.imm8)
        }

        IROP_MOV_REG2MEM_REG => {
            let op1 = instr.reg2(offset);
            format!("mov {}, *({})", op1.op1, op1.op2)
        }

        IROP_MOV_REG2MEM_CST => {
            let op1 = instr.reg1u32(offset);
            format!("mov {}, *({})", op1.op1, op1.imm32)
        }

        IROP_MOV_64CST2REG => {
            let op1 = instr.reg1u64(offset);
            format!("mov {}, {}", op1.imm64, op1.op1)
        }

        IROP_MOV_16CST2REG => {
            let op1 = instr.reg1u16(offset);
            format!("mov {}, {}", op1.imm16, op1.op1)
        }

        IROP_MOV_IPR2REG => {
            let op1 = instr.reg1(offset);
            format!("mov irp, {}", op1.op1)
        }

        IROP_MOV_REG2REG => {
            let op1 = instr.reg2(offset);
            format!("mov {}, {}", op1.op1, op1.op2)
        }

        IROP_SVC => {
            let op1 = instr.reg1u8(offset);
            format!("svc {}", op1.imm8)
        }

        IROP_BRK => {
            let op1 = instr.u16(offset);
            format!("brk {}", op1.imm16)
        }

        IROP_NOP => "nop".to_string(),

        _ => {
            format!("unknown opcode: {:02x}", op)
        }
    }
}
