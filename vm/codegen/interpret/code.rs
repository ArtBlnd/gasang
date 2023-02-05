use crate::ir::{BlockDestination, Type};
use crate::{codegen::Executable, VmState};

use super::InterpretFunc;

pub struct InterpretExeuctableBlock {
    pub(crate) code: Box<dyn InterpretFunc>,
    pub(crate) code_type: Type,
    pub(crate) code_dest: BlockDestination,
}

pub struct CodeBlock {
    pub(crate) codes: Vec<InterpretExeuctableBlock>,
    pub(crate) sizes: Vec<usize>,
}

impl Executable for CodeBlock {
    unsafe fn execute(&self, vm_state: &mut VmState) {
        for (code, size) in self.codes.iter().zip(self.sizes.iter()) {
            let result = code.code.execute(vm_state);
            match &code.code_dest {
                BlockDestination::Flags => todo!(),
                BlockDestination::Eip => {
                    // We run code until Eip modification.
                    // if eip modification occurs, VM need to check we are executing eip's code.
                    vm_state.set_eip(result);
                    break;
                }
                BlockDestination::GprRegister(reg_id) => {
                    vm_state.gpr_mut(reg_id.clone()).set(result)
                }
                BlockDestination::FprRegister(reg_id) => {
                    vm_state.fpr_mut(reg_id.clone()).set(f64::from_bits(result))
                }
                BlockDestination::Memory(addr) => {
                    let mut frame = vm_state.mem(*addr);
                    match code.code_type {
                        Type::I8 | Type::U8 => frame.write_u8(result as u8),
                        Type::I16 | Type::U16 => frame.write_u16(result as u16),
                        Type::I32 | Type::U32 | Type::F32 => frame.write_u32(result as u32),
                        Type::I64 | Type::U64 | Type::F64 => frame.write_u64(result as u64),
                        Type::Void => unreachable!(),
                    }
                    .expect("Failed to write memory");
                }
                BlockDestination::None => {}
            }

            vm_state.eip += *size as u64;
        }
    }
}
