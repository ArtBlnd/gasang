use std::collections::HashMap;

use crate::compiler::aarch64_prelude::*;
use crate::compiler::Compiler;
use crate::ir::*;
use crate::register::RegId;
use crate::value::Value;

use machineinstr::aarch64::*;

pub struct AArch64Compiler {
    register_info: HashMap<String, RegId>,
}

impl AArch64Compiler {
    pub fn new(reg_info: HashMap<String, RegId>) -> Self {
        Self {
            register_info: reg_info,
        }
    }
    pub fn gpr(&self, index: u8) -> RegId {
        self.register_info
            .get(&format!("x{}", index))
            .unwrap()
            .clone()
    }

    pub fn fpr(&self, index: u8) -> RegId {
        self.register_info
            .get(&format!("v{}", index))
            .unwrap()
            .clone()
    }

    pub fn stack_reg(&self) -> RegId {
        self.register_info.get("sp").unwrap().clone()
    }

    pub fn reg_by_name(&self, name: impl AsRef<str>) -> RegId {
        self.register_info.get(name.as_ref()).unwrap().clone()
    }
}

impl Compiler for AArch64Compiler {
    type Item = AArch64Instr;

    fn compile(&self, item: Self::Item) -> IrBlock {
        println!("{item:?}");

        match item {
            AArch64Instr::MovzVar32(operand) | AArch64Instr::MovzVar64(operand) => {
                gen_movz(self, operand)
            }
            AArch64Instr::MovnVar32(operand) => gen_movn(self, operand, Type::U32),
            AArch64Instr::MovnVar64(operand) => gen_movn(self, operand, Type::U64),
            AArch64Instr::MovkVar32(operand) => gen_movk(self, operand, Type::U32),
            AArch64Instr::MovkVar64(operand) => gen_movk(self, operand, Type::U64),
            AArch64Instr::MoviVectorVar64(operand) => gen_movi(self, operand),
            AArch64Instr::Adr(operand) => gen_adr(self, operand),
            AArch64Instr::Adrp(operand) => gen_adrp(self, operand),
            AArch64Instr::OrrShiftedReg64(operand) => gen_orr_shifted_reg(self, operand, Type::U64),
            AArch64Instr::OrrShiftedReg32(operand) => gen_orr_shifted_reg(self, operand, Type::U32),

            // Load and Stores
            AArch64Instr::LdrImm32(operand) => gen_ldr_imm(self, operand, Type::U32),
            AArch64Instr::LdrImm64(operand) => gen_ldr_imm(self, operand, Type::U64),
            AArch64Instr::LdrImmSimdFP64(operand) => gen_ldr_imm_simd_fp(self, operand, Type::U64),
            AArch64Instr::LdrImmSimdFP128(operand) => gen_ldr_imm_simd_fp(self, operand, Type::Vec(VecType::U64, 2)),
            AArch64Instr::LdrLitVar64(operand) => gen_ldr_lit_var64(self, operand),
            AArch64Instr::LdrhImm(operand) => gen_ldrh_imm(self, operand),
            AArch64Instr::LdrbImm(operand) => gen_ldrb_imm(self, operand),
            AArch64Instr::LdrReg32(operand) => gen_ldr_reg(self, operand, Type::U32),
            AArch64Instr::LdrReg64(operand) => gen_ldr_reg(self, operand, Type::U64),
            AArch64Instr::LdrbRegShiftedReg(operand) => gen_ldrb_reg_shifted_reg(self, operand),
            AArch64Instr::LdpVar64(operand) => gen_ldp(self, operand, Type::U64),
            AArch64Instr::LdrshReg64(operand) => gen_ldrsh_reg(self, operand, Type::U64),
            AArch64Instr::LdrshReg32(operand) => gen_ldrsh_reg(self, operand, Type::U32),
            AArch64Instr::LdaxrVar32(operand) => gen_ldaxr(self, operand, Type::U32),
            AArch64Instr::LdarVar64(operand) => gen_ldar(self, operand, Type::U64),
            AArch64Instr::LdpSimdFpVar128(operand) => gen_ldp_simd_fp(self, operand, Type::Vec(VecType::U64, 2)),

            AArch64Instr::StrImm32(operand) => gen_str_imm(self, operand, Type::U32),
            AArch64Instr::StrImm64(operand) => gen_str_imm(self, operand, Type::U64),
            AArch64Instr::StpVar64(operand) => gen_stp_var(self, operand, Type::U64),
            AArch64Instr::StpVar32(operand) => gen_stp_var(self, operand, Type::U32),
            AArch64Instr::StrbImm(operand) => gen_strb_imm(self, operand),
            AArch64Instr::Sturb(operand) => gen_sturb_imm(self, operand),
            AArch64Instr::StrReg32(operand) => gen_str_reg(self, operand, Type::U32),
            AArch64Instr::StrReg64(operand) => gen_str_reg(self, operand, Type::U64),
            AArch64Instr::Stur32(operand) => gen_stur(self, operand, Type::U32),
            AArch64Instr::Stur64(operand) => gen_stur(self, operand, Type::U64),
            AArch64Instr::SturSimdFP64(operand) => gen_stur_simd_fp(self, operand, Type::U64),
            AArch64Instr::SturSimdFP128(operand) => {
                gen_stur_simd_fp(self, operand, Type::Vec(VecType::U64, 2))
            }
            AArch64Instr::StpSimdFpVar128(operand) => {
                gen_stp_simd_fp(self, operand, Type::Vec(VecType::U64, 2))
            }
            AArch64Instr::StrImmSimdFP64(operand) => gen_str_imm_simd_fp(self, operand, Type::U64),
            AArch64Instr::StrImmSimdFP128(operand) => {
                gen_str_imm_simd_fp(self, operand, Type::Vec(VecType::U64, 2))
            }
            AArch64Instr::StlxrVar32(operand) => gen_stlxr(self, operand, Type::U32),

            // Advanced SIMD and FP
            AArch64Instr::DupGeneral(operand) => gen_dup_general(self, operand),

            // Arithmetic instructions
            AArch64Instr::AddImm64(operand) => gen_add_imm(self, operand, Type::U64),
            AArch64Instr::AddImm32(operand) => gen_add_imm(self, operand, Type::U32),
            AArch64Instr::AddsImm64(operand) => gen_adds_imm(self, operand, Type::U64),
            AArch64Instr::AddsImm32(operand) => gen_adds_imm(self, operand, Type::U32),
            AArch64Instr::AddShiftedReg64(operand) => gen_add_shifted_reg64(self, operand),
            AArch64Instr::AddsShiftedReg64(operand) => {
                gen_adds_shifted_reg(self, operand, Type::U64)
            }
            AArch64Instr::AddExtReg64(operand) => gen_add_ext_reg64(self, operand),
            AArch64Instr::SubImm64(operand) => gen_sub_imm(self, operand, Type::U64),
            AArch64Instr::SubImm32(operand) => gen_sub_imm(self, operand, Type::U32),
            AArch64Instr::SubShiftedReg64(operand) => gen_sub_shifted_reg_64(self, operand),
            AArch64Instr::SubsShiftedReg32(operand) => {
                gen_subs_shifted_reg(self, operand, Type::U32)
            }
            AArch64Instr::SubsShiftedReg64(operand) => {
                gen_subs_shifted_reg(self, operand, Type::U64)
            }
            AArch64Instr::SubsImm64(operand) => gen_subs_imm(self, operand, Type::U64),
            AArch64Instr::SubsImm32(operand) => gen_subs_imm(self, operand, Type::U32),
            AArch64Instr::Madd32(operand) => gen_madd(self, operand, Type::U32),
            AArch64Instr::Madd64(operand) => gen_madd(self, operand, Type::U64),
            AArch64Instr::SdivVar32(operand) => gen_div(self, operand, Type::I32),
            AArch64Instr::Msub32(operand) => gen_msub(self, operand, Type::U32),

            // bitwise isntructions
            AArch64Instr::Ubfm32(operand) => gen_ubfm(self, operand, Type::U32),
            AArch64Instr::Ubfm64(operand) => gen_ubfm(self, operand, Type::U64),
            AArch64Instr::Sbfm64(operand) => gen_sbfm(self, operand, Type::U64),
            AArch64Instr::AndImm64(operand) => gen_and_imm(self, operand, Type::U64),
            AArch64Instr::AndImm32(operand) => gen_and_imm(self, operand, Type::U32),
            AArch64Instr::AndsImm64(operand) => gen_ands_imm64(self, operand),
            AArch64Instr::AndsShiftedReg32(operand) => {
                gen_ands_shifted_reg(self, operand, Type::U32)
            }
            AArch64Instr::AndsShiftedReg64(operand) => {
                gen_ands_shifted_reg(self, operand, Type::U64)
            }
            AArch64Instr::AndShiftedReg64(operand) => gen_and_shifted_reg(self, operand, Type::U64),
            AArch64Instr::OrrImm64(operand) => gen_orr_imm(self, operand, Type::U64),
            AArch64Instr::OrrImm32(operand) => gen_orr_imm(self, operand, Type::U32),
            AArch64Instr::OrnShiftedReg64(operand) => gen_orn_shifted_reg(self, operand, Type::U64),

            AArch64Instr::LslvVar64(operand) => gen_lslv(self, operand, Type::U64),
            AArch64Instr::LslvVar32(operand) => gen_lslv(self, operand, Type::U32),

            // Branch instructions
            AArch64Instr::BlImm(operand) => gen_bl_imm(self, operand),
            AArch64Instr::BImm(operand) => gen_b_imm(self, operand),
            AArch64Instr::Br(operand) => gen_br(self, operand),
            AArch64Instr::Blr(operand) => gen_blr(self, operand),
            AArch64Instr::BCond(operand) => gen_b_cond(self, operand),
            AArch64Instr::Cbz64(operand) => gen_cbz(self, operand, Type::U32),
            AArch64Instr::Cbz32(operand) => gen_cbz(self, operand, Type::U64),
            AArch64Instr::Cbnz32(operand) => gen_cbnz(self, operand, Type::U32),
            AArch64Instr::Cbnz64(operand) => gen_cbnz(self, operand, Type::U64),
            AArch64Instr::Ret(operand) => gen_ret(self, operand),
            AArch64Instr::Tbz(operand) => gen_tbz(self, operand),
            AArch64Instr::Tbnz(operand) => gen_tbnz(self, operand),

            // Conditional Instructions
            AArch64Instr::CcmpImmVar32(operand) => gen_ccmp_imm(self, operand, Type::U32),
            AArch64Instr::CcmpImmVar64(operand) => gen_ccmp_imm(self, operand, Type::U64),
            AArch64Instr::CcmnImmVar64(operand) => gen_ccmn_imm(self, operand, Type::U64),
            AArch64Instr::Csel32(operand) => gen_csel32(self, operand),

            // Interrupt Instructions
            AArch64Instr::Svc(operand) => gen_svc(self, operand),
            AArch64Instr::Brk(operand) => gen_brk(self, operand),

            // Speical instructions
            AArch64Instr::Mrs(operand) => gen_mrs(self, operand),
            AArch64Instr::MsrReg(operand) => gen_msr_reg(self, operand),
            AArch64Instr::MsrImm(operand) => gen_msr_imm(self, operand),
            AArch64Instr::Nop | AArch64Instr::Wfi | AArch64Instr::Dmb(_) => {
                let mut block = IrBlock::new(4);

                let ir = Ir::Nop;
                let ds = BlockDestination::None;
                block.append(ir, ds);

                block
            }

            _ => unimplemented!("unimplemented instruction: {:?}", item),
        }
    }
}

