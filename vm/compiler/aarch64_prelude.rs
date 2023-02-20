use std::ops::Range;

use crate::ir::*;
use crate::register::RegId;
use utility::*;

pub enum Pstate {
    N,
    Z,
    C,
    V,
    D,
    A,
    I,
    F,
    SS,
    IL,
    EL,
    NRW,
    SP,
    ALLINT,
    PAN,
    UAO,
    TCO,
    BTYPE,
    DIT,
    SSBS,
    ZA,
    SM,

    NZCV,
}

impl Pstate {
    pub const fn range(&self) -> Range<u64> {
        match self {
            Pstate::N => 63..64,
            Pstate::Z => 62..63,
            Pstate::C => 61..62,
            Pstate::V => 60..61,
            Pstate::D => 59..60,
            Pstate::A => 58..59,
            Pstate::I => 57..58,
            Pstate::F => 56..57,
            Pstate::SS => 55..56,
            Pstate::IL => 54..55,
            Pstate::EL => 52..54,
            Pstate::NRW => 51..52,
            Pstate::SP => 50..51,
            Pstate::ALLINT => 49..50,
            Pstate::PAN => 48..49,
            Pstate::UAO => 47..48,
            Pstate::DIT => 46..47,
            Pstate::TCO => 45..46,
            Pstate::ZA => 44..45,
            Pstate::SM => 43..44,
            Pstate::SSBS => 42..43,
            Pstate::BTYPE => 40..42,

            Pstate::NZCV => Pstate::V.range().start..Pstate::N.range().end
        }
    }

    pub const fn idx(&self) -> u64 {
        self.range().start
    }

    pub const fn mask(&self) -> u64 {
        let rng = self.range();
        ones(rng.end - rng.start) << rng.start
    }
}

pub const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

pub enum ShiftType {
    LSL, // Logical shift left
    LSR, // Logical shift right
    ASR, // Arithmetic shift right
    ROR, // Rotate right
}

pub fn decode_shift(shift: u8) -> ShiftType {
    match shift {
        0b00 => ShiftType::LSL,
        0b01 => ShiftType::LSR,
        0b10 => ShiftType::ASR,
        0b11 => ShiftType::ROR,
        _ => unreachable!("failed to decode shift: 0b{shift:0b}"),
    }
}

pub const fn decode_operand_for_ld_st_reg_imm(
    operand: machineinstr::aarch64::OpcSizeImm12RnRt,
    is_vec: bool,
) -> (bool, bool, u8, i64) {
    let opc1 = (operand.opc & 0b10) >> 1;
    if operand.idxt == 0b00 {
        let imm9 = extract_bits16(2..11, operand.imm12) as i64;
        let post = extract_bits16(0..2, operand.imm12) == 0b01;

        let scale = (if is_vec { 4 } else { 0 }) * opc1 + operand.size;

        (true, post, scale, sign_extend(imm9, 9))
    } else {
        //Unsigned offset
        let scale = (if is_vec { 4 } else { 0 }) * opc1 + operand.size;
        (false, false, scale, (operand.imm12 << scale) as i64)
    }
}

pub const fn decode_o_for_ld_st_pair_offset(o: u8) -> (bool, bool) {
    match o {
        0b001 => (true, true),
        0b011 => (true, false),
        0b010 => (false, false),
        _ => unreachable!(),
    }
}

pub const fn gen_ip_relative(offset: i64) -> Ir {
    if offset > 0 {
        Ir::Add(
            Type::U64,
            Operand::Ip,
            Operand::imm(Type::U64, offset as u64),
        )
    } else {
        Ir::Sub(
            Type::U64,
            Operand::Ip,
            Operand::imm(Type::U64, (-offset) as u64),
        )
    }
}

