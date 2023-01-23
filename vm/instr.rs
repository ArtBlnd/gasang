use crate::mmu::MemoryFrame;
use crate::register::*;
use crate::{Interrupt, Vm, VmContext};

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub struct VmInstr {
    pub op: VmInstrOp,
    pub size: u8,
}

#[derive(Debug, Clone)]
pub enum VmInstrOp {
    // =========================
    // MOVE INSTRUCTIONS

    // move register to register
    MoveReg2Reg {
        size: u8,
        src: RegId,
        dst: RegId,
    },

    // Move register to memory
    MoveReg2Mem {
        size: u8,
        src: RegId,
        dst: usize,
    },

    // Move memory to register
    MoveMem2Reg {
        size: u8,
        src: usize,
        dst: RegId,
    },

    // Move constant to register
    MoveCst2Reg {
        size: u8,
        src: u64,
        dst: RegId,
    },

    MoveIpr2Reg {
        size: u8,
        dst: RegId,
    },

    // =========================
    // CONTROL FLOW INSTRUCTIONS
    JumpIpr {
        dst: RegId,
    },

    JumpIpv {
        dst: usize,
        dst_ipr: u64,
    },

    // =========================
    // ARITHMETIC INSTRUCTIONS

    // Add constant to register
    AddCst {
        size: u8,
        src: RegId,
        dst: RegId,
        value: u64,
    },

    // Bitwise or constant
    OrCst {
        size: u8,
        src: RegId,
        dst: RegId,
        value: u64,
    },
    // Bitwise or register
    OrReg {
        size: u8,
        src: RegId,
        dst: RegId,
        value: RegId,
    },

    // Right shift constant
    LSRCst {
        src: RegId,
        dst: RegId,
        shift: u8,
    },

    // Left shift constant
    LSLCst {
        src: RegId,
        dst: RegId,
        shift: u8,
    },

    // Right routate constant
    RORCst {
        src: RegId,
        dst: RegId,
        shift: u8,
    },

    // Arithmetic shift right constant
    ASRCst {
        src: RegId,
        dst: RegId,
        shift: u8,
    },

    // =========================
    // SPEICAL INSTRUCTIONS
    Interrupt {
        interrupt: Interrupt,
    },
}

impl Display for VmInstrOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            VmInstrOp::MoveReg2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstrOp::MoveReg2Mem { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstrOp::MoveMem2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstrOp::MoveCst2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstrOp::MoveIpr2Reg { size, dst } => {
                write!(f, "mov{} ipr, {}", size, dst)
            }

            VmInstrOp::JumpIpr { dst } => {
                write!(f, "jmp_ipr {}", dst)
            }
            VmInstrOp::JumpIpv { dst, dst_ipr } => {
                write!(f, "jmp_ipv {}:{}", dst, dst_ipr)
            }

            VmInstrOp::AddCst {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "add{} {}, {}, {}", size, src, dst, value)
            }
            VmInstrOp::OrCst {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "or{} {}, {}, {}", size, src, dst, value)
            }
            VmInstrOp::OrReg {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "or{} {}, {}, {}", size, src, dst, value)
            }
            VmInstrOp::LSRCst { src, dst, shift } => {
                write!(f, "lsr {}, {}, {}", src, dst, shift)
            }
            VmInstrOp::LSLCst { src, dst, shift } => {
                write!(f, "lsl {}, {}, {}", src, dst, shift)
            }
            VmInstrOp::RORCst { src, dst, shift } => {
                write!(f, "ror {}, {}, {}", src, dst, shift)
            }
            VmInstrOp::ASRCst { src, dst, shift } => {
                write!(f, "asr {}, {}, {}", src, dst, shift)
            }
            VmInstrOp::Interrupt { interrupt } => {
                write!(f, "int {}", interrupt)
            }
        }
    }
}

impl VmInstrOp {
    pub unsafe fn execute(&self, vm: &mut Vm, vm_ctx: &VmContext) -> Result<(), Interrupt> {
        match self {
            Self::MoveCst2Reg { src, dst, .. } => {
                vm.gpr(*dst).set(*src);
            }
            Self::MoveReg2Mem { size, src, dst } => {
                let mut frame = vm.mem(*dst)?;

                let src = vm.gpr(*src).get();
                match size {
                    1 => frame.write_u8(src as u8)?,
                    2 => frame.write_u16(src as u16)?,
                    4 => frame.write_u32(src as u32)?,
                    8 => frame.write_u64(src as u64)?,
                    _ => unreachable!("bad size: {}", size),
                };
            }
            Self::MoveMem2Reg { size, src, dst } => {
                let mut frame = vm.mem(*src)?;

                let dst = vm.gpr(*dst);
                match size {
                    1 => dst.set(frame.read_u8()? as u64),
                    2 => dst.set(frame.read_u16()? as u64),
                    4 => dst.set(frame.read_u32()? as u64),
                    8 => dst.set(frame.read_u64()? as u64),
                    _ => unreachable!("bad size: {}", size),
                };
            }
            Self::MoveReg2Reg { size, src, dst } => {
                let src = vm.gpr(*src).get();
                let dst = vm.gpr(*dst);

                dst.set(src & make_mask(*size));
            }

            Self::MoveIpr2Reg { size, dst } => {
                let ipr = vm.ipr();
                let dst = vm.gpr(*dst);

                dst.set(ipr & make_mask(*size));
            }

            Self::AddCst {
                size,
                src,
                dst,
                value,
            } => {
                let src = vm.gpr(*src).get();
                let dst = vm.gpr(*dst);

                let result = src.wrapping_add(*value);

                dst.set(result & make_mask(*size));

                // TODO: Handle carry and overflow.
            }

            Self::Interrupt { interrupt } => {
                return Err(interrupt.clone());
            }

            Self::LSLCst { src, dst, shift } => {
                let src = vm.gpr(*src).get();
                let dst = vm.gpr(*dst);

                let result = src << shift;
                dst.set(result);
            }

            Self::OrReg {
                size,
                src,
                dst,
                value,
            } => {
                let src = vm.gpr(*src).get();
                let value = vm.gpr(*value).get();
                let dst = vm.gpr(*dst);

                let result = src | value;
                dst.set(result & make_mask(*size));
            }
            _ => unimplemented!("Instruction not implemented {}", self),
        }

        Ok(())
    }
}

const fn make_mask(size: u8) -> u64 {
    match size {
        1 => 0xff,
        2 => 0xffff,
        4 => 0xffffffff,
        8 => 0xffffffffffffffff,
        _ => panic!("Invalid size"),
    }
}
