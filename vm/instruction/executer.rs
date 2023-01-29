use crate::instruction::VmIr;
use crate::instruction::*;

use crate::{Interrupt, Vm, VmContext};

#[inline]
pub unsafe fn execute_instr(ir: VmIr, vm: &mut Vm, vm_ctx: &VmContext) -> Result<(), Interrupt> {
    let mut executor = VmIrExecutor {
        vm,
        vm_ctx,
        interrupt: None,
    };

    ir.visit(&mut executor);

    if let Some(int) = executor.interrupt {
        return Err(int);
    }

    Ok(())
}

struct VmIrExecutor<'v> {
    vm: &'v mut Vm,
    vm_ctx: &'v VmContext,

    interrupt: Option<Interrupt>,
}

impl<'v> InstrVisitor for VmIrExecutor<'v> {
    fn visit_no_operand(&mut self, op: u8) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1(&mut self, op: u8, operand: Reg1) {
        match op {
            IROP_MOV_IPR2REG => {
                let ipr = self.vm.ipr();
                self.vm.gpr(operand.op1).set(ipr);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        match op {
            IROP_MOV_REG2REG => {
                let src = self.vm.gpr(operand.op1).get();
                self.vm.gpr(operand.op2).set(src);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg3(&mut self, op: u8, _operand: Reg3) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1u8(&mut self, op: u8, _operand: Reg1U8) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg2u8(&mut self, op: u8, _operand: Reg2U8) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1u32(&mut self, op: u8, _operand: Reg1U32) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1i32(&mut self, op: u8, operand: Reg1I32) {
        match op {
            IROP_IADD_CST32 => {
                let reg = self.vm.gpr(operand.op1);
                let (result, _cf) = reg.get().overflowing_add(operand.imm32 as u64);
                reg.set(result);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1u64(&mut self, op: u8, _operand: Reg1U64) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1u16(&mut self, op: u8, operand: Reg1U16) {
        match op {
            IROP_MOV_16CST2REG => self.vm.gpr(operand.op1).set(operand.imm16 as u64),
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_u16(&mut self, op: u8, operand: U16) {
        match op {
            IROP_SVC => {
                self.interrupt = Some(Interrupt::SystemCall(operand.imm16 as usize));
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2i64(&mut self, op: u8, _operand: Reg2I64) {
        unimplemented!("opcode: {:02x}", op)
    }

    fn visit_reg1i64(&mut self, op: u8, _operand: Reg1I64) {
        unimplemented!("opcode: {:02x}", op)
    }
}
