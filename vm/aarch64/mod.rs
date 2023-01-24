mod compiler;
pub use compiler::*;

use crate::instr::{VmInstr, VmInstrOp};
use crate::jump_table::{JumpId, JumpTable};
use crate::register::RegId;
use crate::Vm;
use crate::VmContext;

use machineinstr::aarch64::*;
use machineinstr::utils::BitReader;
use machineinstr::MachineInstParser;

use std::collections::HashMap;

fn relative_ipr(ipr: u64, offset: i64) -> u64 {
    (ipr as i128 + offset as i128) as u64
}

fn find_ipv(ctx: &VmContext, ipr: u64) -> usize {
    // Get checkpoint and its real instruction pointer
    let cp = ctx.jump_table.get_checkpoint(ipr);

    let mut cp_ipv = cp.ipv;
    let mut cp_ipr = cp.ipr;

    // find ipv that has same ipr
    loop {
        assert!(cp_ipr > ipr, "Bad instruction size and its checkpoint!");
        if cp_ipr == ipr {
            break;
        }

        let instr = &ctx.vm_instr[cp_ipv];
        cp_ipv -= 1;
        cp_ipr -= instr.size as u64;
    }

    cp_ipv
}

fn compile_text(addr: usize, data: &[u8], compiler: &AArch64Compiler, vm_ctx: &mut VmContext) {
    // Key; branch destination, value; ip of branch instruction
    let mut unresolved_branches = HashMap::new();
    let mut cp_distance: u64 = 0;

    // Compile instructions into VMIR and insert it to context.
    let mut ipr = addr as u64;
    let parser =
        MachineInstParser::new(BitReader::new(data.iter().cloned()), AArch64InstrParserRule);
    for native_instr in parser {
        // try to compile instruction.
        for instr in compiler.compile_instr(native_instr.op) {
            // if instruction is a branch instruction, try to find its ipv destination.
            match instr {
                // If offset is negative, we need to travel back to find ipv of destination.
                VmInstrOp::JumpIprCnt { dst_offs: dst } if dst < 0 => {
                    // travel back to find ipv of destination.
                    let target_ipr = relative_ipr(ipr, dst);
                    let ipv = find_ipv(vm_ctx, target_ipr);

                    let jump_id = vm_ctx.jump_table.new_jump(ipv);

                    vm_ctx.vm_instr.push(VmInstr {
                        op: VmInstrOp::JumpIpv {
                            dst: jump_id,
                            dst_ipr: target_ipr,
                        },
                        size: native_instr.size,
                    });
                }

                // if offset is positive, save it to unresolved_branches
                // and when target ipr is found. update it to ipv jump.
                VmInstrOp::JumpIprCnt { dst_offs: dst } if dst > 0 => {
                    let ipr = relative_ipr(ipr, dst);
                    unresolved_branches.insert(ipr, vm_ctx.vm_instr.len());
                }

                _ => {
                    vm_ctx.vm_instr.push(VmInstr {
                        op: instr,
                        size: native_instr.size,
                    });
                }
            }
        }

        // We found destination of branch instruction.
        // resolve JumpIprCst to JumpIpv instruction.
        if let Some(br_instr_ipv) = unresolved_branches.remove(&ipr) {
            let jumpid = vm_ctx.jump_table.new_jump(vm_ctx.vm_instr.len());
            vm_ctx.vm_instr[br_instr_ipv].op = VmInstrOp::JumpIpv {
                dst: jumpid,
                dst_ipr: ipr,
            };
        }

        // We've compiled native instruction, increase ipr.
        ipr += native_instr.size as u64;

        // we need checkpoint to find destination of JumpIpr instruction.
        cp_distance += native_instr.size as u64;
        if cp_distance > vm_ctx.jump_table.checkpoint_min_distance() {
            vm_ctx.jump_table.new_checkpoint(ipr, vm_ctx.vm_instr.len());
            cp_distance = 0;
        }
    }

    // Write instructions to memory.
    let mut mem_frame = vm_ctx.mmu.frame(addr).unwrap();
    unsafe {
        mem_frame.write(data).unwrap();
    }
}
