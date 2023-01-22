use crate::{FlagId, Interrupt, RegId, VmState};

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub enum VmInstr {
    // move register to register
    MoveReg2Reg {
        size: usize,
        src: RegId,
        dst: RegId,
    },

    // Move register to memory
    MoveReg2Mem {
        size: usize,
        src: usize,
        dst: RegId,
    },

    // Move memory to register
    MoveMem2Reg {
        size: usize,
        src: RegId,
        dst: usize,
    },

    // Move constant to register
    MoveCst2Reg {
        size: usize,
        src: u64,
        dst: RegId,
    },

    // Add constant to register
    AddCst {
        size: usize,
        src: RegId,
        dst: RegId,
        value: u64,
    },

    // Bitwise or constant
    OrCst {
        size: usize,
        src: RegId,
        dst: RegId,
        value: u64,
    },
    // Bitwise or register
    OrReg {
        size: usize,
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

impl Display for VmInstr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            VmInstr::MoveReg2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstr::MoveReg2Mem { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstr::MoveMem2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstr::MoveCst2Reg { size, src, dst } => {
                write!(f, "mov{} {}, {}", size, src, dst)
            }
            VmInstr::AddCst {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "add{} {}, {}, {}", size, src, dst, value)
            }
            VmInstr::OrCst {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "or{} {}, {}, {}", size, src, dst, value)
            }
            VmInstr::OrReg {
                size,
                src,
                dst,
                value,
            } => {
                write!(f, "or{} {}, {}, {}", size, src, dst, value)
            }
            VmInstr::LSRCst { src, dst, shift } => {
                write!(f, "lsr {}, {}, {}", src, dst, shift)
            }
            VmInstr::LSLCst { src, dst, shift } => {
                write!(f, "lsl {}, {}, {}", src, dst, shift)
            }
            VmInstr::RORCst { src, dst, shift } => {
                write!(f, "ror {}, {}, {}", src, dst, shift)
            }
            VmInstr::ASRCst { src, dst, shift } => {
                write!(f, "asr {}, {}, {}", src, dst, shift)
            }
            VmInstr::Interrupt { interrupt } => {
                write!(f, "int {}", interrupt)
            }
        }
    }
}

impl VmInstr {
    pub fn execute(&self, state: &mut VmState) -> Result<(), Interrupt> {
        match self {
            Self::MoveCst2Reg { src, dst, .. } => {
                state.get_gpr_register(*dst).unwrap().set(*src);
            }
            Self::MoveMem2Reg { size, src, dst } => {
                todo!()
            }
            Self::MoveReg2Reg { size, src, dst } => {
                let src = state.get_gpr_register(*src).unwrap().get();
                let dst = state.get_gpr_register(*dst).unwrap();

                dst.set(src & make_mask(*size));
            }

            Self::AddCst {
                size,
                src,
                dst,
                value,
            } => {
                let src = state.get_gpr_register(*src).unwrap().get();
                let dst = state.get_gpr_register(*dst).unwrap();

                let result = src.wrapping_add(*value);

                dst.set(result & make_mask(*size));

                // TODO: Handle carry and overflow.
            }

            Self::Interrupt { interrupt } => {
                return Err(*interrupt);
            }

            Self::LSLCst { src, dst, shift } => {
                let src = state.get_gpr_register(*src).unwrap().get();
                let dst = state.get_gpr_register(*dst).unwrap();

                let result = src << shift;
                dst.set(result);
            }

            Self::OrReg {
                size,
                src,
                dst,
                value,
            } => {
                let src = state.get_gpr_register(*src).unwrap().get();
                let value = state.get_gpr_register(*value).unwrap().get();
                let dst = state.get_gpr_register(*dst).unwrap();

                let result = src | value;
                dst.set(result & make_mask(*size));
            }
            _ => unimplemented!("Instruction not implemented {}", self),
        }

        Ok(())
    }
}

const fn make_mask(size: usize) -> u64 {
    match size {
        1 => 0xff,
        2 => 0xffff,
        4 => 0xffffffff,
        8 => 0xffffffffffffffff,
        _ => panic!("Invalid size"),
    }
}
