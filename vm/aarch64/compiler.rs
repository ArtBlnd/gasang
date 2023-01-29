use crate::instruction::*;
use crate::register::RegId;
use crate::Interrupt;

use machineinstr::aarch64::*;

use std::iter::Iterator;

use smallvec::SmallVec;

pub struct AArch64Compiler {
    gpr_registers: [RegId; 32],
    fpr_registers: [RegId; 32],

    pstate_reg: RegId,
}

impl AArch64Compiler {
    pub fn new(gpr_registers: [RegId; 32], fpr_registers: [RegId; 32], pstate_reg: RegId) -> Self {
        Self {
            gpr_registers,
            fpr_registers,
            pstate_reg,
        }
    }

    pub fn gpr(&self, reg: u8) -> RegId {
        self.gpr_registers[reg as usize]
    }

    pub fn fpr(&self, reg: u8) -> RegId {
        self.fpr_registers[reg as usize]
    }

    pub fn compile_instr(
        &self,
        orgn_size: u8, // original instruction size
        prev_size: u8, // previous instruction size
        instr: AArch64Instr,
    ) -> SmallVec<[u8; 8]> {
        let mut out = SmallVec::new();

        match instr {
            AArch64Instr::MovzVar32(operand) | AArch64Instr::MovzVar64(operand) => {
                let op = Reg1U16 {
                    op1: self.gpr(operand.rd),
                    imm16: operand.imm16,
                }
                .build(IROP_MOV_16CST2REG);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Nop => {
                let curr_size = 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
            }

            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let op1 = Reg1 {
                    op1: self.gpr(operand.rd),
                }
                .build(IROP_MOV_IPR2REG);

                let op2 = Reg1I32 {
                    op1: self.gpr(operand.rd),
                    imm32: imm as i32,
                }
                .build(IROP_IADD_CST32);

                let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
            }

            AArch64Instr::OrrShiftedReg64(operand) => {
                let rm = self.gpr(operand.rm);
                let rn = self.gpr(operand.rn);
                let rd = self.gpr(operand.rd);

                if operand.imm6 == 0 && operand.shift == 0 && operand.rn == 0b11111 {
                    let op = Reg2 { op1: rm, op2: rd }.build(IROP_MOV_REG2REG);

                    let curr_size = 2 + op.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op);
                } else {
                    let i1 = match decode_shift(operand.shift) {
                        ShiftType::LSL => IROP_LLEFT_SHIFT_IMM8,
                        ShiftType::LSR => IROP_LRIGHT_SHIFT_IMM8,
                        ShiftType::ASR => IROP_ARIGHT_SHIFT_IMM8,
                        ShiftType::ROR => IROP_ROTATE_IMM8,
                    };

                    let op1 = Reg2U8 {
                        op1: rm,
                        op2: rd,
                        imm8: operand.imm6,
                    }
                    .build(i1);

                    let op2 = Reg3 {
                        op1: rd,
                        op2: rd,
                        op3: rn,
                    }
                    .build(IROP_OR_REG3);

                    let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op1);
                    out.extend_from_slice(&op2);
                }
            }

            AArch64Instr::Svc(operand) => {
                let op = U16 {
                    imm16: operand.imm16,
                }
                .build(IROP_SVC);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Brk(operand) => {
                let op = U16 {
                    imm16: operand.imm16,
                }
                .build(IROP_BRK);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            _ => unimplemented!("unknown instruction: {:?}", instr),
        }

        out
    }
}

enum ShiftType {
    LSL, // Logical shift left
    LSR, // Logical shift right
    ASR, // Arithmetic shift right
    ROR, // Rotate right
}

const fn decode_shift(shift: u8) -> ShiftType {
    match shift {
        0b00 => ShiftType::LSL,
        0b01 => ShiftType::LSR,
        0b10 => ShiftType::ASR,
        0b11 => ShiftType::ROR,
        _ => unreachable!(),
    }
}

const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}
