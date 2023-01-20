use crate::{RegId, Interrupt, VmInstr};
use machineinstr::aarch64::*;

use std::iter::Iterator;

use smallvec::SmallVec;

// Program counter register.
const PC_REG: RegId = RegId(32);

pub fn aarch64_translate(instr: AArch64Instr) -> impl Iterator<Item = VmInstr> {
    let mut instrs = SmallVec::<[VmInstr; 2]>::new();


    let mut is_branch_generated = false;
    match instr {
        AArch64Instr::Nop => {}
        AArch64Instr::Brk(ExceptionGen { imm16, .. }) => {
            instrs.push(VmInstr::Interrupt { interrupt: Interrupt::DebugBreakpoint(imm16 as usize) });
        }

        AArch64Instr::Svc(ExceptionGen { imm16, ..}) => {
            instrs.push(VmInstr::Interrupt { interrupt: Interrupt::SystemCall(imm16 as usize) });
        }

        AArch64Instr::MovzVar64(Imm16Rd { imm16, rd }) => {
            instrs.push(VmInstr::MoveCst2Reg {
                size: 8,
                src: imm16 as u64,
                dst: rd.into(),
            });
        }

        AArch64Instr::MovzVar32(Imm16Rd { imm16, rd }) => {
            instrs.push(VmInstr::MoveCst2Reg {
                size: 4,
                src: imm16 as u64,
                dst: rd.into(),
            });
        }

        AArch64Instr::Adr(PcRelAddressing { immlo, immhi, rd }) => {
            let imm = (immhi as u64) << 2 | (immlo as u64);
            instrs.push(VmInstr::AddCst {
                size: 8,
                src: rd.into(),
                dst: PC_REG,
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
            let rm: RegId = rm.into();
            let rn: RegId = rn.into();
            let rd: RegId = rd.into();

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
            src: PC_REG,
            dst: PC_REG,
            value: 4,
        });
    }

    instrs.into_iter()
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