pub fn condition_holds(cond: u8) -> Operand {
    let masked_cond = (cond & 0b1110) >> 1;
    let cond0 = cond & 1;

    let result = match masked_cond {
        0b000 => cmp_eq_op_imm64(Operand::ir(flag(Pstate::Z.range())), 1),
        0b001 => cmp_eq_op_imm64(Operand::ir(flag(Pstate::C.range())), 1),
        0b010 => cmp_eq_op_imm64(Operand::ir(flag(Pstate::N.range())), 1),
        0b011 => cmp_eq_op_imm64(Operand::ir(flag(Pstate::V.range())), 1),
        0b100 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            cmp_eq_op_imm64(Operand::ir(flag(Pstate::C.range())), 1),
            cmp_eq_op_imm64(Operand::ir(flag(Pstate::Z.range())), 0),
        ))),
        0b101 => Operand::Ir(Box::new(Ir::CmpEq(
            Operand::ir(flag(Pstate::N.range())),
            Operand::ir(flag(Pstate::V.range())),
        ))),
        0b110 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            Operand::Ir(Box::new(Ir::CmpEq(
                Operand::ir(flag(Pstate::N.range())),
                Operand::ir(flag(Pstate::V.range())),
            ))),
            cmp_eq_op_imm64(Operand::ir(flag(Pstate::Z.range())), 0),
        ))),
        0b111 => Operand::imm(Type::Bool, 0b1u64),
        _ => unreachable!(),
    };

    if cond0 == 1 && cond != 0b1111 {
        Operand::Ir(Box::new(Ir::Not(Type::Bool, result)))
    } else {
        result
    }
}

pub fn cmp_eq_op_imm64(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

pub fn cmp_eq_op_imm32(op: Operand, immediate: u32) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(
        op,
        Operand::imm(Type::U32, immediate as u64),
    )))
}

