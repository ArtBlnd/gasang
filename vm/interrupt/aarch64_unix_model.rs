use crate::interrupt::InterruptModel;
use crate::VmState;

pub struct AArch64UnixInterruptModel;
impl InterruptModel for AArch64UnixInterruptModel {
    unsafe fn syscall(&self, _: u64, vm: &VmState) {
        let nr = vm.gpr(vm.reg_by_name("x8").unwrap()).get();
        let arg0 = vm.gpr(vm.reg_by_name("x0").unwrap()).get();
        let arg1 = vm.gpr(vm.reg_by_name("x1").unwrap()).get();
        let arg2 = vm.gpr(vm.reg_by_name("x2").unwrap()).get();
        let arg3 = vm.gpr(vm.reg_by_name("x3").unwrap()).get();
        let arg4 = vm.gpr(vm.reg_by_name("x4").unwrap()).get();
        let arg5 = vm.gpr(vm.reg_by_name("x5").unwrap()).get();

        handle_syscall(nr, [arg0, arg1, arg2, arg3, arg4, arg5], vm)
    }
}

pub unsafe fn handle_syscall(nr: u64, args: [u64; 6], vm: &VmState) {
    match nr {
        // write arg0:fd arg1:buf arg0: length
        0x40 => {
            let data = args[1];
            let size = args[2];

            // make a memory for buffer reading
            let mut buf = Vec::with_capacity(size as usize);
            buf.set_len(size as usize);

            // get memory frame and read data
            let mut frame = vm.mem(data);
            frame.read(&mut buf).unwrap();

            const STDOUT: u64 = 1;
            if args[0] == STDOUT {
                let chars = std::str::from_utf8_unchecked(&buf);
                println!("{chars}");
            }
        }

        // exit_group arg0:error_code
        0x5e => {
            std::process::exit(args[0] as i32);
        }
        _ => unimplemented!("unknown interrupt! {}", nr),
    }
}