fn gen_movz(compiler: &AArch64Compiler, operand: HwImm16Rd) -> IrBlock {
    let mut block = IrBlock::new(4);
    let pos = operand.hw << 4;

    let ir = Ir::Value(Operand::imm(Type::U64, (operand.imm16 as u64) << pos));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
    block.append(ir, ds);

    block
}

fn gen_adr(compiler: &AArch64Compiler, operand: PcRelAddressing) -> IrBlock {
    let mut block = IrBlock::new(4);
    let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 21);

    let ir = gen_ip_relative(imm);
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
    block.append(ir, ds);

    block
}

fn gen_adrp(compiler: &AArch64Compiler, operand: PcRelAddressing) -> IrBlock {
    let mut block = IrBlock::new(4);

    let imm = sign_extend(
        ((operand.immhi as i64) << 2 | (operand.immlo as i64)) << 12,
        33,
    );

    let ir = Ir::Add(
        Type::U64,
        Operand::ir(Ir::And(
            Type::U64,
            Operand::Ip,
            Operand::imm(Type::U64, 0xFFFF_FFFF_FFFF_F000),
        )),
        Operand::imm(Type::I64, imm as u64),
    );
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
    block.append(ir, ds);

    block
}

fn gen_orr_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);
    let rd = compiler.gpr(operand.rd);
    let rm = if operand.rm == 31 {
        Operand::Immediate(ty, 0)
    } else {
        Operand::Gpr(ty, compiler.gpr(operand.rm))
    };

    let shift_type = decode_shift(operand.shift);

    let op1 = if operand.rn == 31 {
        Operand::Immediate(ty, 0)
    } else {
        Operand::Gpr(ty, compiler.gpr(operand.rn))
    };

    let amount = Operand::Immediate(ty, operand.imm6 as u64);

    let op2 = Operand::ir(shift_reg(rm, shift_type, amount, ty));

    let ir = Ir::Or(ty, op1, op2);
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, rd);

    block.append(ir, ds);

    block
}

