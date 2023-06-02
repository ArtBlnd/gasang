use core::{
    ir::{BasicBlock, BasicBlockTerminator, IrConstant, IrInst, IrType, IrValue},
    RegisterId,
};

use super::{AArch64Inst, HwImm16Rd};

pub(crate) fn compile_aarch64_to_ir(inst: &AArch64Inst, basic_block: &mut BasicBlock) {
    assert!(basic_block.terminator() != BasicBlockTerminator::None);

    match inst {
        AArch64Inst::MovnVar32(operand) => compile_movn(basic_block, operand, IrType::U32),
        AArch64Inst::MovnVar64(operand) => compile_movn(basic_block, operand, IrType::U64),
        _ => unimplemented!(),
    }
}

fn compile_movn(bb: &mut BasicBlock, operand: &HwImm16Rd, ty: IrType) {
    let rd = operand.rd.raw();

    bb.push_inst(IrInst::ZextCast {
        dst: IrValue::Register(IrType::U64, rd),
        src: IrValue::Constant(IrConstant::new(ty, {
            let pos = operand.hw << 4;
            let res = !((operand.imm16 as u64) << pos);

            res
        })),
    });
}
