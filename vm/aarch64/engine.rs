use crate::aarch64::interrupt::AArch64UnixInterruptModel;
use crate::aarch64::{compile_code, AArch64Compiler};
use crate::engine::Engine;
use crate::image::Image;
use crate::register::{FprRegister, GprRegister, RegId};
use crate::{InterruptModel, Vm, VmContext};

use slab::Slab;

use std::collections::HashMap;
use std::sync::Arc;

pub struct AArch64VMEngine {
    vm: Vm,
    vm_ctx: Arc<VmContext>,

    interrupt: AArch64UnixInterruptModel,
}

impl Engine for AArch64VMEngine {
    fn init(image: Image) -> Self {
        let mut gpr_storage = Slab::new();
        let mut fpr_storage = Slab::new();
        let mut regs_byname = HashMap::new();

        // initialize AArch64 registers.
        let gpr_registers: [RegId; 32] = (0..32)
            .map(|i| {
                let name = format!("x{i}");
                let id = RegId(gpr_storage.insert(GprRegister::new(&name, 8)) as u8);
                regs_byname.insert(name, id);

                id
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let fpr_registers: [RegId; 32] = (0..32)
            .map(|i| {
                let name = format!("d{i}");
                let id = RegId(fpr_storage.insert(FprRegister::new(&name, 8)) as u8);
                regs_byname.insert(name, id);

                id
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let pstate_reg = RegId(gpr_storage.insert(GprRegister::new("pstate", 8)) as u8);
        let stack_reg = RegId(gpr_storage.insert(GprRegister::new("sp", 8)) as u8);

        let compiler = AArch64Compiler::new(gpr_registers, fpr_registers, pstate_reg, stack_reg);
        let mut vm_ctx = VmContext::new();
        build_image(&image, &compiler, &mut vm_ctx);

        let vm_ctx = Arc::new(vm_ctx);

        Self {
            vm: Vm {
                ctx: vm_ctx.clone(),
                gpr_registers: gpr_storage,
                fpr_registers: fpr_storage,
                regs_byname,

                ipv: 0,
                ipr2ipv_cache: HashMap::new(),
                ipr: image.entrypoint(),
                ip_modified: false,

                flags: 0,
            },
            vm_ctx,
            interrupt: AArch64UnixInterruptModel,
        }
    }

    unsafe fn run(&mut self) -> u64 {
        loop {
            let int = match self.vm.run() {
                Ok(ret) => return ret,
                Err(int) => int,
            };

            self.interrupt.interrupt(int, &mut self.vm, &self.vm_ctx);
            self.vm.inc_ip_current();
        }
    }
}

pub fn build_image(image: &Image, compiler: &AArch64Compiler, vm_ctx: &mut VmContext) {
    for sec_name in image.sections() {
        let addr = image.section_addr(sec_name);
        let data = image.section_data(sec_name);

        match sec_name {
            ".text" => compile_code(addr, data, compiler, vm_ctx),
            ".rodata" => unsafe {
                vm_ctx.mmu.mmap(addr, data.len() as u64, true, true, false);
                vm_ctx.mmu.frame(addr).write(data).unwrap();
            },
            _ => {}
        }
    }
}
