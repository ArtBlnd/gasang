use gdbstub::arch::Arch;

use crate::{error::DebugError, Cpu};

#[derive(Debug)]
pub struct AArch64RegId(u8);

impl gdbstub::arch::RegId for AArch64RegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<std::num::NonZeroUsize>)> {
        if id <= 31 {
            unsafe {
                Some((
                    Self(id as u8),
                    Some(std::num::NonZeroUsize::new_unchecked(64)),
                ))
            }
        } else {
            None
        }
    }
}

pub struct AArch64;

impl Arch for AArch64 {
    type Usize = u64;
    type Registers = Cpu;
    type BreakpointKind = usize;
    type RegId = AArch64RegId;

    fn single_step_gdb_behavior() -> gdbstub::arch::SingleStepGdbBehavior {
        gdbstub::arch::SingleStepGdbBehavior::Required
    }
}

pub fn serialize_aarch64(cpu: &Cpu) -> Result<Vec<u8>, DebugError> {
    let mut result = Vec::new();

    for name in cpu.arch().gprs() {
        let id = cpu
            .reg_by_name(&name)
            .ok_or(DebugError::InvalidRegName(name.clone()))?;
        let val = cpu.gpr(id).u64();
        let val = val.to_ne_bytes();
        result.extend_from_slice(&val);
    }

    let val = cpu.pc().to_ne_bytes();
    result.extend_from_slice(&val);

    let val = cpu.flag();
    let val = val.to_ne_bytes();
    result.extend_from_slice(&val);

    Ok(result)
}

pub fn deserialize_aarch64(bytes: &[u8], cpu: &mut Cpu) {
    let idxs = (0..bytes.len()).step_by(8);
    let regs = cpu.arch().gprs();

    let iter = idxs.zip(regs);

    for (idx, name) in iter {
        let bytes: [u8; 8] = bytes[idx..idx + 8].try_into().unwrap();
        let val = u64::from_ne_bytes(bytes);

        let id = cpu.reg_by_name(name).unwrap();
        let reg = cpu.gpr_mut(id);

        *reg.u64_mut() = val;
    }

    let len = bytes.len();

    let val: [u8; 8] = bytes[len - 16..len - 8].try_into().unwrap();
    let val = u64::from_ne_bytes(val);
    cpu.set_pc(val);

    let val: [u8; 8] = bytes[len - 8..].try_into().unwrap();
    let val = u64::from_ne_bytes(val);
    cpu.set_flag(val);
}