fn gen_ldr_imm(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (mut wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, false);
    let pre_offs = if post_index { 0 } else { offset };

    if wback && operand.rn == operand.rt && operand.rn != 31 {
        wback = false;
    }

    let dst = compiler.gpr(operand.rt);
    let src = if operand.rn == 31 {
        // If rn is 31, we use stack register instead of gpr registers.
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Load(
        ty,
        Operand::ir(Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, src),
            Operand::imm(Type::U64, pre_offs as u64),
        )),
    );
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, dst);

    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}

fn gen_str_imm(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, false);
    let pre_offs = if post_index { 0 } else { offset };

    let rn = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let rt = if operand.rt == 31 {
        Operand::imm(ty, 0)
    } else {
        Operand::gpr(ty, compiler.gpr(operand.rt))
    };

    let ir = Ir::Value(rt);
    let ds = BlockDestination::MemoryRelI64(ty, rn, pre_offs);
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, rn),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, rn);

        block.append(ir, ds);
    }

    block
}

fn gen_ldr_lit_var64(compiler: &AArch64Compiler, operand: Imm19Rt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let offset = sign_extend((operand.imm19 << 2) as i64, 21);

    let ir = Ir::Load(Type::U64, Operand::ir(gen_ip_relative(offset)));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_stp_var(compiler: &AArch64Compiler, operand: LoadStoreRegPair, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let scale = 2 + (operand.opc >> 1);

    let (wback, post_index) = decode_o_for_ld_st_pair_offset(operand.o);
    let offset = sign_extend(operand.imm7 as i64, 7) << scale;

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let data1 = if operand.rt == 31 {
        Operand::Immediate(ty, 0)
    } else {
        Operand::Gpr(ty, compiler.gpr(operand.rt))
    };

    let data2 = if operand.rt2 == 31 {
        Operand::Immediate(ty, 0)
    } else {
        Operand::Gpr(ty, compiler.gpr(operand.rt2))
    };

    let ir = Ir::Value(data1);
    let ds = BlockDestination::MemoryRelI64(ty, dst, offset_temp);
    block.append(ir, ds);

    let ir = Ir::Value(data2);
    let ds = BlockDestination::MemoryRelI64(ty, dst, offset_temp + 8);
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, dst),
            Operand::Immediate(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, dst);
        block.append(ir, ds)
    }

    block
}

fn gen_add_imm(compiler: &AArch64Compiler, operand: ShImm12RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rd = if operand.rd == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rd)
    };

    let rn = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let imm = match operand.sh {
        0b0 => operand.imm12 as u64,
        0b1 => (operand.imm12 as u64) << 12,
        _ => unreachable!(),
    };

    let ir = Ir::Add(ty, Operand::gpr(ty, rn), Operand::imm(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, rd);

    block.append(ir, ds);

    block
}

fn gen_add_shifted_reg64(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = compiler.gpr(operand.rn);
    let rm = compiler.gpr(operand.rm);
    let rd = compiler.gpr(operand.rd);

    let amount = Operand::Immediate(Type::U64, operand.imm6 as u64);

    let sh = shift_reg(
        Operand::Gpr(Type::U64, rm),
        decode_shift(operand.shift),
        amount,
        Type::U64,
    );
    let ir = Ir::Add(
        Type::U64,
        Operand::gpr(Type::U64, rn),
        Operand::Ir(Box::new(sh)),
    );

    let ds = BlockDestination::Gpr(Type::U64, rd);
    block.append(ir, ds);

    block
}

fn gen_sub_imm(compiler: &AArch64Compiler, operand: ShImm12RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let imm = match operand.sh {
        0b00 => operand.imm12 as u64,
        0b01 => (operand.imm12 as u64) << 12,
        _ => unreachable!(),
    };

    let ir = Ir::Sub(ty, Operand::gpr(ty, rn), Operand::imm(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));

    let ds = if operand.rd == 31 {
        BlockDestination::None
    } else {
        BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd))
    };

    block.append(ir, ds);

    block
}

fn gen_sub_shifted_reg_64(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rm = compiler.gpr(operand.rm);

    let amount = Operand::Immediate(Type::U64, operand.imm6 as u64);

    let sh = shift_reg(
        Operand::Gpr(Type::U64, rm),
        decode_shift(operand.shift),
        amount,
        Type::U64,
    );

    let rn = if operand.rn == 31 {
        Operand::Immediate(Type::U64, 0)
    } else {
        Operand::gpr(Type::U64, compiler.gpr(operand.rn))
    };

    let ir = Ir::Sub(Type::U64, rn, Operand::Ir(Box::new(sh)));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_subs_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = if operand.rn == 31 {
        Operand::imm(ty, 0)
    }
    else {
        Operand::gpr(ty, compiler.gpr(operand.rn))
    };
    let rm = compiler.gpr(operand.rm);

    let amount = Operand::Immediate(ty, operand.imm6 as u64);

    let sh = shift_reg(
        Operand::Gpr(Type::U64, rm),
        decode_shift(operand.shift),
        amount,
        ty,
    );
    let ir = Ir::Subc(ty, rn, Operand::Ir(Box::new(sh)));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));

    let ds = if operand.rd == 31 {
        BlockDestination::None
    } else {
        BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd))
    };

    block.append(ir, ds);

    block
}

fn gen_subs_imm(compiler: &AArch64Compiler, operand: ShImm12RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let imm = match operand.sh {
        0b00 => operand.imm12 as u64,
        0b01 => (operand.imm12 as u64) << 12,
        _ => unreachable!(),
    };

    let rn = if operand.rn == 0b11111 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Subc(ty, Operand::gpr(ty, rn), Operand::imm(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    // If rd is 31, its alias is CMP(immediate).
    let ds = if operand.rd == 0b11111 {
        BlockDestination::None
    } else {
        BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd))
    };

    block.append(ir, ds);

    block
}

fn gen_ands_imm64(compiler: &AArch64Compiler, operand: LogicalImm) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (imm, _) = decode_bit_masks(operand.n, operand.imms, operand.immr, true, 64);
    let rn = Operand::gpr(Type::U64, compiler.gpr(operand.rn));

    let ir = Ir::And(Type::U64, rn, Operand::imm(Type::U64, imm));

    let ds = if operand.rd == 31 {
        BlockDestination::None
    } else {
        BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd))
    };
    block.append(ir.clone(), ds);

    let ds = BlockDestination::None;
    let ir = Ir::Addc(Type::U64, Operand::ir(ir), Operand::imm(Type::U64, 0)); // Only for flag setting
    block.append(ir, ds);

    block
}