pub fn cmp_ne_op_imm(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

pub fn flag(range: Range<u64>) -> Ir {
    Ir::And(
        Type::U64,
        Operand::ir(Ir::LShr(
            Type::U64,
            Operand::Flag,
            Operand::Immediate(Type::U64, range.start),
        )),
        Operand::imm(Type::U64, ones(range.end - range.start)),
    )
}

pub fn shift_reg(reg: Operand, shift_type: ShiftType, amount: u64, t: Type) -> Ir {
    let amount = Operand::imm(t, amount);

    match shift_type {
        ShiftType::LSL => Ir::LShl(t, reg, amount),
        ShiftType::LSR => Ir::LShr(t, reg, amount),
        ShiftType::ASR => Ir::AShr(t, reg, amount),
        ShiftType::ROR => Ir::Rotr(t, reg, amount),
    }
}

pub const fn highest_set_bit(x: u64) -> u64 {
    63 - x.leading_zeros() as u64
}

pub const fn ones(n: u64) -> u64 {
    replicate(1, n, 1)
}

pub const fn ror(x: u64, shift: u64, size: u64) -> u64 {
    let m = shift % size;
    x >> m | ((x.overflowing_shl((size - m) as u32).0) & ones(shift))
}

pub const fn replicate(x: u64, n: u64, size: u64) -> u64 {
    let mut result = 0b0;
    let mut i = n;

    while i > 0 {
        result |= x;
        result = result.overflowing_shl(size as u32).0;
        i -= 1;
    }

    result
}

pub fn replicate_reg64(val: RegId, n: u8) -> Ir {
    let val = Operand::gpr(Type::U64, val);

    Ir::If(
        Type::U64,
        Operand::ir(Ir::BitCast(
            Type::Bool,
            Operand::ir(Ir::LShr(Type::U64, val, Operand::imm(Type::U64, n as u64))),
        )),
        Operand::imm(Type::U64, u64::MAX),
        Operand::imm(Type::U64, 0),
    )
}

pub const fn decode_bit_masks(immn: u8, imms: u8, immr: u8, immediate: bool, m: u8) -> (u64, u64) {
    let len = highest_set_bit(((immn << 6) as u16 | extract_bits16(0..6, !imms as u16)) as u64);
    assert!(len >= 1, "UNDEFINED");
    assert!(m >= (1 << len), "UNDEFINED");

    let levels = ones(len);

    assert!(
        !(immediate && (imms as u64 & levels) == levels),
        "UNDEFINED"
    );

    let s = imms & levels as u8;
    let r = immr & levels as u8;
    let (diff, _) = s.overflowing_sub(r);

    let esize = 1 << len;
    let d = extract_bits16(0..len as usize, diff as u16);

    let welem = ones(s as u64 + 1);
    let telem = ones(d as u64 + 1);

    let wmask = replicate(ror(welem, r as u64, esize), m as u64, esize);
    let tmask = replicate(telem, m as u64, esize);

    (wmask, tmask)
}

pub enum ExtendType {
    UXTB,
    UXTH,
    UXTW,
    UXTX,
    SXTB,
    SXTH,
    SXTW,
    SXTX,
}

pub const fn decode_reg_extend(op: u8) -> ExtendType {
    match op {
        0b000 => ExtendType::UXTB,
        0b001 => ExtendType::UXTH,
        0b010 => ExtendType::UXTW,
        0b011 => ExtendType::UXTX,
        0b100 => ExtendType::SXTB,
        0b101 => ExtendType::SXTH,
        0b110 => ExtendType::SXTW,
        0b111 => ExtendType::SXTX,
        _ => unreachable!(),
    }
}

pub fn extend_reg(reg: RegId, ext_type: ExtendType, shift: u8, n: u8) -> Ir {
    assert!(shift <= 4);
    let n = Type::uscalar_from_size(n as usize);

    let (unsigned, ty) = match ext_type {
        ExtendType::SXTB => (false, Type::I8),
        ExtendType::SXTH => (false, Type::I16),
        ExtendType::SXTW => (false, Type::I32),
        ExtendType::SXTX => (false, Type::I64),
        ExtendType::UXTB => (true, Type::U8),
        ExtendType::UXTH => (true, Type::U16),
        ExtendType::UXTW => (true, Type::U32),
        ExtendType::UXTX => (true, Type::U64),
    };

    let ir = Ir::LShl(
        ty,
        Operand::Gpr(ty, reg),
        Operand::Immediate(Type::U8, shift as u64),
    );

    if unsigned {
        Ir::ZextCast(n, Operand::ir(ir))
    } else {
        Ir::SextCast(n, Operand::ir(ir))
    }
}

pub fn replace_bits(val: Operand, imm: u64, range: Range<u64>) -> Ir {
    let imm = Operand::imm(val.get_type(), imm << range.start);
    let mask = Operand::imm(
        val.get_type(),
        !(ones(range.end - range.start) << range.start),
    );

    Ir::Or(
        val.get_type(),
        Operand::ir(Ir::And(val.get_type(), val, mask)),
        imm,
    )
}

pub fn gen_mask64<T>(range: Range<T>) -> u64
where
    T: Into<usize> + Copy,
{
    let mask = (u64::MAX >> range.start.into()) << range.start.into();
    (mask << range.end.into()) >> range.end.into()
}

pub fn adv_simd_exapnd_imm(op: u8, cmode: u8, imm8: u8) -> u64 {
    let imm8 = imm8 as u64;
    let cmode0 = cmode & 0b1;

    match cmode >> 1 {
        0b000 => replicate(imm8, 2, 32),
        0b001 => replicate(imm8 << 8, 2, 32),
        0b010 => replicate(imm8 << 16, 2, 32),
        0b011 => replicate(imm8 << 24, 2, 32),
        0b100 => replicate(imm8, 4, 16),
        0b101 => replicate(imm8 << 8, 4, 16),
        0b110 if cmode0 == 0 => replicate(imm8 << 8 | ones(8), 2, 32),
        0b110 if cmode0 == 1 => replicate(imm8 << 16 | ones(16), 2, 32),
        0b111 => {
            if cmode0 == 0 && op == 0 {
                return replicate(imm8, 8, 8);
            }
            if cmode0 == 0 && op == 1 {
                let imm8a = replicate(bit(imm8, 7).into(), 8, 1) << 56;
                let imm8b = replicate(bit(imm8, 6).into(), 8, 1) << 48;
                let imm8c = replicate(bit(imm8, 5).into(), 8, 1) << 40;
                let imm8d = replicate(bit(imm8, 4).into(), 8, 1) << 32;
                let imm8e = replicate(bit(imm8, 3).into(), 8, 1) << 24;
                let imm8f = replicate(bit(imm8, 2).into(), 8, 1) << 16;
                let imm8g = replicate(bit(imm8, 1).into(), 8, 1) << 8;
                let imm8h = replicate(bit(imm8, 0).into(), 8, 1);

                return imm8a | imm8b | imm8c | imm8d | imm8e | imm8f | imm8g | imm8h;
            }
            if cmode0 == 1 && op == 0 {
                let a = u64::from(bit(imm8, 7)) << 31;
                let b = u64::from(!bit(imm8, 6)) << 30;
                let c = replicate(bit(imm8, 6).into(), 5, 1) << 25;
                let d = imm8 & 0b111111 << 19;

                let imm32 = a | b | c | d;
                return replicate(imm32, 2, 32);
            }
            if cmode0 == 1 && op == 1 {
                let a = u64::from(bit(imm8, 7)) << 63;
                let b = u64::from(!bit(imm8, 6)) << 62;
                let c = replicate(bit(imm8, 6).into(), 8, 1) << 54;
                let d = imm8 & 0b111111 << 48;

                return a | b | c | d;
            }

            unreachable!()
        }

        _ => unreachable!(),
    }
}

pub const fn bit(val: u64, idx: u8) -> bool {
    ((val >> idx) & 0b1) == 0b1
}

pub enum ImmediateOp {
    MOVI,
    MVNI,
    ORR,
    BIC,
}

pub enum PSTATEField {
    DAIFSet,
    DAIFClr,
    PAN,
    UAO,
    DIT,
    SSBS,
    TCO,
    SVCRSM,
    SVCRZA,
    SVCRSMZA,
    ALLINT,
    SP,
}

pub fn check_transactional_system_acceess(op0: u8, op1: u8, crn: u8, crm: u8, op2: u8, read: u8) -> bool {
    match (read, op0, op1, crn, crm, op2) {
        (0b0, 0b00, 0b011, 0b0100, _, _) if parse_pattern("xxxx").test_u8(crm) && parse_pattern("11x").test_u8(op2) => true,
        (0b0, 0b01, 0b011, 0b0111, 0b0100, 0b001) => true,
        (0b0, 0b11, 0b011, 0b0100, 0b0010, _) if parse_pattern("00x").test_u8(op2) => true,
        (0b0, 0b11, 0b011, 0b0100, 0b0100, _) if parse_pattern("00x").test_u8(op2) => true,
        (0b0, 0b11, 0b000, 0b0100, 0b0110, 0b000) => true,
        (0b0, 0b11, 0b011, 0b1001, 0b1100, 0b100) => true,
        (0b1, 0b11, _, _, _, _) if parse_pattern("0xxx").test_u8(crn) => true,
        (0b1, 0b11, _, _, _, _) if parse_pattern("100x").test_u8(crn) => true,
        (0b1, 0b11, _, 0b1010, _, _) => true,
        (0b1, 0b11, 0b000, 0b1100, _, 0b010) if parse_pattern("1x00").test_u8(crm) => true,
        (0b1, 0b11, 0b000, 0b1100, 0b1011, 0b011) => true,
        (0b1, 0b11, _, 0b1101, _, _) => true,
        (0b1, 0b11, _, 0b1110, _, _) => true,
        (0b0, 0b01, 0b011, 0b0111, 0b0011, 0b111) => true,
        (0b0, 0b01, 0b011, 0b0111, 0b0011, _) if parse_pattern("10x").test_u8(op2) => true,
        (_, 0b11, _, _, _, _) if parse_pattern("1x11").test_u8(crn) => panic!("Need to return boolean IMPLEMENTATION_DEFINED"),

        _=> false
    }
}