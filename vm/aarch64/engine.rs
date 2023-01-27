use crate::aarch64::{compile_text_segment, AArch64Compiler};
use crate::engine::Engine;
use crate::image::Image;
use crate::register::{FprRegister, GprRegister, RegId};
use crate::{Vm, VmContext};

use slab::Slab;

use std::sync::Arc;
use std::collections::HashMap;

pub struct AArch64VMEngine {
    vm: Vm
}

impl Engine for AArch64VMEngine {
    fn init(image: Image) -> Self {
        let mut gpr_storage = Slab::new();
        let mut fpr_storage = Slab::new();

        // initialize AArch64 registers.
        let pstate_reg = RegId(gpr_storage.insert(GprRegister::new("pstate", 8)) as u8);
        let gpr_registers: [RegId; 32] = (0..32)
            .map(|i| RegId(gpr_storage.insert(GprRegister::new(format!("x{i}"), 8)) as u8))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let fpr_registers: [RegId; 32] = (0..32)
            .map(|i| RegId(fpr_storage.insert(FprRegister::new(format!("f{i}"), 8)) as u8))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let compiler = AArch64Compiler::new(gpr_registers, fpr_registers, pstate_reg);
        let mut vm_ctx = VmContext::new();
        let entry_point = build_image(&image, &compiler, &mut vm_ctx);

        Self {
            vm: Vm {
                ctx: Arc::new(vm_ctx),
                gpr_registers: gpr_storage,
                fpr_registers: fpr_storage,
                
                ipv: 0,
                ipr2ipv_cache: HashMap::new(),
                ipr: entry_point,
                ip_modified: false,

                flags: 0,
            }
        }
    }

    fn run(&mut self) -> u64 {
        loop {
            let int = match self.vm.run() {
                Ok(ret) => return ret,
                Err(int) => int
            };

            match int {
                _ => unimplemented!("unimplemented interrupt: {:?}", int)
            }
        }
    }
}

pub fn build_image(image: &Image, compiler: &AArch64Compiler, vm_ctx: &mut VmContext) -> u64 {
    let mut entry_point = 0;
    for sec_name in image.sections() {
        let addr = image.section_addr(sec_name);
        let data = image.section_data(sec_name);

        match sec_name {
            ".text" => {
                entry_point = addr;
                compile_text_segment(addr, data, compiler, vm_ctx)
            }
            _ => { }
        }
    }

    entry_point
}