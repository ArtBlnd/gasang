use crate::ir::{BlockDestination, Type};
use crate::{codegen::Executable, VmState};

use super::InterpretFunc;

use smallvec::SmallVec;

pub struct InterpretExeuctableBlock {
    pub(crate) code: Box<dyn InterpretFunc>,
    pub(crate) code_type: Type,
    pub(crate) code_dest: BlockDestination,

    pub(crate) restore_flag: bool,
}

pub struct CodeBlock {
    pub(crate) codes: Vec<SmallVec<[InterpretExeuctableBlock; 2]>>,
    pub(crate) sizes: Vec<usize>,
}

impl Executable for CodeBlock {
    unsafe fn execute(&self, vm_state: &mut VmState) {
        if self.codes.is_empty() {
            panic!("empty code block! maybe tried to compile unreadable place?");
        }
        
        'body: for (code, size) in self.codes.iter().zip(self.sizes.iter()) {
            for code in code {
                let flag_backup = vm_state.flag();

                let result = code.code.execute(vm_state);
                match &code.code_dest {
                    BlockDestination::Flags => {
                        vm_state.set_flag(result);
                    }
                    BlockDestination::Ip => {
                        // We run code until Eip modification.
                        // if eip modification occurs, VM need to check we are executing eip's code.
                        vm_state.set_ip(result);
                        break 'body;
                    }
                    BlockDestination::GprRegister(reg_id) => vm_state.gpr_mut(*reg_id).set(result),
                    BlockDestination::FprRegister(reg_id) => {
                        vm_state.fpr_mut(*reg_id).set(f64::from_bits(result))
                    }
                    BlockDestination::Memory(addr) => {
                        let mut frame = vm_state.mem(*addr);
                        match code.code_type {
                            Type::I8 | Type::U8 => frame.write_u8(result as u8),
                            Type::I16 | Type::U16 => frame.write_u16(result as u16),
                            Type::I32 | Type::U32 | Type::F32 => frame.write_u32(result as u32),
                            Type::I64 | Type::U64 | Type::F64 => frame.write_u64(result),
                            _ => unreachable!(),
                        }
                        .expect("Failed to write memory");
                    }
                    BlockDestination::MemoryRel(reg_id, offs) => {
                        let (val, _) = vm_state.gpr(*reg_id).get().overflowing_add_signed(*offs);
                        let mut frame = vm_state.mem(val);
                        match code.code_type {
                            Type::I8 | Type::U8 => frame.write_u8(result as u8),
                            Type::I16 | Type::U16 => frame.write_u16(result as u16),
                            Type::I32 | Type::U32 | Type::F32 => frame.write_u32(result as u32),
                            Type::I64 | Type::U64 | Type::F64 => frame.write_u64(result),
                            _ => unreachable!(),
                        }
                        .expect("Failed to write memory");
                    }
                    BlockDestination::None => {}
                    BlockDestination::SystemCall => {
                        vm_state.interrupt_model().syscall(result, vm_state)
                    }
                    BlockDestination::Exit => std::process::exit(result as i32),
                }

                if code.restore_flag {
                    vm_state.add_flag(flag_backup);
                }
            }

            vm_state.ip += *size as u64;
        }
    }
}
