use crate::register::*;
use crate::{Interrupt, Vm};

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub struct VmInstr {
    pub op: VmInstrOp,
    pub size: u8,
}

#[derive(Debug, Clone)]
pub enum VmInstrOp {
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
    pub unsafe fn execute(&self, state: &mut Vm) -> Result<(), Interrupt> {
        match self {
            Self::MoveCst2Reg { src, dst, .. } => {
                state.gpr(*dst).set(*src);
            }
            Self::MoveReg2Mem { size, src, dst } => {
                let mut frame = state.mem(*dst)?;

                let src = state.gpr(*src).get();
                match size {
                    1 => frame.write_u8(src as u8)?,
                    2 => frame.write_u16(src as u16)?,
                    4 => frame.write_u32(src as u32)?,
                    8 => frame.write_u64(src as u64)?,
                    _ => unreachable!("bad size: {}", size),
                };
            }
            Self::MoveReg2Reg { size, src, dst } => {
                let src = state.gpr(*src).get();
                let dst = state.gpr(*dst);

                dst.set(src & make_mask(*size));
            }

            Self::AddCst {
                size,
                src,
                dst,
                value,
            } => {
                let src = state.gpr(*src).get();
                let dst = state.gpr(*dst);

                let result = src.wrapping_add(*value);

                dst.set(result & make_mask(*size));

                // TODO: Handle carry and overflow.
            }

            Self::Interrupt { interrupt } => {
                return Err(interrupt.clone());
            }

            Self::LSLCst { src, dst, shift } => {
                let src = state.gpr(*src).get();
                let dst = state.gpr(*dst);

                let result = src << shift;
                dst.set(result);
            }

            Self::OrReg {
                size,
                src,
                dst,
                value,
            } => {
                let src = state.gpr(*src).get();
                let value = state.gpr(*value).get();
                let dst = state.gpr(*dst);

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
