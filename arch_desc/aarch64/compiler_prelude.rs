use core::{
    ir::{BasicBlock, IrConstant, IrInst, IrType, IrValue},
    Architecture, Register,
};

use super::AArch64Architecture;

pub fn gen_move_pc(bb: &mut BasicBlock) {
    bb.push_inst(IrInst::Add {
        dst: IrValue::Register(IrType::U64, AArch64Architecture::get_pc_register().raw()),
        lhs: IrValue::Register(IrType::U64, AArch64Architecture::get_pc_register().raw()),
        rhs: IrValue::Constant(IrConstant::U64(4)),
    });
}