fn gen_bl_imm(compiler: &AArch64Compiler, operand: Imm26) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Add(Type::U64, Operand::Ip, Operand::imm(Type::U64, 4));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(30));

    block.append(ir, ds);

    let imm = sign_extend((operand.imm26 << 2) as i64, 28);

    let ir = gen_ip_relative(imm);
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_b_imm(_compiler: &AArch64Compiler, operand: Imm26) -> IrBlock {
    let mut block = IrBlock::new(4);

    let imm = sign_extend((operand.imm26 << 2) as i64, 28);

    let ir = gen_ip_relative(imm);
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_br(compiler: &AArch64Compiler, operand: UncondBranchReg) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Value(Operand::gpr(Type::U64, compiler.gpr(operand.rn)));
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_b_cond(_compiler: &AArch64Compiler, operand: Imm19Cond) -> IrBlock {
    let mut block = IrBlock::new(4);

    let offset = sign_extend((operand.imm19 << 2) as i64, 21);
    let ir = Ir::If(
        Type::U64,
        condition_holds(operand.cond),
        Operand::ir(gen_ip_relative(offset as i64)),
        Operand::ir(gen_ip_relative(4)),
    );
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_cbz(compiler: &AArch64Compiler, operand: Imm19Rt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let offset = sign_extend((operand.imm19 << 2) as i64, 21);

    let is_zero = if ty == Type::U64 {
        cmp_eq_op_imm64(Operand::Gpr(ty, compiler.gpr(operand.rt)), 0)
    } else {
        cmp_eq_op_imm32(Operand::Gpr(ty, compiler.gpr(operand.rt)), 0)
    };

    let ir = Ir::If(
        Type::U64,
        is_zero,
        Operand::ir(gen_ip_relative(offset)),
        Operand::ir(gen_ip_relative(4)),
    );
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_cbnz(compiler: &AArch64Compiler, operand: Imm19Rt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let offset = sign_extend((operand.imm19 << 2) as i64, 21);

    let is_zero = if ty == Type::U64 {
        cmp_eq_op_imm64(Operand::Gpr(ty, compiler.gpr(operand.rt)), 0)
    } else {
        cmp_eq_op_imm32(Operand::Gpr(ty, compiler.gpr(operand.rt)), 0)
    };

    let ir = Ir::If(
        Type::U64,
        is_zero,
        Operand::ir(gen_ip_relative(4)),
        Operand::ir(gen_ip_relative(offset)),
    );
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_ccmp_imm(compiler: &AArch64Compiler, operand: CondCmpImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = compiler.gpr(operand.rn);

    let subc = Operand::void_ir(Ir::Subc(
        ty,
        Operand::gpr(ty, rn),
        Operand::imm(ty, operand.imm5 as u64),
    ));

    let ir = Ir::If(
        Type::U64,
        condition_holds(operand.cond),
        Operand::ir(Ir::Or(
            Type::U64,
            Operand::ir(Ir::BitCast(Type::U64, subc)),
            Operand::Flag,
        )),
        Operand::ir(replace_bits(
            Operand::Flag,
            operand.nzcv as u64,
            Pstate::NZCV.range(),
        )),
    );
    let ds = BlockDestination::Flags;

    block.append(ir, ds);

    block
}

fn gen_csel32(compiler: &AArch64Compiler, operand: RmCondRnRd) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = if operand.rn == 31 {
        Operand::imm(Type::U32, 0)
    } else {
        Operand::gpr(Type::U32, compiler.gpr(operand.rn))
    };

    let rm = if operand.rm == 31 {
        Operand::imm(Type::U32, 0)
    } else {
        Operand::gpr(Type::U32, compiler.gpr(operand.rm))
    };
    let rd = compiler.gpr(operand.rd);

    let ir = Ir::If(Type::U32, condition_holds(operand.cond), rn, rm);
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, rd);

    block.append(ir, ds);

    block
}

fn gen_svc(_compiler: &AArch64Compiler, operand: ExceptionGen) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Value(Operand::imm(Type::U64, operand.imm16 as u64));
    let ds = BlockDestination::SystemCall;

    block.append(ir, ds);

    block
}

fn gen_brk(_compiler: &AArch64Compiler, operand: ExceptionGen) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Value(Operand::imm(Type::U16, operand.imm16 as u64));
    let ds = BlockDestination::Exit;

    block.append(ir, ds);

    block
}

fn gen_ubfm(compiler: &AArch64Compiler, operand: Bitfield, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let src = Operand::Gpr(ty, compiler.gpr(operand.rn));
    let r = Operand::Immediate(Type::U8, operand.immr as u64);

    let (wmask, tmask) = decode_bit_masks(
        operand.n,
        operand.imms,
        operand.immr,
        false,
        (ty.size() * 8) as u8,
    );

    let bot = Ir::And(
        ty,
        Operand::ir(Ir::Rotr(ty, src, r)),
        Operand::Immediate(ty, wmask),
    );
    let ir = Ir::And(ty, Operand::ir(bot), Operand::Immediate(ty, tmask));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_sbfm(compiler: &AArch64Compiler, operand: Bitfield, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let src = Operand::Gpr(ty, compiler.gpr(operand.rn));
    let r = Operand::Immediate(Type::U8, operand.immr as u64);
    let datasize = (ty.size() * 8) as u8;

    let (wmask, tmask) = decode_bit_masks(operand.n, operand.imms, operand.immr, false, datasize);

    let bot = Ir::And(
        ty,
        Operand::ir(Ir::Rotr(ty, src, r)),
        Operand::Immediate(ty, wmask),
    );

    let top = replicate_reg64(compiler.gpr(operand.rn), operand.imms);

    let lhs = Ir::And(ty, Operand::ir(top), Operand::Immediate(ty, !tmask));
    let rhs = Ir::And(ty, Operand::ir(bot), Operand::Immediate(ty, tmask));

    let ir = Ir::Or(ty, Operand::ir(lhs), Operand::ir(rhs));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_ldrb_imm(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, false);

    let pre_offs = if post_index { 0 } else { offset };

    let dst = compiler.gpr(operand.rt);
    let src = if operand.rn == 31 {
        // If rn is 31, we use stack register instead of gpr registers.
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Load(
        Type::U8,
        Operand::ir(Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, src),
            Operand::imm(Type::U64, pre_offs as u64),
        )),
    );
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, dst);

    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}

fn gen_ret(compiler: &AArch64Compiler, operand: UncondBranchReg) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Value(Operand::gpr(Type::U64, compiler.gpr(operand.rn)));
    let ds = BlockDestination::Ip;
    block.append(ir, ds);

    block
}

