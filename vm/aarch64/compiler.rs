use crate::instr::VmInstrOp;
use crate::register::RegId;
use crate::Interrupt;
use machineinstr::aarch64::*;

use std::iter::Iterator;

use smallvec::SmallVec;

pub struct AArch64Compiler {
    pub gpr_registers: [RegId; 32],
    pub fpr_registers: [RegId; 32],

    // Program counter register.
    pub pc_reg: RegId,
    pub pstate_reg: RegId,
}

impl AArch64Compiler {
    pub fn compile_instr(&self, instr: AArch64Instr) -> impl Iterator<Item = VmInstrOp> {
        let mut instrs = SmallVec::<[VmInstrOp; 2]>::new();

        match instr {
            AArch64Instr::Nop => {}
            AArch64Instr::Brk(ExceptionGen { imm16, .. }) => {
                instrs.push(VmInstrOp::Interrupt {
                    interrupt: Interrupt::DebugBreakpoint(imm16 as usize),
                });
            }

            AArch64Instr::Svc(ExceptionGen { imm16, .. }) => {
                instrs.push(VmInstrOp::Interrupt {
                    interrupt: Interrupt::SystemCall(imm16 as usize),
                });
            }

            AArch64Instr::MovzVar64(Imm16Rd { imm16, rd }) => {
                instrs.push(VmInstrOp::MoveCst2Reg {
                    size: 8,
                    src: imm16 as u64,
                    dst: self.gpr_registers[rd as usize],
                });
            }

            AArch64Instr::MovzVar32(Imm16Rd { imm16, rd }) => {
                instrs.push(VmInstrOp::MoveCst2Reg {
                    size: 4,
                    src: imm16 as u64,
                    dst: self.gpr_registers[rd as usize],
                });
            }

            AArch64Instr::Adr(PcRelAddressing { immlo, immhi, rd }) => {
                let imm = sign_extend((immhi as u64) << 2 | (immlo as u64), 20);
                let reg = self.gpr_registers[rd as usize];

                instrs.push(VmInstrOp::MoveIpr2Reg { size: 8, dst: reg });
                instrs.push(VmInstrOp::AddCst {
                    size: 8,
                    src: reg,
                    dst: reg,
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
                    ShiftType::LSL => VmInstrOp::LSLCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::LSR => VmInstrOp::LSRCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::ASR => VmInstrOp::ASRCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                    ShiftType::ROR => VmInstrOp::RORCst {
                        src: rm,
                        dst: rd,
                        shift: imm6,
                    },
                };

                let i2 = VmInstrOp::OrReg {
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

const fn sign_extend(value: u64, size: u8) -> u64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}
