use crate::instr::VmInstr;
use crate::RegId;
use machineinstr::aarch64::*;

use std::iter::Iterator;

use smallvec::SmallVec;

pub fn aarch64_translate(instr: AArch64Instr) -> impl Iterator<Item = VmInstr> {
    let mut instrs = SmallVec::<[VmInstr; 2]>::new();

    match instr {
        AArch64Instr::MovzVar64(Imm16Rd { imm16, rd }) => {
            instrs.push(VmInstr::MoveCst2Reg {
                size: 8,
                src: imm16 as u64,
                dst: rd.into(),
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