fn gen_add_ext_reg64(compiler: &AArch64Compiler, operand: AddSubtractExtReg) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ext_type = decode_reg_extend(operand.option);
    let shift = operand.imm3;
    assert!(shift <= 4);

    let op1 = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };
    let op2 = extend_reg(compiler.gpr(operand.rm), ext_type, shift, 64 / 8);

    let ir = Ir::Add(Type::U64, Operand::gpr(Type::U64, op1), Operand::ir(op2));
    let ds = if operand.rd == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rd)
    };
    let ds = BlockDestination::Gpr(Type::U64, ds);

    block.append(ir, ds);

    block
}

fn gen_ldrh_imm(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (_wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, false);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let ir = Ir::Load(
        Type::U16,
        Operand::ir(Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, src),
            Operand::Immediate(Type::I64, offset_temp as u64),
        )),
    );

    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_mrs(compiler: &AArch64Compiler, operand: SysRegMov) -> IrBlock {
    let mut block = IrBlock::new(4);

    // TODO: emulate system registers
    let op = match (
        operand.o0 + 2,
        operand.op1,
        operand.crn,
        operand.crm,
        operand.op2,
    ) {
        (0b11, 0b011, 0b1101, 0b0000, 0b010) => {
            Operand::Sys(Type::U64, compiler.reg_by_name("tpidr_el0"))
        } // tpidr_el0, get current thread.
        (0b11, 0b011, 0b0000, 0b0000, 0b111) => {
            let implementer = 0; // Reserved for software use
            let variant = 0;
            let architecture = 0b1111; // Architectural features are individually identified in the ID_* registers, see 'ID registers'.
            let partnum = 0;
            let revision = 0;

            let ret =
                implementer << 24 | variant << 20 | architecture << 16 | partnum << 4 | revision;
            Operand::imm(Type::U64, ret)
        }
        (0b11, 0b000, 0b0100, 0b0010, 0b010) => {
            // Get current exception level.
            // We are using exception level one.
            let current_exception_level = 1;
            Operand::imm(Type::U64, current_exception_level << 2)
        }
        (0b11, 0b000, 0b1100, 0b0000, 0b000) => {
            Operand::Sys(Type::U64, compiler.reg_by_name("vbar_el1"))
        }
        _ => unimplemented!("MRS: {:?}", operand),
    };

    let ir = Ir::Value(op);
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_msr_reg(compiler: &AArch64Compiler, operand: SysRegMov) -> IrBlock {
    let mut block = IrBlock::new(4);

    // TODO: emulate system registers
    let dest = match (
        operand.o0 + 2,
        operand.op1,
        operand.crn,
        operand.crm,
        operand.op2,
    ) {
        (0b11, 0b011, 0b1101, 0b0000, 0b010) => compiler.reg_by_name("tpidr_el0"), // tpidr_el0, get current thread.
        (0b11, 0b000, 0b1100, 0b0000, 0b000) => compiler.reg_by_name("vbar_el1"),
        (0b11, 0b000, 0b0001, 0b0000, 0b010) => compiler.reg_by_name("cpacr_el1"),
        _ => unimplemented!("MSR: {:x?}", operand),
    };

    let ir = Ir::Value(Operand::Gpr(Type::U64, compiler.gpr(operand.rt)));
    let ds = BlockDestination::Sys(Type::U64, dest);

    block.append(ir, ds);

    block
}

fn gen_ldr_reg(compiler: &AArch64Compiler, operand: LoadStoreRegRegOffset, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ext_type = decode_reg_extend(operand.option);
    let shift = if operand.s == 1 { operand.size } else { 0 };

    let offset = extend_reg(compiler.gpr(operand.rm), ext_type, shift, ty.size() as u8);
    let offset = Ir::SextCast(Type::I64, Operand::ir(offset));

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Load(
        ty,
        Operand::ir(Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, src),
            Operand::ir(offset),
        )),
    );
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_blr(compiler: &AArch64Compiler, operand: UncondBranchReg) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ir = Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(Type::U64, 4));
    let ds = BlockDestination::Gpr(Type::U64, RegId(30));

    block.append(ir, ds);

    let ir = Ir::Value(Operand::Gpr(Type::U64, compiler.gpr(operand.rn)));
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_ldp(compiler: &AArch64Compiler, operand: LoadStoreRegPair, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (mut wback, post_index) = decode_o_for_ld_st_pair_offset(operand.o);

    if wback && (operand.rt == operand.rn || operand.rt2 == operand.rn) && operand.rn != 31 {
        wback = false;
    }

    let scale = 2 + (operand.opc >> 1);
    let signed = operand.opc & 0b1 != 0;
    let offset = sign_extend(operand.imm7 as i64, 7) << scale;
    let dbytes = ty.size();

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let address = Ir::Add(
        Type::U64,
        Operand::Gpr(Type::U64, src),
        Operand::Immediate(Type::I64, offset_temp as u64),
    );

    let ir = Ir::Load(ty, Operand::ir(address.clone()));
    let ir = if signed {
        Ir::SextCast(Type::I64, Operand::ir(ir))
    } else {
        ir
    };

    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));
    block.append(ir, ds);

    let ir = Ir::Load(
        ty,
        Operand::ir(Ir::Add(
            ty,
            Operand::ir(address),
            Operand::Immediate(Type::U64, dbytes as u64),
        )),
    );
    let ir = if signed {
        Ir::SextCast(Type::I64, Operand::ir(ir))
    } else {
        ir
    };
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt2));
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}

fn gen_ands_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let shift_type = decode_shift(operand.shift);

    let src = Operand::Gpr(ty, compiler.gpr(operand.rn));

    let amount = Operand::Immediate(ty, operand.imm6 as u64);
    let operand2 = shift_reg(
        Operand::Gpr(ty, compiler.gpr(operand.rm)),
        shift_type,
        amount,
        ty,
    );

    let ir = Ir::And(ty, src, Operand::ir(operand2));

    if operand.rd != 31 {
        let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
        block.append(ir.clone(), ds);
    }

    let ir = Ir::Addc(ty, Operand::ir(ir), Operand::imm(ty, 0)); // Only for flag setting
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::None;
    block.append(ir, ds);

    block
}

fn gen_and_imm(compiler: &AArch64Compiler, operand: LogicalImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (imm, _) = decode_bit_masks(
        operand.n,
        operand.imms,
        operand.immr,
        true,
        ty.size() as u8 * 8,
    );

    let src = Operand::Gpr(ty, compiler.gpr(operand.rn));
    let ir = Ir::And(ty, src, Operand::Immediate(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));

    let ds = BlockDestination::Gpr(
        Type::U64,
        if operand.rd == 31 {
            compiler.stack_reg()
        } else {
            compiler.gpr(operand.rd)
        },
    );

    block.append(ir, ds);

    block
}

