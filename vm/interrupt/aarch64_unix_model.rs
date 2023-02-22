use crate::interrupt::InterruptModel;
use crate::mmu::PAGE_SIZE;
use crate::Cpu;

pub struct AArch64UnixInterruptModel;
impl InterruptModel for AArch64UnixInterruptModel {
    unsafe fn syscall(&self, _: u64, vm: &mut Cpu) {
        let nr = vm.gpr(vm.reg_by_name("x8").unwrap()).u64();
        let arg0 = vm.gpr(vm.reg_by_name("x0").unwrap()).u64();
        let arg1 = vm.gpr(vm.reg_by_name("x1").unwrap()).u64();
        let arg2 = vm.gpr(vm.reg_by_name("x2").unwrap()).u64();
        let arg3 = vm.gpr(vm.reg_by_name("x3").unwrap()).u64();
        let arg4 = vm.gpr(vm.reg_by_name("x4").unwrap()).u64();
        let arg5 = vm.gpr(vm.reg_by_name("x5").unwrap()).u64();

        handle_syscall(nr, [arg0, arg1, arg2, arg3, arg4, arg5], vm)
    }
}

pub unsafe fn handle_syscall(nr: u64, args: [u64; 6], vm: &mut Cpu) {
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

        // geteuid
        0xaf => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            // We only have one user emulated on this machine.
            *ret.u64_mut() = 0;
        }

        // getuid
        0xae => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            // We only have one user emulated on this machine.
            *ret.u64_mut() = 0;
        }

        0xb0 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            // We only have one group emulated on this machine.
            *ret.u64_mut() = 0;
        }

        // getegid
        0xb1 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            // We only have one group emulated on this machine.
            *ret.u64_mut() = 0;
        }

        // brk
        0xd6 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }

        // set_tid_address
        0x60 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }

        // flock
        0x49 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }

        // sigaltstack. We don't support stack overflow signals.
        0x84 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }

        // mmap
        0xde => {
            let addr = args[0];
            let len = args[1];
            let prot = args[2];
            let flags = args[3];
            let fd = args[4];
            let offset = args[5];
            println!(
                "mmap: addr: {:#x} len: {:#x} prot: {:#x} flags: {:#x} fd: {:#x} offset: {:#x}",
                addr, len, prot, flags, fd, offset
            );

            let addr = 0x8000_0000_0000_0000;
            vm.mmu().mmap(addr, PAGE_SIZE, true, true, true);

            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = addr;
        }

        // rt_sigaction
        0x86 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }

        // rt_sigprocmask
        0x87 => {
            let ret = vm.reg_by_name("x0").unwrap();
            let ret = vm.gpr_mut(ret);

            *ret.u64_mut() = 0;
        }
        _ => unimplemented!("unknown interrupt! {}", nr),
    }
}
