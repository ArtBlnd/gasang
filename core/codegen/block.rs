use crate::codegen::{CompiledBlockDestination, CompiledCode, Executable};

use crate::Cpu;

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
    unsafe fn execute(&self, vm: &mut Cpu) {
        for code in &self.code {
            let CompiledBlockItem { code, dest } = code;

            let val = code.execute(vm);
            dest.reflect(val, vm);
            if dest.is_dest_ip_or_exit() {
                return;
            }
        }

        vm.set_ip(vm.ip() + self.size);
    }
}

struct CompiledBlockItem<C> {
    code: C,
    dest: CompiledBlockDestination,
}