fn gen_tbz(compiler: &AArch64Compiler, operand: B5B40Imm14Rt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ty = if operand.b5 == 1 {
        Type::U64
    } else {
        Type::U32
    };

    let b = operand.b5 << 5 | operand.b40;
    let mask = gen_mask64(b..b + 1);
    let offs = sign_extend((operand.imm14 << 2) as i64, 14);

    let op = Ir::And(
        ty,
        Operand::gpr(ty, compiler.gpr(operand.rt)),
        Operand::imm(ty, mask),
    );

    let ir = Ir::If(
        Type::U64,
        Operand::ir(Ir::CmpEq(Operand::ir(op), Operand::imm(ty, 0))),
        Operand::ir(gen_ip_relative(offs)),
        Operand::ir(gen_ip_relative(4)),
    );
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_tbnz(compiler: &AArch64Compiler, operand: B5B40Imm14Rt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ty = if operand.b5 == 1 {
        Type::U64
    } else {
        Type::U32
    };

    let b = operand.b5 << 5 | operand.b40;
    let mask = gen_mask64(b..b + 1);
    let offs = sign_extend((operand.imm14 << 2) as i64, 14);

    let op = Ir::And(
        ty,
        Operand::gpr(ty, compiler.gpr(operand.rt)),
        Operand::imm(ty, mask),
    );

    let ir = Ir::If(
        Type::U64,
        Operand::ir(Ir::CmpEq(Operand::ir(op), Operand::imm(ty, 0))),
        Operand::ir(gen_ip_relative(4)),
        Operand::ir(gen_ip_relative(offs)),
    );
    let ds = BlockDestination::Ip;

    block.append(ir, ds);

    block
}

fn gen_movn(compiler: &AArch64Compiler, operand: HwImm16Rd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let pos = operand.hw << 4;
    let res = !((operand.imm16 as u64) << pos);

    let ir = Ir::Value(Operand::Immediate(ty, res));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_strb_imm(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, false);

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let rt = if operand.rt == 31 {
        Operand::imm(Type::U8, 0)
    } else {
        Operand::Gpr(Type::U8, compiler.gpr(operand.rt))
    };

    let ir = Ir::Value(rt);
    let ds = BlockDestination::MemoryRelI64(Type::U8, dst, offset_temp);
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, dst),
            Operand::Immediate(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, dst);

        block.append(ir, ds);
    }

    block
}

fn gen_sturb_imm(compiler: &AArch64Compiler, operand: LdStRegUnscaledImm) -> IrBlock {
    let mut block = IrBlock::new(4);

    let offset = sign_extend(operand.imm9 as i64, 9);

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Value(Operand::Gpr(Type::U8, compiler.gpr(operand.rt)));
    let ds = BlockDestination::MemoryRelI64(Type::U8, dst, offset);
    block.append(ir, ds);

    block
}

fn gen_orr_imm(compiler: &AArch64Compiler, operand: LogicalImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (imm, _) = decode_bit_masks(
        operand.n,
        operand.imms,
        operand.immr,
        true,
        ty.size() as u8 * 8,
    );

    let rn = if operand.rn == 31 {
        Operand::imm(ty, 0)
    } else {
        Operand::gpr(ty, compiler.gpr(operand.rn))
    };

    let ir = Ir::Or(ty, rn, Operand::imm(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));

    if operand.rd == 31 {
        let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
        let ds = BlockDestination::Gpr(Type::U64, compiler.stack_reg());
        block.append(ir, ds);
    } else {
        let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
        block.append(ir, ds);
    };

    block
}

fn gen_madd(compiler: &AArch64Compiler, operand: DataProc3Src, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let op1 = Operand::gpr(ty, compiler.gpr(operand.rn));
    let op2 = Operand::gpr(ty, compiler.gpr(operand.rm));
    let op3 = if operand.ra == 31 {
        Operand::imm(ty, 0)
    } else {
        Operand::gpr(ty, compiler.gpr(operand.ra))
    };

    let ir = Ir::Add(ty, op3, Operand::ir(Ir::Mul(ty, op1, op2)));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));
    block.append(ir, ds);

    block
}

fn gen_str_reg(compiler: &AArch64Compiler, operand: LoadStoreRegRegOffset, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let shift = if operand.s == 1 { operand.size } else { 0 };
    let rm = compiler.gpr(operand.rm);

    let ext_type = decode_reg_extend(operand.option);
    let offset = extend_reg(rm, ext_type, shift, 8);

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let addr = Ir::Add(Type::U64, Operand::gpr(Type::U64, dst), Operand::ir(offset));

    let ir = Ir::Value(Operand::gpr(ty, compiler.gpr(operand.rt)));
    let ds = BlockDestination::MemoryIr(addr);

    block.append(ir, ds);

    block
}

fn gen_stur(compiler: &AArch64Compiler, operand: LdStRegUnscaledImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };
    let rt = operand.rt;

    let offs = sign_extend(operand.imm9 as i64, 9);

    let ir = Ir::Value(Operand::gpr(ty, compiler.gpr(rt)));
    let ds = BlockDestination::MemoryRelI64(ty, rn, offs);

    block.append(ir, ds);

    block
}

fn gen_and_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let shift_type = decode_shift(operand.shift);

    let op1 = compiler.gpr(operand.rn);

    let amount = Operand::Immediate(ty, operand.imm6 as u64);
    let op2 = shift_reg(
        Operand::Gpr(ty, compiler.gpr(operand.rm)),
        shift_type,
        amount,
        ty,
    );

    let ir = Ir::And(ty, Operand::gpr(ty, op1), Operand::ir(op2));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_dup_general(compiler: &AArch64Compiler, operand: AdvancedSimdCopy) -> IrBlock {
    let mut block = IrBlock::new(4);

    let size = operand.imm5.trailing_zeros();
    assert!(size <= 3);
    let esize = 8 << size;
    let datasize = if operand.q == 1 { 128 } else { 64 };
    let elements = datasize / esize;

    let t = Type::uscalar_from_size(esize / 8);

    for i in 0..elements {
        let ir = Ir::Value(Operand::Gpr(t, compiler.gpr(operand.rn)));
        let ds = BlockDestination::FprSlot(t, compiler.fpr(operand.rd), i as u8);

        block.append(ir, ds);
    }

    block
}

fn gen_stur_simd_fp(compiler: &AArch64Compiler, operand: LdStRegUnscaledImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);
    let offset = sign_extend(operand.imm9 as i64, 9);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Value(Operand::Fpr(ty, compiler.fpr(operand.rt)));
    let ds = BlockDestination::MemoryRelI64(ty, src, offset);

    block.append(ir, ds);

    block
}

