use crate::{codegen::Executable, VmState};
use crate::ir::{BlockDestination, Type};

use super::InterpretFunc;

pub struct InterpretExeuctableBlock {
    pub(crate) code: Box<dyn InterpretFunc>,
    pub(crate) code_type: Type,
    pub(crate) code_dest: BlockDestination,
    
}

pub struct CodeBlock {
    code: Vec<InterpretExeuctableBlock>,
    code_sizes: Vec<usize>,
}

impl Executable for CodeBlock {
    unsafe fn execute(&self, vm_state: &mut VmState) {
        for (code, size) in self.code.iter().zip(self.code_sizes.iter()) {
            let result = code.code.execute(vm_state);
            match &code.code_dest {
                BlockDestination::Flags => todo!(),
                BlockDestination::Eip => {
                    // We run code until Eip modification.
                    // if eip modification occurs, VM need to check we are executing eip's code.
                    vm_state.set_eip(result);
                    break;
                },
                BlockDestination::GprRegister(reg_id) => vm_state.gpr_mut(reg_id.clone()).set(result),
                BlockDestination::FprRegister(reg_id) => vm_state.fpr_mut(reg_id.clone()).set(result as f64),
                BlockDestination::Memory(frame) => todo!(),
            }

            vm_state.eip += *size as u64;
        }
    }
}