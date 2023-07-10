use core::{
    ir::{
        BasicBlock, BasicBlockTerminator, IrConstant, IrInst, IrType, IrValue, Reordering, VecTy,
    },
    Architecture, Register,
};

use crate::aarch64::AArch64Register;

use super::{
    compiler_prelude::{self, *},
    AArch64Architecture, AArch64Inst, AddSubtractExtReg, AdvSimdModifiedImm, AdvancedSimdCopy,
    B5B40Imm14Rt, Bitfield, CondCmpImm, CondCmpReg, DataProc2Src, DataProc3Src, ExceptionGen,
    HwImm16Rd, Imm19Cond, Imm19Rt, Imm26, LdStRegUnscaledImm, LoadStoreRegPair,
    LoadStoreRegRegOffset, LogicalImm, OpcSizeImm12RnRt, PcRelAddressing, PstateOp, RmCondRnRd,
    RnRd, RsRt2RnRt, ShImm12RnRd, ShiftRmImm6RnRd, SysRegMov, UncondBranchReg,
};

pub(crate) fn compile_aarch64_to_ir(inst: &AArch64Inst, basic_block: &mut BasicBlock) {
    assert!(basic_block.terminator() == BasicBlockTerminator::None);

    match inst {
        AArch64Inst::MovzVar32(operand) | AArch64Inst::MovzVar64(operand) => {
            compile_movz(basic_block, operand)
        }
        AArch64Inst::MovnVar32(operand) => compile_movn(basic_block, operand, IrType::B32),
        AArch64Inst::MovnVar64(operand) => compile_movn(basic_block, operand, IrType::B64),
        AArch64Inst::MovkVar32(operand) => compile_movk(basic_block, operand, IrType::B32),
        AArch64Inst::MovkVar64(operand) => compile_movk(basic_block, operand, IrType::B64),
        AArch64Inst::MoviVectorVar64(operand) => compile_movi(basic_block, operand),
        AArch64Inst::Adr(operand) => compile_adr(basic_block, operand),
        AArch64Inst::Adrp(operand) => compile_adrp(basic_block, operand),

        AArch64Inst::RevVar32(operand) => compile_rev_var(basic_block, operand, IrType::B32),
        AArch64Inst::RevVar64(operand) => compile_rev_var(basic_block, operand, IrType::B64),

        // Load and Stores
        AArch64Inst::LdrImm32(operand) => compile_ldr_imm(basic_block, operand, IrType::B32),
        AArch64Inst::LdrImm64(operand) => compile_ldr_imm(basic_block, operand, IrType::B64),
        AArch64Inst::LdrImmSimdFP64(operand) => {
            compile_ldr_imm_simd_fp(basic_block, operand, IrType::B64)
        }
        AArch64Inst::LdrImmSimdFP128(operand) => {
            compile_ldr_imm_simd_fp(basic_block, operand, IrType::Vector(VecTy::U64, 2))
        }
        AArch64Inst::LdrLitVar64(operand) => compile_ldr_lit_var64(basic_block, operand),
        AArch64Inst::LdrhImm(operand) => compile_ldrh_imm(basic_block, operand),
        AArch64Inst::LdrbImm(operand) => compile_ldrb_imm(basic_block, operand),
        AArch64Inst::LdrReg32(operand) => compile_ldr_reg(basic_block, operand, IrType::B32),
        AArch64Inst::LdrReg64(operand) => compile_ldr_reg(basic_block, operand, IrType::B64),
        AArch64Inst::LdrbRegShiftedReg(operand) => {
            compile_ldrb_reg_shifted_reg(basic_block, operand)
        }
        AArch64Inst::LdpVar64(operand) => compile_ldp(basic_block, operand, IrType::B64),
        AArch64Inst::LdpVar32(operand) => compile_ldp(basic_block, operand, IrType::B32),
        AArch64Inst::LdrshReg64(operand) => compile_ldrsh_reg(basic_block, operand, IrType::B64),
        AArch64Inst::LdrshReg32(operand) => compile_ldrsh_reg(basic_block, operand, IrType::B32),
        AArch64Inst::LdaxrVar32(operand) => compile_ldaxr(basic_block, operand, IrType::B32),
        AArch64Inst::LdarVar64(operand) => compile_ldar(basic_block, operand, IrType::B64),
        AArch64Inst::Ldur64(operand) => compile_ldur(basic_block, operand, IrType::B64),
        AArch64Inst::LdpSimdFpVar128(operand) => {
            compile_ldp_simd_fp(basic_block, operand, IrType::Vector(VecTy::U64, 2))
        }
        AArch64Inst::LdrRegSimdFP(operand) => compile_ldr_reg_simd_fp(basic_block, operand),
        AArch64Inst::LdxrVar64(operand) => compile_ldxr(basic_block, operand, IrType::B64),

        AArch64Inst::StrImm32(operand) => compile_str_imm(basic_block, operand, IrType::B32),
        AArch64Inst::StrImm64(operand) => compile_str_imm(basic_block, operand, IrType::B64),
        AArch64Inst::StpVar64(operand) => compile_stp_var(basic_block, operand, IrType::B64),
        AArch64Inst::StpVar32(operand) => compile_stp_var(basic_block, operand, IrType::B32),
        AArch64Inst::StrbImm(operand) => compile_strb_imm(basic_block, operand),
        AArch64Inst::Sturb(operand) => compile_sturb_imm(basic_block, operand),
        AArch64Inst::StrReg32(operand) => compile_str_reg(basic_block, operand, IrType::B32),
        AArch64Inst::StrReg64(operand) => compile_str_reg(basic_block, operand, IrType::B64),
        AArch64Inst::Stur32(operand) => compile_stur(basic_block, operand, IrType::B32),
        AArch64Inst::Stur64(operand) => compile_stur(basic_block, operand, IrType::B64),
        AArch64Inst::SturSimdFP64(operand) => {
            compile_stur_simd_fp(basic_block, operand, IrType::B64)
        }
        AArch64Inst::SturSimdFP128(operand) => {
            compile_stur_simd_fp(basic_block, operand, IrType::Vector(VecTy::U64, 2))
        }
        AArch64Inst::StpSimdFpVar128(operand) => {
            compile_stp_simd_fp(basic_block, operand, IrType::Vector(VecTy::U64, 2))
        }
        AArch64Inst::StrImmSimdFP64(operand) => {
            compile_str_imm_simd_fp(basic_block, operand, IrType::B64)
        }
        AArch64Inst::StrImmSimdFP128(operand) => {
            compile_str_imm_simd_fp(basic_block, operand, IrType::Vector(VecTy::U64, 2))
        }
        AArch64Inst::StrRegSimdFP(operand) => compile_str_reg_simd_fp(basic_block, operand),
        AArch64Inst::StlxrVar32(operand) => compile_stlxr(basic_block, operand, IrType::B32),
        AArch64Inst::StxrVar64(operand) => compile_stxr(basic_block, operand, IrType::B64),
        AArch64Inst::StxrVar32(operand) => compile_stxr(basic_block, operand, IrType::B32),
        AArch64Inst::StrbRegShiftedReg(operand) => compile_strb_reg(basic_block, operand),

        // Advanced SIMD and FP
        AArch64Inst::DupGeneral(operand) => compile_dup_general(basic_block, operand),

        // Arithmetic instructions
        AArch64Inst::AddImm64(operand) => compile_add_imm(basic_block, operand, IrType::B64),
        AArch64Inst::AddImm32(operand) => compile_add_imm(basic_block, operand, IrType::B32),
        AArch64Inst::AddsImm64(operand) => compile_adds_imm(basic_block, operand, IrType::B64),
        AArch64Inst::AddsImm32(operand) => compile_adds_imm(basic_block, operand, IrType::B32),
        AArch64Inst::AddShiftedReg64(operand) => {
            compile_add_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::AddsShiftedReg64(operand) => {
            compile_adds_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::AddExtReg64(operand) => compile_add_ext_reg(basic_block, operand, IrType::B64),
        AArch64Inst::SubImm64(operand) => compile_sub_imm(basic_block, operand, IrType::B64),
        AArch64Inst::SubImm32(operand) => compile_sub_imm(basic_block, operand, IrType::B32),
        AArch64Inst::SubShiftedReg64(operand) => {
            compile_sub_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::SubsShiftedReg32(operand) => {
            compile_subs_shifted_reg(basic_block, operand, IrType::B32)
        }
        AArch64Inst::SubsShiftedReg64(operand) => {
            compile_subs_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::SubsExtReg64(operand) => {
            compile_subs_ext_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::SubsImm64(operand) => compile_subs_imm(basic_block, operand, IrType::B64),
        AArch64Inst::SubsImm32(operand) => compile_subs_imm(basic_block, operand, IrType::B32),
        AArch64Inst::Madd32(operand) => compile_madd(basic_block, operand, IrType::B32),
        AArch64Inst::Madd64(operand) => compile_madd(basic_block, operand, IrType::B64),
        AArch64Inst::Msub32(operand) => compile_msub(basic_block, operand, IrType::B32),
        AArch64Inst::SdivVar32(operand) => compile_div(basic_block, operand, IrType::B32),
        AArch64Inst::SdivVar64(operand) => compile_div(basic_block, operand, IrType::B64),
        AArch64Inst::UdivVar32(operand) => compile_div(basic_block, operand, IrType::B32),
        AArch64Inst::UdivVar64(operand) => compile_div(basic_block, operand, IrType::B64),

        // bitwise isntructions
        AArch64Inst::Ubfm32(operand) => compile_ubfm(basic_block, operand, IrType::B32),
        AArch64Inst::Ubfm64(operand) => compile_ubfm(basic_block, operand, IrType::B64),
        AArch64Inst::Sbfm64(operand) => compile_sbfm(basic_block, operand, IrType::B64),
        AArch64Inst::AndImm64(operand) => compile_and_imm(basic_block, operand, IrType::B64),
        AArch64Inst::AndImm32(operand) => compile_and_imm(basic_block, operand, IrType::B32),
        AArch64Inst::AndsImm64(operand) => compile_ands_imm(basic_block, operand, IrType::B64),
        AArch64Inst::AndsImm32(operand) => compile_ands_imm(basic_block, operand, IrType::B32),
        AArch64Inst::AndsShiftedReg32(operand) => {
            compile_ands_shifted_reg(basic_block, operand, IrType::B32)
        }
        AArch64Inst::AndsShiftedReg64(operand) => {
            compile_ands_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::AndShiftedReg64(operand) => {
            compile_and_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::OrrImm64(operand) => compile_orr_imm(basic_block, operand, IrType::B64),
        AArch64Inst::OrrImm32(operand) => compile_orr_imm(basic_block, operand, IrType::B32),
        AArch64Inst::OrrShiftedReg64(operand) => {
            compile_orr_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::OrrShiftedReg32(operand) => {
            compile_orr_shifted_reg(basic_block, operand, IrType::B32)
        }
        AArch64Inst::OrnShiftedReg64(operand) => {
            compile_orn_shifted_reg(basic_block, operand, IrType::B64)
        }
        AArch64Inst::OrnShiftedReg32(operand) => {
            compile_orn_shifted_reg(basic_block, operand, IrType::B32)
        }

        AArch64Inst::LslvVar64(operand) => compile_lslv(basic_block, operand, IrType::B64),
        AArch64Inst::LslvVar32(operand) => compile_lslv(basic_block, operand, IrType::B32),

        // Branch instructions
        AArch64Inst::BlImm(operand) => compile_bl_imm(basic_block, operand),
        AArch64Inst::BImm(operand) => compile_b_imm(basic_block, operand),
        AArch64Inst::Br(operand) => compile_br(basic_block, operand),
        AArch64Inst::Blr(operand) => compile_blr(basic_block, operand),
        AArch64Inst::BCond(operand) => compile_b_cond(basic_block, operand),
        AArch64Inst::Cbz64(operand) => compile_cbz(basic_block, operand, IrType::B32),
        AArch64Inst::Cbz32(operand) => compile_cbz(basic_block, operand, IrType::B64),
        AArch64Inst::Cbnz32(operand) => compile_cbnz(basic_block, operand, IrType::B32),
        AArch64Inst::Cbnz64(operand) => compile_cbnz(basic_block, operand, IrType::B64),
        AArch64Inst::Ret(operand) => compile_ret(basic_block, operand),
        AArch64Inst::Tbz(operand) => compile_tbz(basic_block, operand),
        AArch64Inst::Tbnz(operand) => compile_tbnz(basic_block, operand),

        // Conditional Instructions
        AArch64Inst::CcmpImmVar32(operand) => compile_ccmp_imm(basic_block, operand, IrType::B32),
        AArch64Inst::CcmpImmVar64(operand) => compile_ccmp_imm(basic_block, operand, IrType::B64),
        AArch64Inst::CcmpRegVar64(operand) => compile_ccmp_reg(basic_block, operand, IrType::B64),
        AArch64Inst::CcmnImmVar64(operand) => compile_ccmn_imm(basic_block, operand, IrType::B64),
        AArch64Inst::Csel32(operand) => compile_csel(basic_block, operand, IrType::B32),
        AArch64Inst::Csel64(operand) => compile_csel(basic_block, operand, IrType::B64),
        AArch64Inst::Csinv64(operand) => compile_csinv(basic_block, operand, IrType::B64),

        // Interrupt Instructions
        AArch64Inst::Svc(operand) => compile_svc(basic_block, operand),
        AArch64Inst::Brk(operand) => compile_brk(basic_block, operand),

        // Speical instructions
        AArch64Inst::Mrs(operand) => compile_mrs(basic_block, operand),
        AArch64Inst::MsrReg(operand) => compile_msr_reg(basic_block, operand),
        AArch64Inst::MsrImm(operand) => compile_msr_imm(basic_block, operand),
        AArch64Inst::Nop | AArch64Inst::Wfi | AArch64Inst::Dmb(_) | AArch64Inst::Isb(_) => {
            todo!()
        }
        _ => unimplemented!(),
    }
}

fn compile_movz(bb: &mut BasicBlock, operand: &HwImm16Rd) {
    let rd = operand.rd.raw();

    let pos = operand.hw << 4;
    bb.push_inst(IrInst::Assign {
        dst: IrValue::Register(IrType::B64, rd),
        src: IrValue::Constant(IrConstant::B64((operand.imm16 as u64) << pos)),
    });

    compiler_prelude::gen_move_pc(bb);
}

fn compile_movn(bb: &mut BasicBlock, operand: &HwImm16Rd, ty: IrType) {
    let rd = operand.rd.raw();

    bb.push_inst(IrInst::ZextCast {
        dst: IrValue::Register(IrType::B64, rd),
        src: IrValue::Constant(IrConstant::new(ty, {
            let pos = operand.hw << 4;
            !((operand.imm16 as u64) << pos)
        })),
    });

    compiler_prelude::gen_move_pc(bb);
}

fn compile_movk(bb: &mut BasicBlock, operand: &HwImm16Rd, ty: IrType) {
    let rd = operand.rd.raw();

    let pos = operand.hw << 4;
    let mask = !(ones::<u64>(pos as u32) << pos);

    // t1 = rd & !(ones(pos) << pos)
    // t2 = t1 | (imm16 << pos)
    // rd = zext(t2)
    let t1 = bb.new_variable(ty);
    let t2 = bb.new_variable(ty);
    bb.push_inst(IrInst::And {
        dst: t1,
        lhs: IrValue::Register(ty, rd),
        rhs: IrValue::Constant(IrConstant::new(ty, mask)),
    });
    bb.push_inst(IrInst::Or {
        dst: t2,
        lhs: t1,
        rhs: IrValue::Constant(IrConstant::new(ty, (operand.imm16 as u64) << pos)),
    });
    bb.push_inst(IrInst::ZextCast {
        dst: IrValue::Register(IrType::B64, rd),
        src: t2,
    });

    compiler_prelude::gen_move_pc(bb);
}

fn compile_movi(bb: &mut BasicBlock, operand: &AdvSimdModifiedImm) {
    todo!()
}

fn compile_adr(bb: &mut BasicBlock, operand: &PcRelAddressing) {
    let rd = operand.rd.raw();
    let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 21);
    let off = gen_pc_rel(bb, IrValue::Constant(IrConstant::new(IrType::B64, imm)));

    bb.push_inst(IrInst::Assign {
        dst: IrValue::Register(IrType::B64, rd),
        src: off,
    });
}

fn compile_adrp(bb: &mut BasicBlock, operand: &PcRelAddressing) {
    let rd = operand.rd.raw();
    let pc = AArch64Architecture::get_pc_register().raw();

    let imm = sign_extend(
        ((operand.immhi as i64) << 2 | (operand.immlo as i64)) << 12,
        33,
    );

    let t1 = bb.new_variable(IrType::B64);
    bb.push_inst(IrInst::And {
        dst: t1,
        lhs: IrValue::Register(IrType::B64, pc),
        rhs: IrValue::Constant(IrConstant::new(IrType::B64, 0xFFFF_FFFF_FFFF_F000u64)),
    });
    bb.push_inst(IrInst::Add {
        dst: IrValue::Register(IrType::B64, rd),
        lhs: t1,
        rhs: IrValue::Constant(IrConstant::new(IrType::B64, imm)),
    });
}

fn compile_rev_var(bb: &mut BasicBlock, operand: &RnRd, ty: IrType) {
    todo!()
}

fn compile_ldr_imm(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt, ty: IrType) {
    let (wback, post_index, offset) = decode_operand_for_ld_st_reg_imm(operand, false);

    let address = bb.new_variable(ty);

    let offset_temp = if !post_index { offset } else { 0 };

    bb.push_inst(IrInst::Add {
        dst: address,
        lhs: IrValue::Constant(IrConstant::B64(offset_temp as u64)),
        rhs: IrValue::Register(ty, operand.rn.raw()),
    });

    let data = bb.new_variable(ty);

    bb.push_inst(IrInst::Load {
        dst: data,
        src: address,
    });

    bb.push_inst(IrInst::ZextCast {
        dst: IrValue::Register(ty, operand.rt.raw()),
        src: data,
    });

    if wback {
        bb.push_inst(IrInst::Assign {
            dst: IrValue::Register(ty, operand.rn.raw()),
            src: address,
        })
    }
}

fn compile_ldr_imm_simd_fp(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt, ty: IrType) {
    todo!()
}

fn compile_ldr_lit_var64(bb: &mut BasicBlock, operand: &Imm19Rt) {
    todo!()
}

fn compile_ldrh_imm(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt) {
    todo!()
}

fn compile_ldrb_imm(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt) {
    todo!()
}

fn compile_ldr_reg(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset, ty: IrType) {
    todo!()
}

fn compile_ldrb_reg_shifted_reg(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset) {
    todo!()
}

fn compile_ldp(bb: &mut BasicBlock, operand: &LoadStoreRegPair, ty: IrType) {
    todo!()
}

fn compile_ldrsh_reg(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset, ty: IrType) {
    todo!()
}

fn compile_ldaxr(bb: &mut BasicBlock, operand: &RsRt2RnRt, ty: IrType) {
    todo!()
}

fn compile_ldar(bb: &mut BasicBlock, operand: &RsRt2RnRt, ty: IrType) {
    todo!()
}

fn compile_ldur(bb: &mut BasicBlock, operand: &LdStRegUnscaledImm, ty: IrType) {
    todo!()
}

fn compile_ldp_simd_fp(bb: &mut BasicBlock, operand: &LoadStoreRegPair, ty: IrType) {
    todo!()
}

fn compile_ldr_reg_simd_fp(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset) {
    todo!()
}

fn compile_ldxr(bb: &mut BasicBlock, operand: &RsRt2RnRt, ty: IrType) {
    todo!()
}

fn compile_str_imm(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt, ty: IrType) {
    let (wback, post_index, offset) = decode_operand_for_ld_st_reg_imm(operand, false);

    let address = bb.new_variable(ty);

    let offset_temp = if !post_index { offset } else { 0 };

    let rn = IrValue::Register(ty, operand.rn.raw());

    bb.push_inst(IrInst::Add {
        dst: address,
        lhs: rn,
        rhs: IrValue::Constant(IrConstant::new(ty, offset_temp)),
    });

    bb.push_inst(IrInst::Fence(Reordering::Relaxed));

    bb.push_inst(IrInst::Store {
        dst: address,
        src: IrValue::Register(ty, operand.rt.raw()),
    });

    if wback {
        bb.push_inst(IrInst::Assign {
            dst: rn,
            src: address,
        })
    }
}

fn compile_stp_var(bb: &mut BasicBlock, operand: &LoadStoreRegPair, ty: IrType) {
    todo!()
}

fn compile_strb_imm(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt) {
    todo!()
}

fn compile_sturb_imm(bb: &mut BasicBlock, operand: &LdStRegUnscaledImm) {
    todo!()
}

fn compile_str_reg(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset, ty: IrType) {
    todo!()
}

fn compile_stur(bb: &mut BasicBlock, operand: &LdStRegUnscaledImm, ty: IrType) {
    todo!()
}

fn compile_stur_simd_fp(bb: &mut BasicBlock, operand: &LdStRegUnscaledImm, ty: IrType) {
    todo!()
}

fn compile_stp_simd_fp(bb: &mut BasicBlock, operand: &LoadStoreRegPair, ty: IrType) {
    todo!()
}

fn compile_str_imm_simd_fp(bb: &mut BasicBlock, operand: &OpcSizeImm12RnRt, ty: IrType) {
    todo!()
}

fn compile_str_reg_simd_fp(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset) {
    todo!()
}

fn compile_stlxr(bb: &mut BasicBlock, operand: &RsRt2RnRt, ty: IrType) {
    todo!()
}

fn compile_stxr(bb: &mut BasicBlock, operand: &RsRt2RnRt, ty: IrType) {
    todo!()
}

fn compile_strb_reg(bb: &mut BasicBlock, operand: &LoadStoreRegRegOffset) {
    todo!()
}

fn compile_dup_general(bb: &mut BasicBlock, operand: &AdvancedSimdCopy) {
    todo!()
}

fn compile_add_imm(bb: &mut BasicBlock, operand: &ShImm12RnRd, ty: IrType) {
    let imm = if operand.sh == 0b0 {
        operand.imm12 as u32
    } else {
        (operand.imm12 as u32) << 12
    };

    let operand1 = IrValue::Register(ty, operand.rn.raw());

    let result = bb.new_variable(ty);

    if operand.rd == AArch64Register::Sp {
        bb.push_inst(IrInst::ZextCast {
            dst: result,
            src: result,
        })
    }

    bb.push_inst(IrInst::Add {
        dst: IrValue::Register(ty, operand.rd.raw()),
        lhs: operand1,
        rhs: IrValue::Constant(IrConstant::new(ty, imm)),
    });
}

fn compile_adds_imm(bb: &mut BasicBlock, operand: &ShImm12RnRd, ty: IrType) {
    todo!()
}

fn compile_add_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_adds_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_add_ext_reg(bb: &mut BasicBlock, operand: &AddSubtractExtReg, ty: IrType) {
    todo!()
}

fn compile_sub_imm(bb: &mut BasicBlock, operand: &ShImm12RnRd, ty: IrType) {
    if operand.sh == 0b0 {}
    todo!()
}

fn compile_sub_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    let shift_type = decode_shift(operand.shift);
    let shift_amount = operand.imm6;

    let operand1 = IrValue::Register(ty, operand.rn.raw());
    let operand2 = bb.new_variable(ty);

    bb.push_inst(shift_reg(
        IrValue::Register(ty, operand.rm.raw()),
        shift_type,
        IrValue::Constant(IrConstant::B8(shift_amount)),
        ty,
        operand2,
    ));

    bb.push_inst(IrInst::Not {
        dst: operand2,
        src: operand2,
    });

    bb.push_inst(IrInst::Sub {
        dst: IrValue::Register(ty, operand.rd.raw()),
        lhs: operand1,
        rhs: operand2,
    })
}

fn compile_subs_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_subs_ext_reg(bb: &mut BasicBlock, operand: &AddSubtractExtReg, ty: IrType) {
    todo!()
}

fn compile_subs_imm(bb: &mut BasicBlock, operand: &ShImm12RnRd, ty: IrType) {
    todo!()
}

fn compile_madd(bb: &mut BasicBlock, operand: &DataProc3Src, ty: IrType) {
    todo!()
}

fn compile_msub(bb: &mut BasicBlock, operand: &DataProc3Src, ty: IrType) {
    todo!()
}

fn compile_div(bb: &mut BasicBlock, operand: &DataProc2Src, ty: IrType) {
    todo!()
}

fn compile_ubfm(bb: &mut BasicBlock, operand: &Bitfield, ty: IrType) {
    todo!()
}

fn compile_sbfm(bb: &mut BasicBlock, operand: &Bitfield, ty: IrType) {
    todo!()
}

fn compile_and_imm(bb: &mut BasicBlock, operand: &LogicalImm, ty: IrType) {
    todo!()
}

fn compile_ands_imm(bb: &mut BasicBlock, operand: &LogicalImm, ty: IrType) {
    todo!()
}

fn compile_ands_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_and_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_orr_imm(bb: &mut BasicBlock, operand: &LogicalImm, ty: IrType) {
    todo!()
}

fn compile_orr_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    let shift_type = decode_shift(operand.shift);
    let shift_amount = operand.imm6;

    let operand1 = IrValue::Register(ty, operand.rn.raw());
    let operand2 = bb.new_variable(ty);

    bb.push_inst(shift_reg(
        IrValue::Register(ty, operand.rm.raw()),
        shift_type,
        IrValue::Constant(IrConstant::B8(shift_amount)),
        ty,
        operand2,
    ));

    bb.push_inst(IrInst::Or {
        dst: IrValue::Register(ty, operand.rd.raw()),
        lhs: operand1,
        rhs: operand2,
    })
}

fn compile_orn_shifted_reg(bb: &mut BasicBlock, operand: &ShiftRmImm6RnRd, ty: IrType) {
    todo!()
}

fn compile_lslv(bb: &mut BasicBlock, operand: &DataProc2Src, ty: IrType) {
    todo!()
}

fn compile_bl_imm(bb: &mut BasicBlock, operand: &Imm26) {
    let offset = sign_extend((operand.imm26 as i64) << 2, 28) as u64;

    let x_30 = IrValue::Register(IrType::B64, AArch64Register::X(30).raw());
    bb.push_inst(IrInst::Assign {
        dst: x_30,
        src: IrValue::Register(IrType::B64, AArch64Register::Pc.raw()),
    });

    let temp = bb.new_variable(IrType::B64);
    bb.push_inst(IrInst::Add {
        dst: temp,
        lhs: IrValue::Register(IrType::B64, AArch64Register::Pc.raw()),
        rhs: IrValue::Constant(IrConstant::B64(offset)),
    });

    bb.set_terminator(BasicBlockTerminator::Branch(temp));
}

fn compile_b_imm(bb: &mut BasicBlock, operand: &Imm26) {
    todo!()
}

fn compile_br(bb: &mut BasicBlock, operand: &UncondBranchReg) {
    todo!()
}

fn compile_blr(bb: &mut BasicBlock, operand: &UncondBranchReg) {
    todo!()
}

fn compile_b_cond(bb: &mut BasicBlock, operand: &Imm19Cond) {
    todo!()
}

fn compile_cbz(bb: &mut BasicBlock, operand: &Imm19Rt, ty: IrType) {
    todo!()
}

fn compile_cbnz(bb: &mut BasicBlock, operand: &Imm19Rt, ty: IrType) {
    todo!()
}

fn compile_ret(bb: &mut BasicBlock, operand: &UncondBranchReg) {
    todo!()
}

fn compile_tbz(bb: &mut BasicBlock, operand: &B5B40Imm14Rt) {
    todo!()
}

fn compile_tbnz(bb: &mut BasicBlock, operand: &B5B40Imm14Rt) {
    todo!()
}

fn compile_ccmp_imm(bb: &mut BasicBlock, operand: &CondCmpImm, ty: IrType) {
    todo!()
}

fn compile_ccmp_reg(bb: &mut BasicBlock, operand: &CondCmpReg, ty: IrType) {
    todo!()
}

fn compile_ccmn_imm(bb: &mut BasicBlock, operand: &CondCmpImm, ty: IrType) {
    todo!()
}

fn compile_csel(bb: &mut BasicBlock, operand: &RmCondRnRd, ty: IrType) {
    todo!()
}

fn compile_csinv(bb: &mut BasicBlock, operand: &RmCondRnRd, ty: IrType) {
    todo!()
}

fn compile_svc(bb: &mut BasicBlock, operand: &ExceptionGen) {
    todo!()
}

fn compile_brk(bb: &mut BasicBlock, operand: &ExceptionGen) {
    todo!()
}

fn compile_mrs(bb: &mut BasicBlock, operand: &SysRegMov) {
    todo!()
}

fn compile_msr_reg(bb: &mut BasicBlock, operand: &SysRegMov) {
    todo!()
}

fn compile_msr_imm(bb: &mut BasicBlock, operand: &PstateOp) {
    todo!()
}