fn gen_stp_simd_fp(compiler: &AArch64Compiler, operand: LoadStoreRegPair, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index) = decode_o_for_ld_st_pair_offset(operand.o);
    let scale = 2 + operand.opc;
    let offset = sign_extend(operand.imm7 as i64, 7) << scale;

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let ir = Ir::Value(Operand::Fpr(ty, compiler.fpr(operand.rt)));
    let ds = BlockDestination::MemoryRelI64(ty, dst, offset_temp);

    block.append(ir, ds);

    let ir = Ir::Value(Operand::Fpr(ty, compiler.fpr(operand.rt2)));
    let ds = BlockDestination::MemoryRelI64(ty, dst, offset_temp + ty.size() as i64);

    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, dst),
            Operand::Immediate(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, dst);

        block.append(ir, ds);
    }

    block
}

fn gen_movk(compiler: &AArch64Compiler, operand: HwImm16Rd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rd = compiler.gpr(operand.rd);

    let pos = operand.hw << 4;
    let mask = !(ones(pos as u64) << pos);
    let masked = Ir::And(ty, Operand::Gpr(ty, rd), Operand::Immediate(ty, mask));
    let ir = Ir::Or(
        ty,
        Operand::ir(masked),
        Operand::Immediate(ty, (operand.imm16 as u64) << pos),
    );
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, rd);

    block.append(ir, ds);
    block
}

//==================================================================================

fn gen_div(compiler: &AArch64Compiler, operand: DataProc2Src, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let op1 = Operand::gpr(ty, compiler.gpr(operand.rn));
    let op2 = Operand::gpr(ty, compiler.gpr(operand.rm));
    let zero = Operand::imm(ty, 0);

    let ir = Ir::If(
        ty,
        Operand::ir(Ir::CmpEq(op2.clone(), zero.clone())),
        Operand::ir(Ir::Value(zero)),
        Operand::ir(Ir::Div(ty, op1, op2)),
    );
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_msub(compiler: &AArch64Compiler, operand: DataProc3Src, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let op1 = Operand::gpr(ty, compiler.gpr(operand.rn));
    let op2 = Operand::gpr(ty, compiler.gpr(operand.rm));
    let op3 = Operand::gpr(ty, compiler.gpr(operand.ra));

    let ir = Ir::Sub(ty, op3, Operand::ir(Ir::Mul(ty, op1, op2)));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_adds_imm(compiler: &AArch64Compiler, operand: ShImm12RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let imm = if operand.sh == 0 {
        operand.imm12 as u64
    } else {
        (operand.imm12 as u64) << 12
    };

    let rn = if operand.rn == 31 {
        Operand::gpr(ty, compiler.stack_reg())
    } else {
        Operand::gpr(ty, compiler.gpr(operand.rn))
    };

    let ir = Ir::Addc(ty, rn, Operand::Immediate(ty, imm));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = if operand.rd == 31 {
        BlockDestination::None
    } else {
        BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd))
    };

    block.append(ir, ds);

    block
}

fn gen_ldrb_reg_shifted_reg(compiler: &AArch64Compiler, operand: LoadStoreRegRegOffset) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = if operand.rn == 31 {
        Operand::Gpr(Type::U64, compiler.stack_reg())
    } else {
        Operand::Gpr(Type::U64, compiler.gpr(operand.rn))
    };

    let ext_type = decode_reg_extend(operand.option);
    let offs = extend_reg(compiler.gpr(operand.rm), ext_type, operand.s, 8);

    let ir = Ir::ZextCast(
        Type::U32,
        Operand::ir(Ir::Load(
            Type::U8,
            Operand::ir(Ir::Add(Type::U64, rn, Operand::ir(offs))),
        )),
    );
    let ds = BlockDestination::Gpr(Type::U32, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_movi(compiler: &AArch64Compiler, operand: AdvSimdModifiedImm) -> IrBlock {
    use utility::parse_pattern;

    let mut block = IrBlock::new(4);

    let cmode_op = operand.cmode << 1 | operand.op;
    let ty = if operand.q == 0b1 {
        Type::Vec(VecType::U64, 2)
    } else {
        Type::U64
    };
    let datasize = ty.size();

    let operation = match cmode_op {
        _ if parse_pattern("0xx00").test_u8(operand.cmode) => ImmediateOp::MOVI,
        _ if parse_pattern("0xx01").test_u8(operand.cmode) => ImmediateOp::MVNI,
        _ if parse_pattern("0xx10").test_u8(operand.cmode) => ImmediateOp::ORR,
        _ if parse_pattern("0xx11").test_u8(operand.cmode) => ImmediateOp::BIC,
        _ if parse_pattern("10x00").test_u8(operand.cmode) => ImmediateOp::MOVI,
        _ if parse_pattern("10x01").test_u8(operand.cmode) => ImmediateOp::MVNI,
        _ if parse_pattern("10x10").test_u8(operand.cmode) => ImmediateOp::ORR,
        _ if parse_pattern("10x11").test_u8(operand.cmode) => ImmediateOp::BIC,
        _ if parse_pattern("110x0").test_u8(operand.cmode) => ImmediateOp::MOVI,
        _ if parse_pattern("110x1").test_u8(operand.cmode) => ImmediateOp::MVNI,
        _ if parse_pattern("1110x").test_u8(operand.cmode) => ImmediateOp::MOVI,
        0b11110 => ImmediateOp::MOVI,
        0b11111 => ImmediateOp::MOVI,
        _ => unreachable!(),
    };

    let abcdefgh = operand.a << 7
        | operand.b << 6
        | operand.c << 5
        | operand.d << 4
        | operand.e << 3
        | operand.f << 2
        | operand.g << 1
        | operand.h;
    let imm64 = adv_simd_exapnd_imm(0b1, operand.cmode, abcdefgh);

    let rep_cnt = (datasize / 64) as usize;
    let imm = if rep_cnt == 1 {
        Value::from_u64(imm64)
    } else {
        let mut imm = Value::new(rep_cnt * 8);
        for q in imm.u64_slice_mut() {
            *q = imm64;
        }

        imm
    };

    let rd = compiler.fpr(operand.rd);

    let imm = Operand::imm_value(ty, imm);
    let operand = Operand::Fpr(ty, rd);

    let ir = match operation {
        ImmediateOp::MOVI => Ir::Value(imm),
        ImmediateOp::MVNI => Ir::Not(ty, imm),
        ImmediateOp::ORR => Ir::Or(ty, operand, imm),
        ImmediateOp::BIC => Ir::And(ty, operand, Operand::ir(Ir::Not(ty, imm))),
    };

    let ds = BlockDestination::Fpr(ty, rd);

    block.append(ir, ds);

    block
}

fn gen_str_imm_simd_fp(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, true);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let ir = Ir::Value(Operand::Fpr(ty, compiler.fpr(operand.rt)));
    let ds = BlockDestination::MemoryRelI64(ty, src, offset_temp);

    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}

