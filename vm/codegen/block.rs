use crate::codegen::{CompiledBlockDestination, CompiledCode, Executable};
use crate::interrupt::InterruptModel;
use crate::VmState;

use smallvec::SmallVec;

pub struct CompiledBlock<C>
where
    C: CompiledCode,
{
    code: SmallVec<[CompiledBlockItem<C>; 2]>,
    size: u64,

    restore_flag: bool,
}

impl<C> CompiledBlock<C>
where
    C: CompiledCode,
{
    pub fn new(size: u64) -> Self {
        Self {
            code: Default::default(),
            size,

            restore_flag: false,
        }
    }

    pub fn push(&mut self, code: C, dest: CompiledBlockDestination) {
        self.code.push(CompiledBlockItem { code, dest });
    }

    pub fn set_restore_flag(&mut self) {
        self.restore_flag = true;
    }
}

impl<C> Executable for CompiledBlock<C>
where
    C: CompiledCode,
{
    unsafe fn execute(&self, vm: &mut VmState, interrupt_mode: &dyn InterruptModel) {
        for code in &self.code {
            let CompiledBlockItem { code, dest } = code;

            let val = code.execute(vm);
            dest.reflect(val, vm, interrupt_mode);
        }

        vm.set_ip(vm.ip() + self.size);
    }
}

struct CompiledBlockItem<C> {
    code: C,
    dest: CompiledBlockDestination,
}
