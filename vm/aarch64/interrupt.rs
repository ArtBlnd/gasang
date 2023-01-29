use crate::{Interrupt, InterruptModel, Vm, VmContext};

pub struct AArch64UnixInterruptModel;
impl InterruptModel for AArch64UnixInterruptModel {
    unsafe fn interrupt(&self, int: Interrupt, vm: &mut Vm, vm_ctx: &VmContext) {
        match int {
            Interrupt::SystemCall(_) => {
                let nr_reg = vm.reg_by_name("x8").unwrap();
                let nr_val = vm.gpr(nr_reg).get();

                let arg0 = vm.gpr(vm.reg_by_name("x0").unwrap()).get();
                let arg1 = vm.gpr(vm.reg_by_name("x1").unwrap()).get();
                let arg2 = vm.gpr(vm.reg_by_name("x2").unwrap()).get();
                let arg3 = vm.gpr(vm.reg_by_name("x3").unwrap()).get();
                let arg4 = vm.gpr(vm.reg_by_name("x4").unwrap()).get();
                let arg5 = vm.gpr(vm.reg_by_name("x5").unwrap()).get();

                handle_sys_call(nr_val, [arg0, arg1, arg2, arg3, arg4, arg5], vm, vm_ctx);
            }

            Interrupt::DebugBreakpoint(_) => {
                panic!("debug breakpoint triggered!");
            }

            _ => unreachable!("unknown interrupt type! {}", int),
        }
    }
}

pub unsafe fn handle_sys_call(nr: u64, args: [u64; 6], vm: &mut Vm, vm_ctx: &VmContext) {
    match nr {
        // write arg0:fd arg1:buf arg0: length
        0x40 => {
            let data = args[1];
            let size = args[2];

            // make a memory for buffer reading
            let mut buf = Vec::new();
            buf.resize(size as usize, 0);

            // get memory frame and read string data
            let mut frame = vm.mem(data);
            frame.read(&mut buf).unwrap();

            let chars = std::str::from_utf8_unchecked(&buf);

            const STDOUT: u64 = 1;
            if args[0] == STDOUT {
                println!("{}", chars);
            }
        }

        // exit_group arg0:error_code
        0x5e => {
            std::process::exit(args[0] as i32);
        }
        _ => unimplemented!("unknown interrupt! {}", nr),
    }
}