fn gen_adds_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = compiler.gpr(operand.rn);
    let rm = compiler.gpr(operand.rm);
    let rd = compiler.gpr(operand.rd);

    let amount = Operand::Immediate(ty, operand.imm6 as u64);

    let sh = shift_reg(
        Operand::Gpr(ty, rm),
        decode_shift(operand.shift),
        amount,
        ty,
    );
    let ir = Ir::Addc(ty, Operand::gpr(ty, rn), Operand::Ir(Box::new(sh)));

    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, rd);
    block.append(ir, ds);

    block
}

fn gen_ldrsh_reg(compiler: &AArch64Compiler, operand: LoadStoreRegRegOffset, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let ext_type = decode_reg_extend(operand.option);
    let shift = if operand.s == 1 { 1 } else { 0 };

    let offset = extend_reg(compiler.gpr(operand.rm), ext_type, shift, 8);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let addr = Ir::Add(Type::U64, Operand::gpr(Type::U64, src), Operand::ir(offset));

    let ir = Ir::SextCast(ty, Operand::ir(Ir::Load(Type::U16, Operand::ir(addr))));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_orn_shifted_reg(compiler: &AArch64Compiler, operand: ShiftRmImm6RnRd, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let shift_type = decode_shift(operand.shift);

    let op1 = if operand.rn == 31 {
        Operand::Immediate(ty, 0)
    } else {
        Operand::Gpr(ty, compiler.gpr(operand.rn))
    };

    let amount = Operand::Immediate(ty, operand.imm6 as u64);

    let op2 = shift_reg(
        Operand::Gpr(ty, compiler.gpr(operand.rm)),
        shift_type,
        amount,
        ty,
    );
    let op2 = Operand::ir(Ir::Not(ty, Operand::ir(op2)));

    let ir = Ir::Or(ty, op1, op2);
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_lslv(compiler: &AArch64Compiler, operand: DataProc2Src, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let shift_type = decode_shift(0);

    let amount = Operand::ir(Ir::Mod(
        ty,
        Operand::gpr(ty, compiler.gpr(operand.rm)),
        Operand::imm(ty, ty.size() as u64 * 8),
    ));

    let ir = shift_reg(
        Operand::gpr(ty, compiler.gpr(operand.rn)),
        shift_type,
        amount,
        ty,
    );
    let ds = BlockDestination::Gpr(ty, compiler.gpr(operand.rd));

    block.append(ir, ds);

    block
}

fn gen_ccmn_imm(compiler: &AArch64Compiler, operand: CondCmpImm, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let rn = compiler.gpr(operand.rn);

    let addc = Operand::void_ir(Ir::Addc(
        ty,
        Operand::gpr(ty, rn),
        Operand::imm(ty, operand.imm5 as u64),
    ));

    let ir = Ir::If(
        Type::U64,
        condition_holds(operand.cond),
        Operand::ir(Ir::Or(
            Type::U64,
            Operand::ir(Ir::BitCast(Type::U64, addc)),
            Operand::Flag,
        )),
        Operand::ir(replace_bits(
            Operand::Flag,
            operand.nzcv as u64,
            Pstate::NZCV.range(),
        )),
    );
    let ds = BlockDestination::Flags;

    block.append(ir, ds);

    block
}

fn gen_ldr_imm_simd_fp(compiler: &AArch64Compiler, operand: OpcSizeImm12RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index, _scale, offset) = decode_operand_for_ld_st_reg_imm(operand, true);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let offset_temp = if !post_index { offset } else { 0 };

    let addr = Operand::ir(Ir::Add(
        Type::U64,
        Operand::Gpr(Type::U64, src),
        Operand::imm(Type::U64, offset_temp as u64),
    ));

    let ir = Ir::Load(ty, addr);
    let ds = BlockDestination::Fpr(ty, compiler.fpr(operand.rt));
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}

fn gen_ldaxr(compiler: &AArch64Compiler, operand: RsRt2RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);
    block.set_atomic(); // this is atomic operation.

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Load(ty, Operand::gpr(Type::U64, src));
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}

fn gen_stlxr(compiler: &AArch64Compiler, operand: RsRt2RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);
    block.set_atomic(); // this is atomic operation.

    if operand.rs == operand.rt || ((operand.rs == operand.rn) && (operand.rn != 31)) {
        block.append(Ir::Nop, BlockDestination::None);

        return block;
    } // Constrain Unpredictable

    let dst = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let ir = Ir::Value(Operand::gpr(ty, compiler.gpr(operand.rt)));
    let ds = BlockDestination::MemoryIr(Ir::Value(Operand::Gpr(ty, dst)));

    block.append(ir, ds);

    let ir = Ir::Value(Operand::Immediate(Type::U64, 0));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rs));

    block.append(ir, ds);

    block
}

fn gen_ldar(compiler: &AArch64Compiler, operand: RsRt2RnRt, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);
    block.set_atomic(); // this is atomic operation.

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };

    let addr = Operand::ir(Ir::Value(Operand::gpr(Type::U64, src)));

    let ir = Ir::Load(ty, addr);
    let ir = Ir::ZextCast(Type::U64, Operand::ir(ir));
    let ds = BlockDestination::Gpr(Type::U64, compiler.gpr(operand.rt));

    block.append(ir, ds);

    block
}


fn gen_ldp_simd_fp(compiler: &AArch64Compiler, operand: LoadStoreRegPair, ty: Type) -> IrBlock {
    let mut block = IrBlock::new(4);

    let (wback, post_index) = decode_o_for_ld_st_pair_offset(operand.o);

    let src = if operand.rn == 31 {
        compiler.stack_reg()
    } else {
        compiler.gpr(operand.rn)
    };
    let scale = operand.opc + 2;
    let offset = sign_extend(operand.imm7 as i64, 7) << scale;

    let addr = if !post_index {
        Operand::ir(Ir::Add(Type::U64, Operand::Gpr(Type::U64, src), Operand::imm(Type::I64, offset as u64)))
    } else {
        Operand::Gpr(Type::U64, src)
    };

    let ir = Ir::Load(ty, addr.clone());
    let ds = BlockDestination::Fpr(ty, compiler.fpr(operand.rt));
    block.append(ir, ds);

    let addr =  Operand::ir(Ir::Add(Type::U64, addr, Operand::imm(Type::I64, ty.size() as u64)));
    
    let ir = Ir::Load(ty, addr);
    let ds = BlockDestination::Fpr(ty, compiler.fpr(operand.rt2));
    block.append(ir, ds);

    if wback {
        let ir = Ir::Add(
            Type::U64,
            Operand::Gpr(Type::U64, src),
            Operand::imm(Type::I64, offset as u64),
        );
        let ds = BlockDestination::Gpr(Type::U64, src);

        block.append(ir, ds);
    }

    block
}
fn gen_msr_imm(compiler: &AArch64Compiler, operand: PstateOp) -> IrBlock {
    let mut block = IrBlock::new(4);
    block
}
