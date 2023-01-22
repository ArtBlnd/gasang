use crate::{Interrupt, RegId, VmInstr};
use machineinstr::aarch64::*;

use std::iter::Iterator;

use smallvec::SmallVec;

// Program counter register.
const PC_REG: RegId = RegId(32);
const PSTATE_REG: RegId = RegId(33);

pub struct AArch64Translater {
    pub gpr_registers: [RegId; 32],
    pub fpr_registers: [RegId; 32],

    // Program counter register.
    pub pc_reg: RegId,
    pub pstate_reg: RegId,
}

impl AArch64Translater {
    pub fn translate(&self, instr: AArch64Instr) -> impl Iterator<Item = VmInstr> {
        let mut instrs = SmallVec::<[VmInstr; 2]>::new();

        let mut is_branch_generated = false;
        match instr {
            AArch64Instr::Nop => {}
            AArch64Instr::Brk(ExceptionGen { imm16, .. }) => {
                instrs.push(VmInstr::Interrupt {
                    interrupt: Interrupt::DebugBreakpoint(imm16 as usize),
                });
            }

            AArch64Instr::Svc(ExceptionGen { imm16, .. }) => {
                instrs.push(VmInstr::Interrupt {
                    interrupt: Interrupt::SystemCall(imm16 as usize),
                });
            }

            AArch64Instr::MovzVar64(Imm16Rd { imm16, rd }) => {
                instrs.push(VmInstr::MoveCst2Reg {
                    size: 8,
                    src: imm16 as u64,
                    dst: self.gpr_registers[rd as usize],
                });
            }

            AArch64Instr::MovzVar32(Imm16Rd { imm16, rd }) => {
                instrs.push(VmInstr::MoveCst2Reg {
                    size: 4,
                    src: imm16 as u64,
                    dst: self.gpr_registers[rd as usize],
                });
            }

            AArch64Instr::Adr(PcRelAddressing { immlo, immhi, rd }) => {
                let imm = sign_extend((immhi as u64) << 2 | (immlo as u64), 20);
                instrs.push(VmInstr::AddCst {
                    size: 8,
                    src: self.pc_reg,
                    dst: self.gpr_registers[rd as usize],
                    value: imm,
                });
            }

            AArch64Instr::OrrShiftedReg64(ShiftRmImm6RnRd {
                shift,
                rm,
                imm6,
                rn,
                rd,
            }) => {
                let rm: RegId = self.gpr_registers[rm as usize];
                let rn: RegId = self.gpr_registers[rn as usize];
                let rd: RegId = self.gpr_registers[rd as usize];

                let i1 = match decode_shift(shift) {
                    ShiftType::LSL => VmInstr::LSLCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::LSR => VmInstr::LSRCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::ASR => VmInstr::ASRCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::ROR => VmInstr::RORCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                };

                let i2 = VmInstr::OrReg {
                    size: 8,
                    src: rd,
                    dst: rd,
                    value: rn,
                };

                instrs.push(i1);
                instrs.push(i2);
            }
            v => todo!("{:?}", v),
        }

        // if branch did not generated, increase PC by 4(= 1 instruction)
        if !is_branch_generated {
            instrs.push(VmInstr::AddCst {
                size: 8,
                src: self.pc_reg,
                dst: self.pc_reg,
                value: 4,
            });
        }

        instrs.into_iter()
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

pub const fn sign_extend(value: u64, size: u8) -> u64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}
