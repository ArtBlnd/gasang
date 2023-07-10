use core::Architecture;
use std::str::Chars;

use crate::aarch64::inst::AArch64Inst;
use crate::aarch64::inst_operand::*;
use crate::aarch64::AArch64Architecture;
use crate::aarch64::AArch64MnemonicHint;
use utility::BitPatternMatcher;

use once_cell::sync::Lazy;
use utility::Extract;

fn to_le(pat: impl AsRef<str>) -> String {
    let pat: Vec<char> = pat
        .as_ref()
        .chars()
        .filter(|&c| c != ' ' && c != '_')
        .collect();

    pat.chunks(8).rev().flatten().collect()
}

pub(crate) fn decode_aarch64_inst(raw: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_xx_0000_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            |raw_instr: &[u8],
             Extract(op0): Extract<u8, 29, 31>,
             Extract(op1): Extract<u8, 16, 25>,
             Extract(imm16): Extract<u16, 0, 16>| {
                let imm16 = Imm16 { imm16 };
                match (op0, op1) {
                    (0b00, 0b000000000) => Some(AArch64Inst::Udf(imm16)),
                    _ => todo!("Unknown reserved instruction {:?}", raw_instr),
                }
            },
        )
        .bind(
            to_le("1_xx_0000_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            |_raw_instr: &[u8]| todo!("SME encodings"),
        )
        .bind(
            to_le("x_xx_0010_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            |_raw_instr: &[u8]| todo!("SVE encodings"),
        )
        .bind(
            to_le("x_xx_100x_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            parse_aarch64_d_p_i,
        )
        .bind(
            to_le("x_xx_101x_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            parse_aarch64_branches_exception_gen_and_sys_instr,
        )
        .bind(
            to_le("x_xx_x1x0_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            parse_aarch64_load_and_stores,
        )
        .bind(
            to_le("x_xx_x101_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            parse_aarch64_d_p_r,
        )
        .bind(
            to_le("x_xx_x111_xxxxxxxxxxxxxxxxxxxxxxxxx"),
            parse_aarch64_dp_sfp_adv_simd,
        );

        m
    });

    MATCHER.try_match(raw).flatten()
}

// parse DPI(Data Processing Immediate) instructions in AArch64
fn parse_aarch64_d_p_i(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xxx_100_00x_xxxxxxxxxxxxxxxxxxxxxxx"),
            parse_pc_rel_addressing,
        )
        .bind(
            to_le("xxx_100_010_xxxxxxxxxxxxxxxxxxxxxxx"),
            parse_add_sub_immediate,
        )
        .bind(
            to_le("xxx_100_011_xxxxxxxxxxxxxxxxxxxxxxx"),
            parse_add_sub_imm_with_tags,
        )
        .bind(
            to_le("xxx_100_100_xxxxxxxxxxxxxxxxxxxxxxx"),
            parse_logical_imm,
        )
        .bind(
            to_le("xxx_100_101_xxxxxxxxxxxxxxxxxxxxxxx"),
            parse_move_wide_imm,
        )
        .bind(to_le("xxx_100_110_xxxxxxxxxxxxxxxxxxxxxxx"), parse_bitfield)
        .bind(to_le("xxx_100_111_xxxxxxxxxxxxxxxxxxxxxxx"), parse_extract);

        m
    });

    MATCHER.try_match(raw_instr).flatten()
}

// parse DPI(Data Processing Register) instructions in AArch64
fn parse_aarch64_d_p_r(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_1_101_0110_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_data_proc_2src,
        )
        .bind(
            to_le("x_1_x_1_101_0110_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_data_proc_1src,
        )
        .bind(
            to_le("x_x_x_0_101_0xxx_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_logical_shifted_register,
        )
        .bind(
            to_le("x_x_x_0_101_1xx0_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_add_sub_shifted_reg,
        )
        .bind(
            to_le("x_x_x_0_101_1xx1_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_add_sub_ext_reg,
        )
        .bind(
            to_le("x_x_x_1_101_0000_xxxxx_000000_xxxxxxxxxx"),
            parse_add_sub_with_carry,
        )
        .bind(
            to_le("x_x_x_1_101_0000_xxxxx_x00001_xxxxxxxxxx"),
            parse_rot_right_into_flags,
        )
        .bind(
            to_le("x_x_x_1_101_0000_xxxxx_xx0010_xxxxxxxxxx"),
            parse_eval_into_flags,
        )
        .bind(
            to_le("x_x_x_1_101_0010_xxxxx_xxxx0x_xxxxxxxxxx"),
            parse_cond_cmp_reg,
        )
        .bind(
            to_le("x_x_x_1_101_0010_xxxxx_xxxx1x_xxxxxxxxxx"),
            parse_cond_cmp_imm,
        )
        .bind(
            to_le("x_x_x_1_101_0100_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_cond_sel,
        )
        .bind(
            to_le("x_x_x_1_101_1xxx_xxxxx_xxxxxx_xxxxxxxxxx"),
            parse_data_proccessing_3src,
        );

        m
    });

    MATCHER.try_match(raw_instr).flatten()
}

fn parse_aarch64_dp_sfp_adv_simd(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0100"),
                to_le("0x"),
                to_le("x101"),
                "00xxxxx10"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic AES"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0101"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx0xxx00"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic three-register SHA"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0101"),
                to_le("0x"),
                to_le("x101"),
                "00xxxxx10"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic two-register SHA"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("00"),
                to_le("00xx"),
                "xxx0xxxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar copy"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("10xx"),
                "xxx00xxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar three same FP16"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("1111"),
                "00xxxxx10"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar two-register miscellaneous FP16"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx1xxxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar three same extra"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("x100"),
                "00xxxxx10"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar two-register miscellaneous"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("x110"),
                "00xxxxx10"
            ),
            parse_adv_simd_scalar_pairwise,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxx00"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar three different"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar three same"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("10"),
                to_le("xxxx"),
                "xxxxxxxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD scalar shifted by immediate"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("01x1"),
                to_le("1x"),
                to_le("xxxx"),
                "xxxxxxxx0"
            ),
            parse_adv_simd_scalar_x_indexed_elem,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0x00"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx0xxx00"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD table lookup"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0x00"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx0xxx10"
            ),
            parse_advanced_simd_permute,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0x10"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx0xxxx0"
            ),
            parse_advanced_simd_extract,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("00"),
                to_le("00xx"),
                "xxx0xxxx1"
            ),
            parse_advanced_simd_copy,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("10xx"),
                "xxx00xxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD three same (FP16)"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("1111"),
                "00xxxxx10"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD two-register miscellaneous (FP16)"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("x0xx"),
                "xxx1xxxx1"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD three-register extension"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("x100"),
                "00xxxxx10"
            ),
            parse_adv_simd_2reg_miscellaneous,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("x110"),
                "00xxxxx10"
            ),
            parse_adv_simd_across_lanes,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxx00"
            ),
            |_raw_instr: &[u8]| todo!("Advanced SIMD three different"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxxx1"
            ),
            parse_advanced_simd_three_same,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("10"),
                to_le("xxxx"),
                "xxxxxxxx1"
            ),
            |raw_instr: &[u8], Extract(op2): Extract<u8, 19, 23>| {
                if op2 == 0b0000 {
                    parse_adv_simd_modified_imm(raw_instr)
                } else {
                    parse_adv_simd_shift_by_imm(raw_instr)
                }
            },
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("0xx0"),
                to_le("1x"),
                to_le("xxxx"),
                "xxxxxxxx0"
            ),
            parse_adv_simd_vec_x_indexed_elem,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("1100"),
                to_le("00"),
                to_le("10xx"),
                "xxx10xxxx"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic three-register, imm2"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("1100"),
                to_le("00"),
                to_le("11xx"),
                "xxx1x00xx"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic three-reigster SHA 512"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("1100"),
                to_le("00"),
                to_le("xxxx"),
                "xxx0xxxxx"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic four-register"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("1100"),
                to_le("01"),
                to_le("00xx"),
                "xxxxxxxxx"
            ),
            |_raw_instr: &[u8]| todo!("XAR"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("1100"),
                to_le("01"),
                to_le("1000"),
                "0001000xx"
            ),
            |_raw_instr: &[u8]| todo!("Cryptographic two-register SHA 512"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x0xx"),
                "xxxxxxxxx"
            ),
            parse_conv_between_float_and_fixed_point,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxx000000"
            ),
            parse_conv_between_float_and_int,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxx10000"
            ),
            parse_float_data_proc_1src,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxx1000"
            ),
            parse_floating_point_compare,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxx100"
            ),
            parse_floating_point_immediate,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxx01"
            ),
            |_raw_instr: &[u8]| todo!("Floating-point conditional compare"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxx10"
            ),
            parse_float_data_proc_2src,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("0x"),
                to_le("x1xx"),
                "xxxxxxx11"
            ),
            parse_floating_point_conditional_select,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                to_le("x0x1"),
                to_le("1x"),
                to_le("xxxx"),
                "xxxxxxxxx"
            ),
            parse_fp_data_processing_3src,
        );

        m
    });

    MATCHER.try_match(raw_instr).flatten()
}

// parse Load and stores instructions i pairn AArch64
fn parse_aarch64_load_and_stores(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0x00_1_0_0_00_x_1xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_compare_and_swap_pair,
        )
        .bind(
            to_le("0x00_1_1_0_00_x_000000_xxxx_xx_xxxxxxxxxx"),
            parse_adv_simd_ld_st_multi_structures,
        )
        .bind(
            to_le("0x00_1_1_0_01_x_0xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_adv_simd_ld_st_multi_structures_post_indexed,
        )
        .bind(
            to_le("0x00_1_1_0_10_x_x00000_xxxx_xx_xxxxxxxxxx"),
            parse_adv_simd_ld_st_single_structure,
        )
        .bind(
            to_le("0x00_1_1_0_11_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            |_raw_instr: &[u8]| {
                todo!("Advanced SIMD Load/Store single structure(post-indexed)");
            },
        )
        .bind(
            to_le("1101_1_0_0_1x_x_1xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_memory_tags,
        )
        .bind(
            to_le("1x00_1_0_0_00_x_1xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_exclusive_pair,
        )
        .bind(
            to_le("xx00_1_0_0_00_x_0xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_exclusive_register,
        )
        .bind(
            to_le("xx00_1_0_0_01_x_0xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_ordered,
        )
        .bind(
            to_le("xx00_1_0_0_01_x_1xxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_compare_and_swap,
        )
        .bind(
            to_le("xx01_1_0_0_1x_x_0xxxxx_xxxx_00_xxxxxxxxxx"),
            parse_ldapr_stlr_unscaled_imm,
        )
        .bind(
            to_le("xx01_1_x_0_0x_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_register_literal,
        )
        .bind(
            to_le("xx01_1_x_0_1x_x_0xxxxx_xxxx_01_xxxxxxxxxx"),
            |_raw_instr: &[u8]| {
                todo!("Memory Copy and Memory Set");
            },
        )
        .bind(
            to_le("xx10_1_x_0_00_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_ld_st_no_alloc_pair_offset,
        )
        .bind(
            to_le("xx10_1_x_0_01_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_reg_pair_post_indexed,
        )
        .bind(
            to_le("xx10_1_x_0_10_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_reg_pair_offset,
        )
        .bind(
            to_le("xx10_1_x_0_11_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_reg_pair_pre_indexed,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_0xxxxx_xxxx_00_xxxxxxxxxx"),
            parse_load_store_reg_unscaled_imm,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_0xxxxx_xxxx_01_xxxxxxxxxx"),
            parse_load_store_reg_imm_post_indexed,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_0xxxxx_xxxx_10_xxxxxxxxxx"),
            parse_load_store_reg_unprivileged,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_0xxxxx_xxxx_11_xxxxxxxxxx"),
            parse_load_store_reg_imm_pre_indexed,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_1xxxxx_xxxx_00_xxxxxxxxxx"),
            parse_atomic_memory_operations,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_1xxxxx_xxxx_10_xxxxxxxxxx"),
            parse_load_store_reg_reg_offset,
        )
        .bind(
            to_le("xx11_1_x_0_0x_x_1xxxxx_xxxx_x1_xxxxxxxxxx"),
            |_raw_instr: &[u8]| {
                todo!("Load/Store register (pac)"); // Need to do FEAT_PAuth feature instructions
            },
        )
        .bind(
            to_le("xx11_1_x_0_1x_x_xxxxxx_xxxx_xx_xxxxxxxxxx"),
            parse_load_store_reg_unsigned_imm,
        );

        m
    });

    MATCHER.try_match(raw_instr).flatten()
}

fn parse_aarch64_branches_exception_gen_and_sys_instr(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<Option<AArch64Inst>>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        //--------------------------------------------
        //      |op1|101|      op2     |       | op3 |
        m.bind(
            to_le("010_101_0xxxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_cond_branch_imm,
        )
        .bind(
            to_le("110_101_00xxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_exception_gen,
        )
        .bind(
            to_le("110_101_01000000110001_xxxxxxx_xxxxx"),
            parse_sys_instr_with_reg_arg,
        )
        .bind(to_le("110_101_01000000110010_xxxxxxx_11111"), parse_hints)
        .bind(
            to_le("110_101_01000000110011_xxxxxxx_xxxxx"),
            parse_barriers,
        )
        .bind(to_le("110_101_0100000xxx0100_xxxxxxx_xxxxx"), parse_pstate)
        .bind(
            to_le("110_101_0100100xxxxxxx_xxxxxxx_xxxxx"),
            parse_sys_with_result,
        )
        .bind(
            to_le("110_101_0100x01xxxxxxx_xxxxxxx_xxxxx"),
            parse_sys_instr,
        )
        .bind(
            to_le("110_101_0100x1xxxxxxxx_xxxxxxx_xxxxx"),
            parse_sys_reg_mov,
        )
        .bind(
            to_le("110_101_1xxxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_uncond_branch_reg,
        )
        .bind(
            to_le("x00_101_xxxxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_uncond_branch_imm,
        )
        .bind(
            to_le("x01_101_0xxxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_cmp_and_branch_imm,
        )
        .bind(
            to_le("x01_101_1xxxxxxxxxxxxx_xxxxxxx_xxxxx"),
            parse_test_and_branch_imm,
        );

        m
    });

    MATCHER.try_match(raw_instr).flatten()
}

fn parse_add_sub_shifted_reg(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xxx_01011_xx_0_xxxxxxxxxxxxxxxxxxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(shift): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(imm6): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = ShiftRmImm6RnRd {
                    shift,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                    imm6,
                };

                match (sf_op_s, shift, imm6) {
                    (0b000, _, _) => AArch64Inst::AddShiftedReg32(data),
                    (0b001, _, _) => AArch64Inst::AddsShiftedReg32(data),
                    (0b010, _, _) => AArch64Inst::SubShiftedReg32(data),
                    (0b011, _, _) => AArch64Inst::SubsShiftedReg32(data),
                    (0b100, _, _) => AArch64Inst::AddShiftedReg64(data),
                    (0b101, _, _) => AArch64Inst::AddsShiftedReg64(data),
                    (0b110, _, _) => AArch64Inst::SubShiftedReg64(data),
                    (0b111, _, _) => AArch64Inst::SubsShiftedReg64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_add_sub_immediate(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_100010_x_xxxxxxxxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(sh): Extract<u8, 22, 23>,
             Extract(imm12): Extract<u16, 10, 22>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = ShImm12RnRd {
                    sh,
                    imm12,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match sf_op_s {
                    0b000 => AArch64Inst::AddImm32(data),
                    0b001 => AArch64Inst::AddsImm32(data),
                    0b010 => AArch64Inst::SubImm32(data),
                    0b011 => AArch64Inst::SubsImm32(data),
                    0b100 => AArch64Inst::AddImm64(data),
                    0b101 => AArch64Inst::AddsImm64(data),
                    0b110 => AArch64Inst::SubImm64(data),
                    0b111 => AArch64Inst::SubsImm64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_fp_data_processing_3src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11111_xx_x_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(o1): Extract<u8, 21, 22>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(ra): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RmRaRnRd {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    ra: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, ra),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (m, s, ptype, o1, o0) {
                    (0b0, 0b0, 0b00, 0b0, 0b0) => AArch64Inst::FmAddSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b0, 0b1) => AArch64Inst::FmSubSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b1, 0b0) => AArch64Inst::FnmAddSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b1, 0b1) => AArch64Inst::FnmSubSinglePrecision(data),
                    (0b0, 0b0, 0b01, 0b0, 0b0) => AArch64Inst::FmAddDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b0, 0b1) => AArch64Inst::FmSubDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b1, 0b0) => AArch64Inst::FnmAddDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b1, 0b1) => AArch64Inst::FnmSubDoublePrecision(data),
                    (0b0, 0b0, 0b11, 0b0, 0b0) => AArch64Inst::FmAddHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b0, 0b1) => AArch64Inst::FmSubHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b1, 0b0) => AArch64Inst::FnmAddHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b1, 0b1) => AArch64Inst::FnmSubHalfPrecision(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_unsigned_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_01_xx_xxxxxxxxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(idxt): Extract<u8, 24, 26>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm12): Extract<u16, 10, 22>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = OpcSizeImm12RnRt {
                    idxt,
                    opc,
                    size,
                    imm12,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rt,
                    ),
                };

                match (size, v, opc) {
                    (0b00, 0b0, 0b00) => AArch64Inst::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Inst::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Inst::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Inst::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Inst::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Inst::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Inst::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Inst::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Inst::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Inst::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Inst::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Inst::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP16(data),
                    (0b10, 0b0, 0b00) => AArch64Inst::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Inst::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Inst::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Inst::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Inst::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Inst::LdrImm64(data),
                    (0b11, 0b0, 0b10) => AArch64Inst::PrfmImm(data),
                    (0b11, 0b1, 0b00) => AArch64Inst::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_move_wide_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_100101_xx_xxxxxxxxxxxxxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_opc): Extract<u8, 29, 32>,
             Extract(hw): Extract<u8, 21, 23>,
             Extract(imm16): Extract<u16, 5, 21>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = HwImm16Rd {
                    hw,
                    imm16,
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf_opc, hw) {
                    (0b000, 0b00 | 0b01) => AArch64Inst::MovnVar32(data),
                    (0b010, 0b00 | 0b01) => AArch64Inst::MovzVar32(data),
                    (0b011, 0b00 | 0b01) => AArch64Inst::MovkVar32(data),
                    (0b100, _) => AArch64Inst::MovnVar64(data),
                    (0b110, _) => AArch64Inst::MovzVar64(data),
                    (0b111, _) => AArch64Inst::MovkVar64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_uncond_branch_reg(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1101011_xxxx_xxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 21, 25>,
             Extract(op2): Extract<u8, 16, 21>,
             Extract(op3): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(op4): Extract<u8, 0, 5>,
             Extract(z): Extract<u8, 24, 25>,
             Extract(op): Extract<u8, 21, 23>,
             Extract(a): Extract<u8, 11, 12>| {
                let data = UncondBranchReg {
                    z,
                    op,
                    a,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rm: AArch64Architecture::get_register_by_mnemonic(
                        if z == 0b01 && op4 == 31 {
                            AArch64MnemonicHint::X_SP
                        } else {
                            AArch64MnemonicHint::X
                        },
                        op4,
                    ),
                };

                match (opc, op2, op3, rn, op4) {
                    (0b0000, 0b11111, 0b000000, _, 0b00000) => AArch64Inst::Br(data),
                    (0b0000, 0b11111, 0b000010, _, 0b11111) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ. Key A, zero modifier")
                    }
                    (0b0000, 0b11111, 0b000011, _, 0b11111) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ. Key B, zero modifier")
                    }
                    (0b0001, 0b11111, 0b000000, _, 0b00000) => AArch64Inst::Blr(data),
                    (0b0001, 0b11111, 0b000010, _, 0b11111) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ. Key A, zero modifier")
                    }
                    (0b0001, 0b11111, 0b000011, _, 0b11111) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ. Key B, zero modifier")
                    }
                    (0b0010, 0b11111, 0b000000, _, 0b00000) => AArch64Inst::Ret(data),
                    (0b0010, 0b11111, 0b000010, 0b11111, 0b11111) => {
                        todo!("RETAA, RETAB - RETAA variant")
                    }
                    (0b0010, 0b11111, 0b000011, 0b11111, 0b11111) => {
                        todo!("RETAA, RETAB - RETAB variant")
                    }
                    (0b0100, 0b11111, 0b000000, 0b11111, 0b00000) => AArch64Inst::ERet(data),
                    (0b0100, 0b11111, 0b000010, 0b11111, 0b11111) => {
                        todo!("ERETAA, ERETAB - ERETAA variant")
                    }
                    (0b0100, 0b11111, 0b000011, 0b11111, 0b11111) => {
                        todo!("ERETAA, ERETAB - ERETAB variant")
                    }
                    (0b0101, 0b11111, 0b000000, 0b11111, 0b00000) => AArch64Inst::Drps(data),
                    (0b1000, 0b11111, 0b000010, _, _) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ - Key A, register modifier")
                    }
                    (0b1000, 0b11111, 0b000011, _, _) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ - Key B, register modifier")
                    }
                    (0b1001, 0b11111, 0b000010, _, _) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ - Key A, register modifier")
                    }
                    (0b1001, 0b11111, 0b000011, _, _) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ - Key B, register modifier")
                    }
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_uncond_branch_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_00101_xxxxxxxxxxxxxxxxxxxxxxxxxx"),
            |raw_instr: &[u8],
             Extract(op): Extract<u8, 31, 32>,
             Extract(imm26): Extract<u32, 0, 26>| {
                let data = Imm26 { imm26 };

                match op {
                    0b0 => AArch64Inst::BImm(data),
                    0b1 => AArch64Inst::BlImm(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_cond_branch_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0101010_x_xxxxxxxxxxxxxxxxxxx_x_xxxx"),
            |raw_instr: &[u8],
             Extract(o1): Extract<u8, 24, 25>,
             Extract(imm19): Extract<u32, 5, 24>,
             Extract(o0): Extract<u8, 4, 5>,
             Extract(cond): Extract<u8, 0, 4>| {
                let data = Imm19Cond { imm19, cond };

                match (o1, o0) {
                    (0b0, 0b0) => AArch64Inst::BCond(data),
                    (0b0, 0b1) => AArch64Inst::BcCond(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_cond_sel(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010100_xxxxx_xxxx_xx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(cond): Extract<u8, 12, 16>,
             Extract(op2): Extract<u8, 10, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RmCondRnRd {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    cond,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf_op_s, op2) {
                    (0b000, 0b00) => AArch64Inst::Csel32(data),
                    (0b000, 0b01) => AArch64Inst::Csinc32(data),
                    (0b010, 0b00) => AArch64Inst::Csinv32(data),
                    (0b010, 0b01) => AArch64Inst::Csneg32(data),
                    (0b100, 0b00) => AArch64Inst::Csel64(data),
                    (0b100, 0b01) => AArch64Inst::Csinc64(data),
                    (0b110, 0b00) => AArch64Inst::Csinv64(data),
                    (0b110, 0b01) => AArch64Inst::Csneg64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_test_and_branch_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_011011_x_xxxxx_xxxxxxxxxxxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(b5): Extract<u8, 31, 32>,
             Extract(op): Extract<u8, 24, 25>,
             Extract(b40): Extract<u8, 19, 24>,
             Extract(imm14): Extract<u16, 5, 19>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = B5B40Imm14Rt {
                    b5,
                    b40,
                    imm14,
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match op {
                    0b0 => AArch64Inst::Tbz(data),
                    0b1 => AArch64Inst::Tbnz(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_logical_shifted_register(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_01010_xx_x_xxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(opc): Extract<u8, 29, 31>,
             Extract(shift): Extract<u8, 22, 24>,
             Extract(n): Extract<u8, 21, 22>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(imm6): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = ShiftRmImm6RnRd {
                    shift,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    imm6,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, opc, n) {
                    (0b0, _, _) if imm6 & 0b100000 == 0b100000 => unreachable!(),
                    (0b0, 0b00, 0b0) => AArch64Inst::AndShiftedReg32(data),
                    (0b0, 0b00, 0b1) => AArch64Inst::BicShiftedReg32(data),
                    (0b0, 0b01, 0b0) => AArch64Inst::OrrShiftedReg32(data),
                    (0b0, 0b01, 0b1) => AArch64Inst::OrnShiftedReg32(data),
                    (0b0, 0b10, 0b0) => AArch64Inst::EorShiftedReg32(data),
                    (0b0, 0b10, 0b1) => AArch64Inst::EonShiftedReg32(data),
                    (0b0, 0b11, 0b0) => AArch64Inst::AndsShiftedReg32(data),
                    (0b0, 0b11, 0b1) => AArch64Inst::BicsShiftedReg32(data),
                    (0b1, 0b00, 0b0) => AArch64Inst::AndShiftedReg64(data),
                    (0b1, 0b00, 0b1) => AArch64Inst::BicShiftedReg64(data),
                    (0b1, 0b01, 0b0) => AArch64Inst::OrrShiftedReg64(data),
                    (0b1, 0b01, 0b1) => AArch64Inst::OrnShiftedReg64(data),
                    (0b1, 0b10, 0b0) => AArch64Inst::EorShiftedReg64(data),
                    (0b1, 0b10, 0b1) => AArch64Inst::EonShiftedReg64(data),
                    (0b1, 0b11, 0b0) => AArch64Inst::AndsShiftedReg64(data),
                    (0b1, 0b11, 0b1) => AArch64Inst::BicsShiftedReg64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_hints(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("11010101000000110010_xxxx_xxx_11111"),
            |raw_instr: &[u8],
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>| match (crm, op2) {
                (0b0000, 0b000) => AArch64Inst::Nop,
                (0b0000, 0b001) => AArch64Inst::Yield,
                (0b0000, 0b010) => AArch64Inst::Wfe,
                (0b0000, 0b011) => AArch64Inst::Wfi,
                (0b0000, 0b100) => AArch64Inst::Sev,
                (0b0000, 0b101) => AArch64Inst::Sevl,

                (0b0000, 0b111) => AArch64Inst::Xpaclri,
                (0b0001, 0b000) => AArch64Inst::Pacia1716Var,
                (0b0001, 0b010) => AArch64Inst::Pacib1716Var,
                (0b0001, 0b100) => AArch64Inst::Autia1716Var,
                (0b0001, 0b110) => AArch64Inst::Autib1716Var,

                (0b0011, 0b000) => AArch64Inst::PaciazVar,
                (0b0011, 0b001) => AArch64Inst::PaciaspVar,
                (0b0011, 0b010) => AArch64Inst::PacibzVar,
                (0b0011, 0b011) => AArch64Inst::PacibspVar,
                (0b0011, 0b100) => AArch64Inst::AutiazVar,
                (0b0011, 0b101) => AArch64Inst::AutiaspVar,
                (0b0011, 0b110) => AArch64Inst::AutibzVar,
                (0b0011, 0b111) => AArch64Inst::AutibspVar,
                _ => todo!("Unknown instruction {:?}", raw_instr),
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_pc_rel_addressing(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_10000_xxxxxxxxxxxxxxxxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(op): Extract<u8, 31, 32>,
             Extract(immlo): Extract<u8, 29, 31>,
             Extract(immhi): Extract<u32, 5, 24>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = PcRelAddressing {
                    immlo,
                    immhi,
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match op {
                    0b0 => AArch64Inst::Adr(data),
                    0b1 => AArch64Inst::Adrp(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_exception_gen(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("11010100_xxx_xxxxxxxxxxxxxxxx_xxx_xx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 21, 24>,
             Extract(imm16): Extract<u16, 5, 21>,
             Extract(op2): Extract<u8, 2, 5>,
             Extract(ll): Extract<u8, 0, 2>| {
                let data = ExceptionGen {
                    opc,
                    imm16,
                    op2,
                    ll,
                };

                match (opc, op2, ll) {
                    (0b000, 0b000, 0b01) => AArch64Inst::Svc(data),
                    (0b000, 0b000, 0b10) => AArch64Inst::Hvc(data),
                    (0b000, 0b000, 0b11) => AArch64Inst::Smc(data),
                    (0b001, 0b000, 0b00) => AArch64Inst::Brk(data),
                    (0b010, 0b000, 0b00) => AArch64Inst::Hlt(data),
                    (0b011, 0b000, 0b00) => AArch64Inst::TCancle(data),
                    (0b101, 0b000, 0b01) => AArch64Inst::DcpS1(data),
                    (0b101, 0b000, 0b10) => AArch64Inst::DcpS2(data),
                    (0b101, 0b000, 0b11) => AArch64Inst::DcpS3(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_reg_offset(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_xx_1_xxxxx_xxx_x_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(option): Extract<u8, 13, 16>,
             Extract(s): Extract<u8, 12, 13>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreRegRegOffset {
                    size,
                    v,
                    opc,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    option,
                    s,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, opc, option) {
                    (0b00, 0b0, 0b00, _) if option != 0b011 => AArch64Inst::StrbRegExtReg(data),
                    (0b00, 0b0, 0b00, 0b011) => AArch64Inst::StrbRegShiftedReg(data),
                    (0b00, 0b0, 0b01, _) if option != 0b011 => AArch64Inst::LdrbRegExtReg(data),
                    (0b00, 0b0, 0b01, 0b011) => AArch64Inst::LdrbRegShiftedReg(data),
                    (0b00, 0b0, 0b10, _) if option != 0b011 => AArch64Inst::LdrsbRegExtReg64(data),
                    (0b00, 0b0, 0b10, 0b011) => AArch64Inst::LdrsbRegShiftedReg64(data),
                    (0b00, 0b0, 0b11, _) if option != 0b011 => AArch64Inst::LdrsbRegExtReg32(data),
                    (0b00, 0b0, 0b11, 0b011) => AArch64Inst::LdrsbRegShiftedReg32(data),
                    (_, 0b1, 0b00, _) | (0b00, 0b1, 0b10, _) => AArch64Inst::StrRegSimdFP(data),
                    (_, 0b1, 0b01, _) | (0b00, 0b1, 0b11, _) => AArch64Inst::LdrRegSimdFP(data),
                    (0b01, 0b0, 0b00, _) => AArch64Inst::StrhReg(data),
                    (0b01, 0b0, 0b01, _) => AArch64Inst::LdrhReg(data),
                    (0b01, 0b0, 0b10, _) => AArch64Inst::LdrshReg64(data),
                    (0b01, 0b0, 0b11, _) => AArch64Inst::LdrshReg32(data),
                    (0b10, 0b0, 0b00, _) => AArch64Inst::StrReg32(data),
                    (0b10, 0b0, 0b01, _) => AArch64Inst::LdrReg32(data),
                    (0b10, 0b0, 0b10, _) => AArch64Inst::LdrswReg(data),
                    (0b11, 0b0, 0b00, _) => AArch64Inst::StrReg64(data),
                    (0b11, 0b0, 0b01, _) => AArch64Inst::LdrReg64(data),
                    (0b11, 0b0, 0b10, _) => AArch64Inst::PrfmReg(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_add_sub_ext_reg(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_01011_xx_1_xxxxx_xxx_xxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(opt): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(option): Extract<u8, 13, 16>,
             Extract(imm3): Extract<u8, 10, 13>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AddSubtractExtReg {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    option,
                    imm3,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rd: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rd,
                    ),
                };

                match (sf_op_s, opt) {
                    (0b000, 0b00) => AArch64Inst::AddExtReg32(data),
                    (0b001, 0b00) => AArch64Inst::AddsExtReg32(data),
                    (0b010, 0b00) => AArch64Inst::SubExtReg32(data),
                    (0b011, 0b00) => AArch64Inst::SubsExtReg32(data),
                    (0b100, 0b00) => AArch64Inst::AddExtReg64(data),
                    (0b101, 0b00) => AArch64Inst::AddsExtReg64(data),
                    (0b110, 0b00) => AArch64Inst::SubExtReg64(data),
                    (0b111, 0b00) => AArch64Inst::SubsExtReg64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_bitfield(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_100110_x_xxxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(opc): Extract<u8, 29, 31>,
             Extract(n): Extract<u8, 22, 23>,
             Extract(immr): Extract<u8, 16, 22>,
             Extract(imms): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = Bitfield {
                    n,
                    immr,
                    imms,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, opc, n) {
                    (0b0, 0b00, 0b0) => AArch64Inst::Sbfm32(data),
                    (0b0, 0b01, 0b0) => AArch64Inst::Bfm32(data),
                    (0b0, 0b10, 0b0) => AArch64Inst::Ubfm32(data),
                    (0b1, 0b00, 0b1) => AArch64Inst::Sbfm64(data),
                    (0b1, 0b01, 0b1) => AArch64Inst::Bfm64(data),
                    (0b1, 0b10, 0b1) => AArch64Inst::Ubfm64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_logical_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_100100_x_xxxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(opc): Extract<u8, 29, 31>,
             Extract(n): Extract<u8, 22, 23>,
             Extract(immr): Extract<u8, 16, 22>,
             Extract(imms): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = LogicalImm {
                    n,
                    immr,
                    imms,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rd,
                    ),
                };

                match (sf, opc, n) {
                    (0b0, 0b00, 0b0) => AArch64Inst::AndImm32(data),
                    (0b0, 0b01, 0b0) => AArch64Inst::OrrImm32(data),
                    (0b0, 0b10, 0b0) => AArch64Inst::EorImm32(data),
                    (0b0, 0b11, 0b0) => AArch64Inst::AndsImm32(data),
                    (0b1, 0b00, _) => AArch64Inst::AndImm64(data),
                    (0b1, 0b01, _) => AArch64Inst::OrrImm64(data),
                    (0b1, 0b10, _) => AArch64Inst::EorImm64(data),
                    (0b1, 0b11, _) => AArch64Inst::AndsImm64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_pair_offset(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_101_x_010_x_xxxxxxx_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(imm7): Extract<u8, 15, 22>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreRegPair {
                    opc,
                    imm7,
                    o: 0b010,
                    rt2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (opc, v, l) {
                    (0b00, 0b0, 0b0) => AArch64Inst::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Inst::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Inst::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Inst::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Inst::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Inst::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Inst::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Inst::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_add_sub_imm_with_tags(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_100011_x_xxxxxx_xx_xxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(o2): Extract<u8, 22, 23>,
             Extract(uimm6): Extract<u8, 16, 22>,
             Extract(op3): Extract<u8, 14, 16>,
             Extract(uimm4): Extract<u8, 10, 14>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AddSubImmWithTags {
                    o2,
                    uimm6,
                    op3,
                    uimm4,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rd: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rd,
                    ),
                };

                match (sf_op_s, o2) {
                    (0b100, 0b0) => AArch64Inst::Addg(data),
                    (0b110, 0b0) => AArch64Inst::Subg(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_extract(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_100111_x_x_xxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op21): Extract<u8, 29, 32>,
             Extract(n): Extract<u8, 22, 23>,
             Extract(o0): Extract<u8, 21, 22>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(imms): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = ExtractImm {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    imms,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf_op21, n, o0, imms) {
                    (0b000, 0b0, 0b0, imms) if (imms & 0b100000) == 0b000000 => {
                        AArch64Inst::Extr32(data)
                    }
                    (0b100, 1, 0, _) => AArch64Inst::Extr64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_data_proc_1src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_1_x_11010110_xxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(opcode2): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RnRd {
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, s, opcode2, opcode) {
                    (0b0, 0b0, 0b00000, 0b000000) => AArch64Inst::RbitVar32(data),
                    (0b0, 0b0, 0b00000, 0b000001) => AArch64Inst::Rev16Var32(data),
                    (0b0, 0b0, 0b00000, 0b000010) => AArch64Inst::RevVar32(data),
                    (0b0, 0b0, 0b00000, 0b000100) => AArch64Inst::ClzVar32(data),
                    (0b0, 0b0, 0b00000, 0b000101) => AArch64Inst::ClsVar32(data),
                    (0b1, 0b0, 0b00000, 0b000000) => AArch64Inst::RbitVar64(data),
                    (0b1, 0b0, 0b00000, 0b000001) => AArch64Inst::Rev16Var64(data),
                    (0b1, 0b0, 0b00000, 0b000010) => AArch64Inst::Rev32(data),
                    (0b1, 0b0, 0b00000, 0b000011) => AArch64Inst::RevVar64(data),
                    (0b1, 0b0, 0b00000, 0b000100) => AArch64Inst::ClzVar64(data),
                    (0b1, 0b0, 0b00000, 0b000101) => AArch64Inst::ClsVar64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_cmp_and_branch_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_011010_x_xxxxxxxxxxxxxxxxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(op): Extract<u8, 24, 25>,
             Extract(imm19): Extract<u32, 5, 24>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Imm19Rt {
                    imm19,
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (sf, op) {
                    (0b0, 0b0) => AArch64Inst::Cbz32(data),
                    (0b0, 0b1) => AArch64Inst::Cbnz32(data),
                    (0b1, 0b0) => AArch64Inst::Cbz64(data),
                    (0b1, 0b1) => AArch64Inst::Cbnz64(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_data_proccessing_3src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_xx_11011_xxx_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(op54): Extract<u8, 29, 31>,
             Extract(op31): Extract<u8, 21, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(ra): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = DataProc3Src {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    ra: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, ra),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, op54, op31, o0) {
                    (0b0, 0b00, 0b000, 0b0) => AArch64Inst::Madd32(data),
                    (0b0, 0b00, 0b000, 0b1) => AArch64Inst::Msub32(data),
                    (0b1, 0b00, 0b000, 0b0) => AArch64Inst::Madd64(data),
                    (0b1, 0b00, 0b000, 0b1) => AArch64Inst::Msub64(data),
                    (0b1, 0b00, 0b001, 0b0) => AArch64Inst::Smaddl(data),
                    (0b1, 0b00, 0b001, 0b1) => AArch64Inst::Smsubl(data),
                    (0b1, 0b00, 0b010, 0b0) => AArch64Inst::Smulh(data),
                    (0b1, 0b00, 0b101, 0b0) => AArch64Inst::Umaddl(data),
                    (0b1, 0b00, 0b101, 0b1) => AArch64Inst::Umsubl(data),
                    (0b1, 0b00, 0b110, 0b0) => AArch64Inst::Umulh(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_unscaled_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_xx_0_xxxxxxxxx_00_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm9): Extract<u16, 12, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LdStRegUnscaledImm {
                    imm9,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, opc) {
                    (0b00, 0b0, 0b00) => AArch64Inst::Sturb(data),
                    (0b00, 0b0, 0b01) => AArch64Inst::Ldurb(data),
                    (0b00, 0b0, 0b10) => AArch64Inst::Ldursb64(data),
                    (0b00, 0b0, 0b11) => AArch64Inst::Ldursb32(data),
                    (0b00, 0b1, 0b00) => AArch64Inst::SturSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Inst::LdurSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Inst::SturSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Inst::LdurSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Inst::Sturh(data),
                    (0b01, 0b0, 0b01) => AArch64Inst::Ldurh(data),
                    (0b01, 0b0, 0b10) => AArch64Inst::Ldursh64(data),
                    (0b01, 0b0, 0b11) => AArch64Inst::Ldursh32(data),
                    (0b01, 0b1, 0b00) => AArch64Inst::SturSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Inst::LdurSimdFP16(data),
                    (0b10, 0b0, 0b00) => AArch64Inst::Stur32(data),
                    (0b10, 0b0, 0b01) => AArch64Inst::Ldur32(data),
                    (0b10, 0b0, 0b10) => AArch64Inst::Ldursw(data),
                    (0b10, 0b1, 0b00) => AArch64Inst::SturSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Inst::LdurSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Inst::Stur64(data),
                    (0b11, 0b0, 0b01) => AArch64Inst::Ldur64(data),
                    (0b11, 0b0, 0b10) => AArch64Inst::Prefum(data),
                    (0b11, 0b1, 0b00) => AArch64Inst::SturSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Inst::LdurSimdFP64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_sys_reg_mov(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1101010100_x_1_x_xxx_xxxx_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(l): Extract<u8, 21, 22>,
             Extract(o0): Extract<u8, 19, 20>,
             Extract(op1): Extract<u8, 16, 19>,
             Extract(crn): Extract<u8, 12, 16>,
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = SysRegMov {
                    o0,
                    op1,
                    crn,
                    crm,
                    op2,
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match l {
                    0 => AArch64Inst::MsrReg(data),
                    1 => AArch64Inst::Mrs(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_pair_pre_indexed(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_101_x_011_x_xxxxxxx_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(imm7): Extract<u8, 15, 22>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreRegPair {
                    opc,
                    o: 0b011,
                    imm7,
                    rt2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (opc, v, l) {
                    (0b00, 0b0, 0b0) => AArch64Inst::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Inst::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Inst::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Inst::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Inst::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Inst::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Inst::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Inst::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_pair_post_indexed(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_101_x_001_x_xxxxxxx_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(imm7): Extract<u8, 15, 22>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreRegPair {
                    opc,
                    o: 0b001,
                    imm7,
                    rt2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (opc, v, l) {
                    (0b00, 0b0, 0b0) => AArch64Inst::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Inst::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Inst::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Inst::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Inst::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Inst::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Inst::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Inst::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_data_proc_2src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11010110_xxxxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = DataProc2Src {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, s, opcode) {
                    (0b0, 0b0, 0b000010) => AArch64Inst::UdivVar32(data),
                    (0b0, 0b0, 0b000011) => AArch64Inst::SdivVar32(data),
                    (0b0, 0b0, 0b001000) => AArch64Inst::LslvVar32(data),
                    (0b0, 0b0, 0b001001) => AArch64Inst::LsrvVar32(data),
                    (0b0, 0b0, 0b001010) => AArch64Inst::AsrvVar32(data),
                    (0b0, 0b0, 0b001011) => AArch64Inst::RorvVar32(data),
                    (0b1, 0b0, 0b000010) => AArch64Inst::UdivVar64(data),
                    (0b1, 0b0, 0b000011) => AArch64Inst::SdivVar64(data),
                    (0b1, 0b0, 0b001000) => AArch64Inst::LslvVar64(data),
                    (0b1, 0b0, 0b001001) => AArch64Inst::LsrvVar64(data),
                    (0b1, 0b0, 0b001010) => AArch64Inst::AsrvVar64(data),
                    (0b1, 0b0, 0b001011) => AArch64Inst::RorvVar64(data),

                    (0b1, 0b0, 0b001100) => AArch64Inst::Pacga(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_imm_pre_indexed(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_xx_0_xxxxxxxxx_11_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(idxt): Extract<u8, 24, 26>, // Indexing type
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm12): Extract<u16, 10, 22>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = OpcSizeImm12RnRt {
                    idxt,
                    opc,
                    size,
                    imm12,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, opc) {
                    (0b00, 0b0, 0b00) => AArch64Inst::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Inst::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Inst::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Inst::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Inst::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Inst::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Inst::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Inst::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Inst::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Inst::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Inst::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Inst::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP16(data),

                    (0b10, 0b0, 0b00) => AArch64Inst::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Inst::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Inst::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Inst::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Inst::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Inst::LdrImm64(data),
                    (0b11, 0b1, 0b00) => AArch64Inst::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_imm_post_indexed(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_xx_0_xxxxxxxxx_01_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(idxt): Extract<u8, 24, 26>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm12): Extract<u16, 10, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = OpcSizeImm12RnRt {
                    idxt,
                    opc,
                    size,
                    imm12,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, opc) {
                    (0b00, 0b0, 0b00) => AArch64Inst::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Inst::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Inst::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Inst::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Inst::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Inst::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Inst::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Inst::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Inst::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Inst::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Inst::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Inst::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP16(data),

                    (0b10, 0b0, 0b00) => AArch64Inst::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Inst::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Inst::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Inst::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Inst::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Inst::LdrImm64(data),
                    (0b11, 0b1, 0b00) => AArch64Inst::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Inst::LdrImmSimdFP64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_barriers(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("11010101000000110011_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Barriers { crm };

                match (crm, op2, rt) {
                    (_, 0b010, 0b11111) => AArch64Inst::Clrex(data),
                    (_, 0b100, 0b11111) => AArch64Inst::DsbEncoding(data),
                    (_, 0b101, 0b11111) => AArch64Inst::Dmb(data),
                    (_, 0b110, 0b11111) => AArch64Inst::Isb(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_advanced_simd_copy(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01110000_xxxxx_0_xxxx_1_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(op): Extract<u8, 29, 30>,
             Extract(imm5): Extract<u8, 16, 21>,
             Extract(imm4): Extract<u8, 11, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvancedSimdCopy {
                    q,
                    imm5,
                    imm4,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (q, op, imm5, imm4) {
                    (_, 0b0, _, 0b0000) => AArch64Inst::DupElement(data),
                    (_, 0b0, _, 0b0001) => AArch64Inst::DupGeneral(data),
                    (0b0 | 0b1, 0b0, _, 0b0101) => AArch64Inst::Smov(data),
                    (0b0, 0b0, _, 0b0111) => AArch64Inst::Umov(data),
                    (0b1, 0b0, 0b01000 | 0b11000, 0b0111) => AArch64Inst::Umov(data),
                    (0b1, 0b0, _, 0b0011) => AArch64Inst::InsGeneral(data),
                    (0b1, 0b1, _, _) => AArch64Inst::InsElement(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_cond_cmp_reg(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010010_xxxxx_xxxx_0_x_xxxxx_x_xxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(cond): Extract<u8, 12, 16>,
             Extract(o2): Extract<u8, 10, 11>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(o3): Extract<u8, 4, 5>,
             Extract(nzcv): Extract<u8, 0, 4>| {
                let data = CondCmpReg {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    cond,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    nzcv,
                };

                match (sf_op_s, o2, o3) {
                    (0b001, 0b0, 0b0) => AArch64Inst::CcmnRegVar32(data),
                    (0b011, 0b0, 0b0) => AArch64Inst::CcmpRegVar32(data),
                    (0b101, 0b0, 0b0) => AArch64Inst::CcmnRegVar64(data),
                    (0b111, 0b0, 0b0) => AArch64Inst::CcmpRegVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_ld_st_multi_structures(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_0011000_x_000000_xxxx_xx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(opcode): Extract<u8, 12, 16>,
             Extract(size): Extract<u8, 10, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = AdvSimdLdStMultiStructures {
                    q,
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt,
                };

                match (l, opcode) {
                    (0b0, 0b0000) => AArch64Inst::St4MulStructures(data),
                    (0b0, 0b0010) => AArch64Inst::St1MulStructures4RegsVar(data),
                    (0b0, 0b0100) => AArch64Inst::St3MulStructures(data),
                    (0b0, 0b0110) => AArch64Inst::St1MulStructures3RegsVar(data),
                    (0b0, 0b0111) => AArch64Inst::St1MulStructures1RegsVar(data),
                    (0b0, 0b1000) => AArch64Inst::St2MulStructures(data),
                    (0b0, 0b1010) => AArch64Inst::St1MulStructures2RegsVar(data),

                    (0b1, 0b0000) => AArch64Inst::Ld4MulStructures(data),
                    (0b1, 0b0010) => AArch64Inst::Ld1MulStructures4RegsVar(data),
                    (0b1, 0b0100) => AArch64Inst::Ld3MulStructures(data),
                    (0b1, 0b0110) => AArch64Inst::Ld1MulStructures3RegsVar(data),
                    (0b1, 0b0111) => AArch64Inst::Ld1MulStructures1RegsVar(data),
                    (0b1, 0b1000) => AArch64Inst::Ld2MulStructures(data),
                    (0b1, 0b1010) => AArch64Inst::Ld1MulStructures2RegsVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_advanced_simd_extract(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_101110_xx_0_xxxxx_0_xxxx_0_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(op2): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(imm4): Extract<u8, 11, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvancedSimdExtract {
                    q,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    imm4,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match op2 {
                    0b00 => AArch64Inst::Ext(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_ld_st_multi_structures_post_indexed(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_0011001_x_0_xxxxx_xxxx_xx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 12, 16>,
             Extract(size): Extract<u8, 10, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = AdvSimdLdStMultiStructuresPostIndexed {
                    q,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt,
                };

                match (l, rm, opcode) {
                    (0b0, rm, 0b0000) if rm != 0b11111 => {
                        AArch64Inst::St4MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0010) if rm != 0b11111 => {
                        AArch64Inst::St1MulStructures4RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0100) if rm != 0b11111 => {
                        AArch64Inst::St3MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0110) if rm != 0b11111 => {
                        AArch64Inst::St1MulStructures3RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0111) if rm != 0b11111 => {
                        AArch64Inst::St1MulStructures1RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b1000) if rm != 0b11111 => {
                        AArch64Inst::St2MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b1010) if rm != 0b11111 => {
                        AArch64Inst::St1MulStructures2RegRegOffsetVar(data)
                    }

                    (0b0, 0b11111, 0b0000) => AArch64Inst::St4MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b0010) => AArch64Inst::St1MulStructures4RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b0100) => AArch64Inst::St3MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b0110) => AArch64Inst::St1MulStructures3RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b0111) => AArch64Inst::St1MulStructures1RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b1000) => AArch64Inst::St2MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b1010) => AArch64Inst::St1MulStructures2RegImmOffsetVar(data),

                    (0b1, rm, 0b0000) if rm != 0b11111 => {
                        AArch64Inst::Ld4MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0010) if rm != 0b11111 => {
                        AArch64Inst::Ld1MulStructures4RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0100) if rm != 0b11111 => {
                        AArch64Inst::Ld3MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0110) if rm != 0b11111 => {
                        AArch64Inst::Ld1MulStructures3RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0111) if rm != 0b11111 => {
                        AArch64Inst::Ld1MulStructures1RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b1000) if rm != 0b11111 => {
                        AArch64Inst::Ld2MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b1010) if rm != 0b11111 => {
                        AArch64Inst::Ld1MulStructures2RegRegOffsetVar(data)
                    }

                    (0b1, 0b11111, 0b0000) => AArch64Inst::Ld4MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b0010) => AArch64Inst::Ld1MulStructures4RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b0100) => AArch64Inst::Ld3MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b0110) => AArch64Inst::Ld1MulStructures3RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b0111) => AArch64Inst::Ld1MulStructures1RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b1000) => AArch64Inst::Ld2MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b1010) => AArch64Inst::Ld1MulStructures2RegImmOffsetVar(data),
                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_conv_between_float_and_int(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xx_xxx_000000_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(rmode): Extract<u8, 19, 21>,
             Extract(opcode): Extract<u8, 16, 19>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RnRd {
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match (sf, s, ptype, rmode, opcode) {
                    (0b0, 0b0, 0b00, 0b00, 0b000) => {
                        AArch64Inst::FcvtnsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b001) => {
                        AArch64Inst::FcvtnuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b010) => {
                        AArch64Inst::ScvtfScalarInt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b011) => {
                        AArch64Inst::UcvtfScalarInt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b100) => {
                        AArch64Inst::FcvtasScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b101) => {
                        AArch64Inst::FcvtauScalarSinglePrecisionTo32(data)
                    }

                    (0b0, 0b0, 0b00, 0b00, 0b110) => {
                        AArch64Inst::FmovGeneralSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b111) => {
                        AArch64Inst::FmovGeneral32ToSinglePrecision(data)
                    }

                    (0b0, 0b0, 0b00, 0b01, 0b000) => {
                        AArch64Inst::FcvtpsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b01, 0b001) => {
                        AArch64Inst::FcvtpuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b10, 0b000) => {
                        AArch64Inst::FcvtmsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b10, 0b001) => {
                        AArch64Inst::FcvtmuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b000) => {
                        AArch64Inst::FcvtzsScalarIntSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b001) => {
                        AArch64Inst::FcvtzuScalarIntSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b000) => {
                        AArch64Inst::FcvtnsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b001) => {
                        AArch64Inst::FcvtnuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b010) => {
                        AArch64Inst::ScvtfScalarInt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b011) => {
                        AArch64Inst::UcvtfScalarInt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b100) => {
                        AArch64Inst::FcvtasScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b101) => {
                        AArch64Inst::FcvtauScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b01, 0b000) => {
                        AArch64Inst::FcvtpsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b01, 0b001) => {
                        AArch64Inst::FcvtpuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b10, 0b000) => {
                        AArch64Inst::FcvtmsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b10, 0b001) => {
                        AArch64Inst::FcvtmuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b000) => {
                        AArch64Inst::FcvtzsScalarIntDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b001) => {
                        AArch64Inst::FcvtzsScalarIntDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b110) => AArch64Inst::Fjcvtzs(data),

                    (0b1, 0b0, 0b00, 0b00, 0b000) => {
                        AArch64Inst::FcvtnsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b001) => {
                        AArch64Inst::FcvtnuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b010) => {
                        AArch64Inst::ScvtfScalarInt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b011) => {
                        AArch64Inst::UcvtfScalarInt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b100) => {
                        AArch64Inst::FcvtasScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b101) => {
                        AArch64Inst::FcvtauScalarSinglePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b01, 0b00, 0b110) => {
                        AArch64Inst::FmovGeneralDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b111) => {
                        AArch64Inst::FmovGeneral64ToDoublePrecision(data)
                    }

                    (0b1, 0b0, 0b00, 0b01, 0b000) => {
                        AArch64Inst::FcvtpsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b01, 0b001) => {
                        AArch64Inst::FcvtpuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b10, 0b000) => {
                        AArch64Inst::FcvtmsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b10, 0b001) => {
                        AArch64Inst::FcvtmuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b000) => {
                        AArch64Inst::FcvtzsScalarIntSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b001) => {
                        AArch64Inst::FcvtzuScalarIntSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b000) => {
                        AArch64Inst::FcvtnsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b001) => {
                        AArch64Inst::FcvtnuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b010) => {
                        AArch64Inst::ScvtfScalarInt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b011) => {
                        AArch64Inst::UcvtfScalarInt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b100) => {
                        AArch64Inst::FcvtasScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b101) => {
                        AArch64Inst::FcvtauScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b01, 0b000) => {
                        AArch64Inst::FcvtpsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b01, 0b001) => {
                        AArch64Inst::FcvtpuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b10, 0b000) => {
                        AArch64Inst::FcvtmsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b10, 0b001) => {
                        AArch64Inst::FcvtmuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b000) => {
                        AArch64Inst::FcvtzsScalarIntDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b001) => {
                        AArch64Inst::FcvtzsScalarIntDoublePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b10, 0b01, 0b110) => AArch64Inst::FmovGeneralTopHalfOf128To64(data),
                    (0b1, 0b0, 0b10, 0b01, 0b111) => AArch64Inst::FmovGeneral64toTopHalfOf128(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_modified_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01111_00000_x_x_x_xxxx_x_1_x_x_x_x_x_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(op): Extract<u8, 29, 30>,
             Extract(a): Extract<u8, 18, 19>,
             Extract(b): Extract<u8, 17, 18>,
             Extract(c): Extract<u8, 16, 17>,
             Extract(cmode): Extract<u8, 12, 16>,
             Extract(cmode3): Extract<u8, 12, 13>,
             Extract(cmode2): Extract<u8, 13, 14>,
             Extract(cmode1): Extract<u8, 14, 15>,
             Extract(cmode0): Extract<u8, 15, 16>,
             Extract(o2): Extract<u8, 11, 12>,
             Extract(d): Extract<u8, 9, 10>,
             Extract(e): Extract<u8, 8, 9>,
             Extract(f): Extract<u8, 7, 8>,
             Extract(g): Extract<u8, 6, 7>,
             Extract(h): Extract<u8, 5, 6>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvSimdModifiedImm {
                    q,
                    op,
                    a,
                    b,
                    c,
                    cmode,
                    d,
                    e,
                    f,
                    g,
                    h,
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (q, op, cmode0, cmode1, cmode2, cmode3, o2) {
                    (_, 0b0, 0, _, _, 0, 0b0) => AArch64Inst::MoviShiftedImmVar32(data),
                    (_, 0b0, 0, _, _, 1, 0b0) => AArch64Inst::OrrVecImmVar32(data),
                    (_, 0b0, 1, 0, _, 0, 0b0) => AArch64Inst::MoviShiftedImmVar16(data),
                    (_, 0b0, 1, 0, _, 1, 0b0) => AArch64Inst::OrrVecImmVar16(data),
                    (_, 0b0, 1, 1, 0, _, 0b0) => AArch64Inst::MoviShiftingOnesVar32(data),
                    (_, 0b0, 1, 1, 1, 0, 0b0) => AArch64Inst::MoviVar8(data),
                    (_, 0b0, 1, 1, 1, 1, 0b0) => AArch64Inst::FmovVecImmSinglePrecisionVar(data),

                    (_, 0b1, 0, _, _, 0, 0b0) => AArch64Inst::MvniShiftedImmVar32(data),
                    (_, 0b1, 0, _, _, 1, 0b0) => AArch64Inst::BicVecImmVar32(data),
                    (_, 0b1, 1, 0, _, 0, 0b0) => AArch64Inst::MvniShiftedImmVar16(data),
                    (_, 0b1, 1, 0, _, 1, 0b0) => AArch64Inst::BicVecImmVar16(data),

                    (_, 0b1, 1, 1, 0, _, 0b0) => AArch64Inst::MvniShiftingOnesVar32(data),
                    (0b0, 0b1, 1, 1, 1, 0, 0b0) => AArch64Inst::MoviScalarVar64(data),

                    (0b1, 0b1, 1, 1, 1, 0, 0b0) => AArch64Inst::MoviVectorVar64(data),
                    (0b1, 0b1, 1, 1, 1, 1, 0b0) => AArch64Inst::FmovVecImmDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_cond_cmp_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010010_xxxxx_xxxx_1_x_xxxxx_x_xxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(imm5): Extract<u8, 16, 21>,
             Extract(cond): Extract<u8, 12, 16>,
             Extract(o2): Extract<u8, 10, 11>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(o3): Extract<u8, 4, 5>,
             Extract(nzcv): Extract<u8, 0, 4>| {
                let data = CondCmpImm {
                    imm5,
                    cond,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    nzcv,
                };

                match (sf_op_s, o2, o3) {
                    (0b001, 0b0, 0b0) => AArch64Inst::CcmnImmVar32(data),
                    (0b011, 0b0, 0b0) => AArch64Inst::CcmpImmVar32(data),
                    (0b101, 0b0, 0b0) => AArch64Inst::CcmnImmVar64(data),
                    (0b111, 0b0, 0b0) => AArch64Inst::CcmpImmVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_exclusive_register(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_0010000_x_0_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = RsRt2RnRt {
                    rs: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rs),
                    rt2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, l, o0) {
                    (0b00, 0b0, 0b0) => AArch64Inst::Stxrb(data),
                    (0b00, 0b1, 0b0) => AArch64Inst::Ldxrb(data),
                    (0b01, 0b0, 0b0) => AArch64Inst::Stxrh(data),
                    (0b01, 0b1, 0b0) => AArch64Inst::Ldxrh(data),
                    (0b10, 0b0, 0b0) => AArch64Inst::StxrVar32(data),
                    (0b10, 0b1, 0b0) => AArch64Inst::LdxrVar32(data),
                    (0b11, 0b0, 0b0) => AArch64Inst::StxrVar64(data),
                    (0b11, 0b1, 0b0) => AArch64Inst::LdxrVar64(data),

                    (0b00, 0b0, 0b1) => AArch64Inst::Stlxrb(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::Ldaxrb(data),
                    (0b01, 0b0, 0b1) => AArch64Inst::Stlxrh(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::Ldaxrh(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::StlxrVar32(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdaxrVar32(data),
                    (0b11, 0b0, 0b1) => AArch64Inst::StlxrVar64(data),
                    (0b11, 0b1, 0b1) => AArch64Inst::LdaxrVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_ordered(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_0010001_x_0_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = RsRt2RnRt {
                    rs: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rs),
                    rt2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, l, o0) {
                    (0b00, 0b0, 0b1) => AArch64Inst::Stlrb(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::Ldarb(data),
                    (0b01, 0b0, 0b1) => AArch64Inst::Stlrh(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::Ldarh(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::StlrVar32(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdarVar32(data),
                    (0b11, 0b0, 0b1) => AArch64Inst::StlrVar64(data),
                    (0b11, 0b1, 0b1) => AArch64Inst::LdarVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_advanced_simd_three_same(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01110_xx_1_xxxxx_xxxxx_1_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 11, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = QSizeRmRnRd {
                    q,
                    size,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b00000) => AArch64Inst::Shadd(data),
                    (0b0, _, 0b00001) => AArch64Inst::Sqadd(data),
                    (0b0, _, 0b00010) => AArch64Inst::Srhadd(data),
                    (0b0, _, 0b00100) => AArch64Inst::Shsub(data),
                    (0b0, _, 0b00101) => AArch64Inst::Sqsub(data),
                    (0b0, _, 0b00110) => AArch64Inst::CmgtReg(data),
                    (0b0, _, 0b00111) => AArch64Inst::CmgeReg(data),
                    (0b0, _, 0b01000) => AArch64Inst::Sshl(data),
                    (0b0, _, 0b01001) => AArch64Inst::SqshlReg(data),
                    (0b0, _, 0b01010) => AArch64Inst::Srshl(data),
                    (0b0, _, 0b01011) => AArch64Inst::Sqrshl(data),
                    (0b0, _, 0b01100) => AArch64Inst::Smax(data),
                    (0b0, _, 0b01101) => AArch64Inst::Smin(data),
                    (0b0, _, 0b01110) => AArch64Inst::Sabd(data),
                    (0b0, _, 0b01111) => AArch64Inst::Saba(data),
                    (0b0, _, 0b10000) => AArch64Inst::AddVec(data),
                    (0b0, _, 0b10001) => AArch64Inst::Cmtst(data),
                    (0b0, _, 0b10010) => AArch64Inst::MlaVec(data),
                    (0b0, _, 0b10011) => AArch64Inst::MulVec(data),
                    (0b0, _, 0b10100) => AArch64Inst::Smaxp(data),
                    (0b0, _, 0b10101) => AArch64Inst::Sminp(data),
                    (0b0, _, 0b10110) => AArch64Inst::SqdmulhVec(data),
                    (0b0, _, 0b10111) => AArch64Inst::AddpVec(data),

                    (0b0, 0b00 | 0b01, 0b11000) => AArch64Inst::FmaxnmVec(data),
                    (0b0, 0b00 | 0b01, 0b11001) => AArch64Inst::FmlaVec(data),
                    (0b0, 0b00 | 0b01, 0b11010) => AArch64Inst::FaddVec(data),
                    (0b0, 0b00 | 0b01, 0b11011) => AArch64Inst::Fmulx(data),
                    (0b0, 0b00 | 0b01, 0b11100) => AArch64Inst::FcmeqReg(data),
                    (0b0, 0b00 | 0b01, 0b11110) => AArch64Inst::FmaxVec(data),
                    (0b0, 0b00 | 0b01, 0b11111) => AArch64Inst::Frecps(data),

                    (0b0, 0b00, 0b00011) => AArch64Inst::AndVec(data),
                    (0b0, 0b01, 0b00011) => AArch64Inst::AndVec(data),

                    (0b0, 0b10 | 0b11, 0b11000) => AArch64Inst::FminnmVec(data),
                    (0b0, 0b10 | 0b11, 0b11001) => AArch64Inst::FmlsVec(data),
                    (0b0, 0b10 | 0b11, 0b11010) => AArch64Inst::FsubVec(data),
                    (0b0, 0b10 | 0b11, 0b11110) => AArch64Inst::FminVec(data),
                    (0b0, 0b10 | 0b11, 0b11111) => AArch64Inst::Frsqrts(data),

                    (0b0, 0b10, 0b00011) => AArch64Inst::OrrVecReg(data),
                    (0b0, 0b11, 0b00011) => AArch64Inst::OrnVec(data),

                    (0b1, _, 0b00000) => AArch64Inst::Uhadd(data),
                    (0b1, _, 0b00001) => AArch64Inst::Uqadd(data),
                    (0b1, _, 0b00010) => AArch64Inst::Urhadd(data),
                    (0b1, _, 0b00100) => AArch64Inst::Uhsub(data),
                    (0b1, _, 0b00101) => AArch64Inst::Uqsub(data),
                    (0b1, _, 0b00110) => AArch64Inst::CmhiReg(data),
                    (0b1, _, 0b00111) => AArch64Inst::CmhsReg(data),
                    (0b1, _, 0b01000) => AArch64Inst::Ushl(data),
                    (0b1, _, 0b01001) => AArch64Inst::UqshlReg(data),
                    (0b1, _, 0b01010) => AArch64Inst::Urshl(data),
                    (0b1, _, 0b01011) => AArch64Inst::Uqrshl(data),
                    (0b1, _, 0b01100) => AArch64Inst::Umax(data),
                    (0b1, _, 0b01101) => AArch64Inst::Umin(data),
                    (0b1, _, 0b01110) => AArch64Inst::Uabd(data),
                    (0b1, _, 0b01111) => AArch64Inst::Uaba(data),

                    (0b1, _, 0b10000) => AArch64Inst::SubVec(data),
                    (0b1, _, 0b10001) => AArch64Inst::CmeqReg(data),
                    (0b1, _, 0b10010) => AArch64Inst::MlsVec(data),
                    (0b1, _, 0b10011) => AArch64Inst::Pmul(data),
                    (0b1, _, 0b10100) => AArch64Inst::Umaxp(data),
                    (0b1, _, 0b10101) => AArch64Inst::Uminp(data),
                    (0b1, _, 0b10110) => AArch64Inst::SqrdmulhVec(data),

                    (0b1, 0b00 | 0b01, 0b11000) => AArch64Inst::FmaxnmpVec(data),
                    (0b1, 0b00 | 0b01, 0b11010) => AArch64Inst::FaddpVec(data),
                    (0b1, 0b00 | 0b01, 0b11011) => AArch64Inst::FmulVec(data),
                    (0b1, 0b00 | 0b01, 0b11100) => AArch64Inst::FcmgeReg(data),
                    (0b1, 0b00 | 0b01, 0b11101) => AArch64Inst::Facge(data),
                    (0b1, 0b00 | 0b01, 0b11110) => AArch64Inst::FmaxpVec(data),
                    (0b1, 0b00 | 0b01, 0b11111) => AArch64Inst::FdivVec(data),

                    (0b1, 0b00, 0b00011) => AArch64Inst::EorVec(data),
                    (0b1, 0b01, 0b00011) => AArch64Inst::Bsl(data),

                    (0b1, 0b10 | 0b11, 0b11000) => AArch64Inst::FminnmpVec(data),
                    (0b1, 0b10 | 0b11, 0b11010) => AArch64Inst::Fabd(data),
                    (0b1, 0b10 | 0b11, 0b11100) => AArch64Inst::FcmgtReg(data),
                    (0b1, 0b10 | 0b11, 0b11101) => AArch64Inst::Facgt(data),
                    (0b1, 0b10 | 0b11, 0b11110) => AArch64Inst::FminpVec(data),

                    (0b1, 0b10, 0b00011) => AArch64Inst::Bit(data),
                    (0b1, 0b11, 0b00011) => AArch64Inst::Bif(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_shift_by_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_011110_xxxx_xxx_xxxxx_1_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(u): Extract<u8, 29, 30>,
             Extract(immb): Extract<u8, 16, 19>,
             Extract(opcode): Extract<u8, 11, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvSimdShiftByImm {
                    q,
                    immb,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, opcode) {
                    (0b0, 0b00000) => AArch64Inst::Sshr(data),
                    (0b0, 0b00010) => AArch64Inst::Ssra(data),
                    (0b0, 0b00100) => AArch64Inst::Srshr(data),
                    (0b0, 0b00110) => AArch64Inst::Srsra(data),
                    (0b0, 0b01010) => AArch64Inst::Shl(data),
                    (0b0, 0b01110) => AArch64Inst::SqshlImm(data),
                    (0b0, 0b10000) => AArch64Inst::Shrn(data),
                    (0b0, 0b10001) => AArch64Inst::Rshrn(data),
                    (0b0, 0b10010) => AArch64Inst::Sqshrn(data),
                    (0b0, 0b10011) => AArch64Inst::Sqrshrn(data),
                    (0b0, 0b10100) => AArch64Inst::Sshll(data),
                    (0b0, 0b11100) => AArch64Inst::ScvtfVecFixedPt(data),
                    (0b0, 0b11111) => AArch64Inst::FcvtzsVecFixedPt(data),

                    (0b1, 0b00000) => AArch64Inst::Ushr(data),
                    (0b1, 0b00010) => AArch64Inst::Usra(data),
                    (0b1, 0b00100) => AArch64Inst::Urshr(data),
                    (0b1, 0b00110) => AArch64Inst::Ursra(data),

                    (0b1, 0b01000) => AArch64Inst::Sri(data),
                    (0b1, 0b01010) => AArch64Inst::Sli(data),

                    (0b1, 0b01100) => AArch64Inst::Sqshlu(data),
                    (0b1, 0b01110) => AArch64Inst::UqshlImm(data),

                    (0b1, 0b10000) => AArch64Inst::Sqshrun(data),
                    (0b1, 0b10001) => AArch64Inst::Sqrshrun(data),
                    (0b1, 0b10010) => AArch64Inst::Uqshrn(data),
                    (0b1, 0b10011) => AArch64Inst::Uqrshrn(data),
                    (0b1, 0b10100) => AArch64Inst::Ushll(data),
                    (0b1, 0b11100) => AArch64Inst::UcvtfVecFixedPt(data),
                    (0b1, 0b11111) => AArch64Inst::FcvtzuVecFixedPt(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_float_data_proc_1src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xxxxxx_10000_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(opcode): Extract<u8, 15, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RnRd {
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (m, s, ptype, opcode) {
                    (0b0, 0b0, 0b00, 0b000000) => AArch64Inst::FmovRegSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000001) => AArch64Inst::FabsScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000010) => AArch64Inst::FnegScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000011) => AArch64Inst::FsqrtScalarSinglePrecisionVar(data),

                    (0b0, 0b0, 0b00, 0b000101) => AArch64Inst::FcvtSingleToDoublePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000111) => AArch64Inst::FcvtSingleToHalfPrecisionVar(data),

                    (0b0, 0b0, 0b00, 0b001000) => AArch64Inst::FrintnScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001001) => AArch64Inst::FrintpScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001010) => AArch64Inst::FrintmScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001011) => AArch64Inst::FrintzScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001100) => AArch64Inst::FrintaScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001110) => AArch64Inst::FrintxScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b001111) => AArch64Inst::FrintiScalarSinglePrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b000000) => AArch64Inst::FmovRegDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000001) => AArch64Inst::FabsScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000010) => AArch64Inst::FnegScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000011) => AArch64Inst::FsqrtScalarDoublePrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b000100) => AArch64Inst::FcvtDoubleToSinglePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000111) => AArch64Inst::FcvtDoubleToHalfPrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b001000) => AArch64Inst::FrintnScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001001) => AArch64Inst::FrintpScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001010) => AArch64Inst::FrintmScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001011) => AArch64Inst::FrintzScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001100) => AArch64Inst::FrintaScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001110) => AArch64Inst::FrintxScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b001111) => AArch64Inst::FrintiScalarDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_scalar_pairwise(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("01_x_11110_xx_11000_xxxxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(opcode): Extract<u8, 12, 17>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvSimdScalarPairwise {
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b11011) => AArch64Inst::AddpScalar(data),
                    (0b0, 0b00 | 0b01, 0b01100) => AArch64Inst::FmaxnmpScalarEncoding(data),
                    (0b0, 0b00 | 0b01, 0b01101) => AArch64Inst::FaddpScalarEncoding(data),
                    (0b0, 0b00 | 0b01, 0b01111) => AArch64Inst::FmaxpScalarEncoding(data),
                    (0b0, 0b10 | 0b11, 0b01100) => AArch64Inst::FminnmpScalarEncoding(data),
                    (0b0, 0b10 | 0b11, 11) => AArch64Inst::FminpScalarEncoding(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_ld_st_single_structure(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_0011010_x_x_00000_xxx_x_xx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(r): Extract<u8, 21, 22>,
             Extract(opcode): Extract<u8, 13, 16>,
             Extract(s): Extract<u8, 12, 13>,
             Extract(size): Extract<u8, 10, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = AdvSimdLdStSingleStructure {
                    q,
                    s,
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt,
                };

                match (l, r, opcode, s, size) {
                    (0b0, 0b0, 0b000, _, _) => AArch64Inst::St1SingleStructureVar8(data),
                    (0b0, 0b0, 0b001, _, _) => AArch64Inst::St3SingleStructureVar8(data),
                    (0b0, 0b0, 0b010, _, 0b00 | 0b10) => AArch64Inst::St1SingleStructureVar16(data),
                    (0b0, 0b0, 0b011, _, 0b00 | 0b10) => AArch64Inst::St3SingleStructureVar16(data),

                    (0b0, 0b0, 0b100, _, 0b00) => AArch64Inst::St1SingleStructureVar32(data),
                    (0b0, 0b0, 0b100, 0b0, 0b01) => AArch64Inst::St1SingleStructureVar64(data),
                    (0b0, 0b0, 0b101, _, 0b00) => AArch64Inst::St3SingleStructureVar32(data),
                    (0b0, 0b0, 0b101, 0b0, 0b01) => AArch64Inst::St3SingleStructureVar64(data),

                    (0b0, 0b1, 0b000, _, _) => AArch64Inst::St2SingleStructureVar8(data),
                    (0b0, 0b1, 0b001, _, _) => AArch64Inst::St4SingleStructureVar8(data),
                    (0b0, 0b1, 0b010, _, 0b00 | 0b10) => AArch64Inst::St2SingleStructureVar16(data),

                    (0b0, 0b1, 0b011, _, 0b00 | 0b10) => AArch64Inst::St4SingleStructureVar16(data),

                    (0b0, 0b1, 0b100, _, 0b00) => AArch64Inst::St2SingleStructureVar32(data),
                    (0b0, 0b1, 0b100, 0b0, 0b01) => AArch64Inst::St2SingleStructureVar64(data),
                    (0b0, 0b1, 0b101, _, 0b00) => AArch64Inst::St4SingleStructureVar32(data),
                    (0b0, 0b1, 0b101, 0b0, 0b01) => AArch64Inst::St4SingleStructureVar64(data),

                    (0b1, 0b0, 0b000, _, _) => AArch64Inst::Ld1SingleStructureVar8(data),
                    (0b1, 0b0, 0b001, _, _) => AArch64Inst::Ld3SingleStructureVar8(data),
                    (0b1, 0b0, 0b010, _, 0b00 | 0b10) => AArch64Inst::Ld1SingleStructureVar16(data),

                    (0b1, 0b0, 0b011, _, 0b00 | 0b10) => AArch64Inst::Ld3SingleStructureVar16(data),

                    (0b1, 0b0, 0b100, _, 0b00) => AArch64Inst::Ld1SingleStructureVar32(data),
                    (0b1, 0b0, 0b100, 0b0, 0b01) => AArch64Inst::Ld1SingleStructureVar64(data),
                    (0b1, 0b0, 0b101, _, 0b00) => AArch64Inst::Ld3SingleStructureVar32(data),
                    (0b1, 0b0, 0b101, 0b0, 0b01) => AArch64Inst::Ld3SingleStructureVar64(data),

                    (0b1, 0b0, 0b110, 0b0, _) => AArch64Inst::Ld1r(data),
                    (0b1, 0b0, 0b111, 0b0, _) => AArch64Inst::Ld3r(data),

                    (0b1, 0b1, 0b000, _, _) => AArch64Inst::Ld2SingleStructureVar8(data),
                    (0b1, 0b1, 0b001, _, _) => AArch64Inst::Ld4SingleStructureVar8(data),
                    (0b1, 0b1, 0b010, _, 0b00 | 0b10) => AArch64Inst::Ld2SingleStructureVar16(data),

                    (0b1, 0b1, 0b011, _, 0b00 | 0b10) => AArch64Inst::Ld4SingleStructureVar16(data),

                    (0b1, 0b1, 0b100, _, 0b00) => AArch64Inst::Ld2SingleStructureVar32(data),
                    (0b1, 0b1, 0b100, 0b0, 0b01) => AArch64Inst::Ld2SingleStructureVar64(data),
                    (0b1, 0b1, 0b101, _, 0b00) => AArch64Inst::Ld4SingleStructureVar32(data),
                    (0b1, 0b1, 0b101, 0b0, 0b01) => AArch64Inst::Ld4SingleStructureVar64(data),

                    (0b1, 0b1, 0b110, 0b0, _) => AArch64Inst::Ld2r(data),
                    (0b1, 0b1, 0b111, 0b0, _) => AArch64Inst::Ld2r(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_2reg_miscellaneous(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01110_xx_10000_xxxxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(opcode): Extract<u8, 12, 17>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = QSizeRnRd {
                    q,
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b00000) => AArch64Inst::Rev64(data),
                    (0b0, _, 0b00001) => AArch64Inst::Rev16Vec(data),
                    (0b0, _, 0b00010) => AArch64Inst::Saddlp(data),
                    (0b0, _, 0b00011) => AArch64Inst::Suqadd(data),
                    (0b0, _, 0b00100) => AArch64Inst::ClsVec(data),
                    (0b0, _, 0b00101) => AArch64Inst::Cnt(data),
                    (0b0, _, 0b00110) => AArch64Inst::Sadalp(data),
                    (0b0, _, 0b00111) => AArch64Inst::Sqabs(data),
                    (0b0, _, 0b01000) => AArch64Inst::CmgtZero(data),
                    (0b0, _, 0b01001) => AArch64Inst::CmeqZero(data),
                    (0b0, _, 0b01010) => AArch64Inst::CmltZero(data),
                    (0b0, _, 0b01011) => AArch64Inst::Abs(data),
                    (0b0, _, 0b10010) => AArch64Inst::XtnXtn2(data),
                    (0b0, _, 0b10100) => AArch64Inst::Sqxtn(data),

                    (0b0, 0b00 | 0b01, 0b10110) => AArch64Inst::Fcvtn(data),
                    (0b0, 0b00 | 0b01, 0b10111) => AArch64Inst::Fcvtl(data),
                    (0b0, 0b00 | 0b01, 0b11000) => AArch64Inst::FrintnVec(data),
                    (0b0, 0b00 | 0b01, 0b11001) => AArch64Inst::FrintmVec(data),
                    (0b0, 0b00 | 0b01, 0b11010) => AArch64Inst::FcvtnsVec(data),
                    (0b0, 0b00 | 0b01, 0b11011) => AArch64Inst::FcvtmsVec(data),
                    (0b0, 0b00 | 0b01, 0b11100) => AArch64Inst::FcvtasVec(data),
                    (0b0, 0b00 | 0b01, 0b11101) => AArch64Inst::ScvtfVecInt(data),

                    (0b0, 0b10 | 0b11, 0b01100) => AArch64Inst::FcmgtZero(data),
                    (0b0, 0b10 | 0b11, 0b01101) => AArch64Inst::FcmeqZero(data),
                    (0b0, 0b10 | 0b11, 0b01110) => AArch64Inst::FcmltZero(data),
                    (0b0, 0b10 | 0b11, 0b01111) => AArch64Inst::FabsVec(data),
                    (0b0, 0b10 | 0b11, 0b11000) => AArch64Inst::FrintpVec(data),
                    (0b0, 0b10 | 0b11, 0b11001) => AArch64Inst::FrintzVec(data),
                    (0b0, 0b10 | 0b11, 0b11010) => AArch64Inst::FcvtpsVec(data),
                    (0b0, 0b10 | 0b11, 0b11011) => AArch64Inst::FcvtzsVecInt(data),
                    (0b0, 0b10 | 0b11, 0b11100) => AArch64Inst::Urecpe(data),
                    (0b0, 0b10 | 0b11, 0b11101) => AArch64Inst::Frecpe(data),

                    (0b1, _, 0b00000) => AArch64Inst::Rev32Vec(data),
                    (0b1, _, 0b00010) => AArch64Inst::Uaddlp(data),
                    (0b1, _, 0b00011) => AArch64Inst::Usqadd(data),
                    (0b1, _, 0b00100) => AArch64Inst::ClzVec(data),
                    (0b1, _, 0b00110) => AArch64Inst::Uadalp(data),
                    (0b1, _, 0b00111) => AArch64Inst::Sqneg(data),
                    (0b1, _, 0b01000) => AArch64Inst::CmgeZero(data),
                    (0b1, _, 0b01001) => AArch64Inst::CmleZero(data),
                    (0b1, _, 0b01011) => AArch64Inst::NegVec(data),
                    (0b1, _, 0b10010) => AArch64Inst::Sqxtun(data),
                    (0b1, _, 0b10011) => AArch64Inst::Shll(data),
                    (0b1, _, 0b10100) => AArch64Inst::Uqxtn(data),

                    (0b1, 0b00 | 0b01, 0b10110) => AArch64Inst::Fcvtxn(data),
                    (0b1, 0b00 | 0b01, 0b11000) => AArch64Inst::FrintaVec(data),
                    (0b1, 0b00 | 0b01, 0b11001) => AArch64Inst::FrintxVec(data),
                    (0b1, 0b00 | 0b01, 0b11010) => AArch64Inst::FcvtnuVec(data),
                    (0b1, 0b00 | 0b01, 0b11011) => AArch64Inst::FcvtmuVec(data),
                    (0b1, 0b00 | 0b01, 0b11100) => AArch64Inst::FcvtauVec(data),
                    (0b1, 0b00 | 0b01, 0b11101) => AArch64Inst::UcvtfVecInt(data),

                    (0b1, 0b00, 0b00101) => AArch64Inst::Not(data),
                    (0b1, 0b01, 0b00101) => AArch64Inst::RbitVec(data),

                    (0b1, 0b10 | 0b11, 0b01100) => AArch64Inst::FcmgeZero(data),
                    (0b1, 0b10 | 0b11, 0b01101) => AArch64Inst::FcmleZero(data),
                    (0b1, 0b10 | 0b11, 0b01111) => AArch64Inst::FnegVec(data),
                    (0b1, 0b10 | 0b11, 0b11001) => AArch64Inst::FrintiVec(data),
                    (0b1, 0b10 | 0b11, 0b11010) => AArch64Inst::FcvtpuVec(data),
                    (0b1, 0b10 | 0b11, 0b11011) => AArch64Inst::FcvtzuVecInt(data),
                    (0b1, 0b10 | 0b11, 0b11100) => AArch64Inst::Ursqrte(data),
                    (0b1, 0b10 | 0b11, 0b11101) => AArch64Inst::Frsqrte(data),
                    (0b1, 0b10 | 0b11, 0b11111) => AArch64Inst::FsqrtVec(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_across_lanes(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01110_xx_11000_xxxxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(opcode): Extract<u8, 12, 17>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = QSizeRnRd {
                    q,
                    size,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b00011) => AArch64Inst::Saddlv(data),
                    (0b0, _, 0b01010) => AArch64Inst::Smaxv(data),
                    (0b0, _, 0b11010) => AArch64Inst::Sminv(data),
                    (0b0, _, 0b11011) => AArch64Inst::Addv(data),

                    (0b1, _, 0b00011) => AArch64Inst::Uaddlv(data),
                    (0b1, _, 0b01010) => AArch64Inst::Uaddlv(data),
                    (0b1, _, 0b11010) => AArch64Inst::Uminv(data),

                    (0b1, 0b00 | 0b01, 0b01100) => AArch64Inst::FmaxnvmEncoding(data),
                    (0b1, 0b00 | 0b01, 0b01111) => AArch64Inst::FmaxvEncoding(data),

                    (0b1, 0b10 | 0b11, 0b01100) => AArch64Inst::FminnmvEncoding(data),
                    (0b1, 0b10 | 0b11, 0b01111) => AArch64Inst::FminvEncoding(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_compare_and_swap(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_0010001_x_1_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = RsRnRt {
                    rs: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rs),
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, l, o0, rt2) {
                    (0b00, 0b0, 0b0, 0b11111) => AArch64Inst::Casb(data),
                    (0b00, 0b0, 0b1, 0b11111) => AArch64Inst::Caslb(data),
                    (0b00, 0b1, 0b0, 0b11111) => AArch64Inst::Casab(data),
                    (0b00, 0b1, 0b1, 0b11111) => AArch64Inst::Casalb(data),

                    (0b01, 0b0, 0b0, 0b11111) => AArch64Inst::Cash(data),
                    (0b01, 0b0, 0b1, 0b11111) => AArch64Inst::Caslh(data),
                    (0b01, 0b1, 0b0, 0b11111) => AArch64Inst::Casah(data),
                    (0b01, 0b1, 0b1, 0b11111) => AArch64Inst::Casalh(data),

                    (0b10, 0b0, 0b0, 0b11111) => AArch64Inst::CasVar32(data),
                    (0b10, 0b0, 0b1, 0b11111) => AArch64Inst::CaslVar32(data),
                    (0b10, 0b1, 0b0, 0b11111) => AArch64Inst::CasaVar32(data),
                    (0b10, 0b1, 0b1, 0b11111) => AArch64Inst::CasalVar32(data),

                    (0b11, 0b0, 0b0, 0b11111) => AArch64Inst::CasVar64(data),
                    (0b11, 0b0, 0b1, 0b11111) => AArch64Inst::CaslVar64(data),
                    (0b11, 0b1, 0b0, 0b11111) => AArch64Inst::CasaVar64(data),
                    (0b11, 0b1, 0b1, 0b11111) => AArch64Inst::CasalVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_atomic_memory_operations(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_x_x_1_xxxxx_x_xxx_00_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(a): Extract<u8, 23, 24>,
             Extract(r): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o3): Extract<u8, 15, 16>,
             Extract(opc): Extract<u8, 12, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = RsRnRt {
                    rs: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rs),
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, a, r, rs, o3, opc) {
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpbVar(data),

                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclrlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeorlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Inst::SwplbVar(data),

                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpabVar(data),

                    (0b00, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Inst::Ldaprb(data),

                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclralbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeoralbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Inst::SwpalbVar(data),

                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsethVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Inst::SwphVar(data),

                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclrlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeorlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Inst::SwplhVar(data),

                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpahVar(data),

                    (0b01, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Inst::Ldaprh(data),

                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclralhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeoralhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Inst::SwpalhVar(data),

                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpVar32(data),

                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclrlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeorlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Inst::SwplVar32(data),

                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclraVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeoraVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Inst::LdaprVar32(data),

                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclralVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeoralVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Inst::SwpalVar32(data),

                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclrVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeorVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpVar64(data),

                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b010) => AArch64Inst::St64bv0(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b011) => AArch64Inst::St64bv(data),
                    (0b11, 0b0, 0b0, 0b0, 0b11111, 0b1, 0b001) => AArch64Inst::St64b(data),
                    (0b11, 0b0, 0b0, 0b0, 0b11111, 0b1, 0b101) => AArch64Inst::Ld64b(data),

                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclrlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeorlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Inst::SwplVar64(data),

                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Inst::LdaddaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Inst::LdclraVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Inst::LdeoraVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Inst::LdsetaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Inst::LdsmaxaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Inst::LdsminaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Inst::LdumaxaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Inst::LduminaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Inst::SwpaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Inst::LdaprVar64(data),

                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Inst::LdaddalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Inst::LdclralVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Inst::LdeoralVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Inst::LdsetalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Inst::LdsmaxalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Inst::LdsminalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Inst::LdumaxalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Inst::LduminalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Inst::SwpalVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_add_sub_with_carry(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010000_xxxxx_000000_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RmRnRd {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rd),
                };

                match sf_op_s {
                    0b000 => AArch64Inst::AdcVar32(data),
                    0b001 => AArch64Inst::AdcsVar32(data),
                    0b010 => AArch64Inst::SbcVar32(data),
                    0b011 => AArch64Inst::SbcsVar32(data),

                    0b100 => AArch64Inst::AdcVar64(data),
                    0b101 => AArch64Inst::AdcsVar64(data),
                    0b110 => AArch64Inst::SbcVar64(data),
                    0b111 => AArch64Inst::SbcsVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_floating_point_compare(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xxxxx_xx_1000_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(op): Extract<u8, 14, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(opcode2): Extract<u8, 0, 5>| {
                let data = FloatingPointCompare {
                    ptype,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    opcode2,
                };

                match (m, s, ptype, op, opcode2) {
                    (0b0, 0b0, 0b00, 0b00, 0b00000 | 0b01000)
                    | (0b0, 0b0, 0b01, 0b00, 0b00000 | 0b01000)
                    | (0b0, 0b0, 0b11, 0b01, 0b00000 | 0b01000) => AArch64Inst::Fcmp(data),

                    (0b0, 0b0, 0b00, 0b00, 0b10000 | 0b11000)
                    | (0b0, 0b0, 0b01, 0b00, 0b10000 | 0b11000)
                    | (0b0, 0b0, 0b11, 0b01, 0b10000 | 0b11000) => AArch64Inst::Fcmp(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_advanced_simd_permute(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_001110_xx_0_xxxxx_0_xxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 30, 31>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 12, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = QSizeRmRnRd {
                    q,
                    size,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match opcode {
                    0b001 => AArch64Inst::Uzp1(data),
                    0b010 => AArch64Inst::Trn1(data),
                    0b011 => AArch64Inst::Zip1(data),

                    0b101 => AArch64Inst::Uzp2(data),
                    0b110 => AArch64Inst::Trn2(data),
                    0b111 => AArch64Inst::Zip2(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_float_data_proc_2src(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xxxxx_xxxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(opcode): Extract<u8, 12, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RmRnRd {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (m, s, ptype, opcode) {
                    (0b0, 0b0, 0b00, 0b0000) => AArch64Inst::FmulScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0001) => AArch64Inst::FdivScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0010) => AArch64Inst::FaddScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0011) => AArch64Inst::FsubScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0100) => AArch64Inst::FmaxScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0101) => AArch64Inst::FminScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0110) => AArch64Inst::FmaxnmScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0111) => AArch64Inst::FminnmScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b1000) => AArch64Inst::FnmulScalarSinglePrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b0000) => AArch64Inst::FmulScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0001) => AArch64Inst::FdivScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0010) => AArch64Inst::FaddScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0011) => AArch64Inst::FsubScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0100) => AArch64Inst::FmaxScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0101) => AArch64Inst::FminScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0110) => AArch64Inst::FmaxnmScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0111) => AArch64Inst::FminnmScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b1000) => AArch64Inst::FnmulScalarDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_floating_point_immediate(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xxxxxxxx_100_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(imm8): Extract<u8, 13, 21>,
             Extract(imm5): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = FloatingPointImmediate {
                    imm8,
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (m, s, ptype, imm5) {
                    (0b0, 0b0, 0b00, 0b00000) => AArch64Inst::FmovScalarImmSinglePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b00000) => AArch64Inst::FmovScalarImmDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_conv_between_float_and_fixed_point(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_0_xx_xxx_xxxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sf): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(rmode): Extract<u8, 19, 21>,
             Extract(opcode): Extract<u8, 16, 19>,
             Extract(scale): Extract<u8, 10, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = ConvBetweenFloatAndFixedPoint {
                    scale,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (sf, s, ptype, rmode, opcode, scale) {
                    (0b0, 0b0, 0b00, 0b00, 0b010, _) => {
                        AArch64Inst::ScvtfScalarFixedPt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b011, _) => {
                        AArch64Inst::UcvtfScalarFixedPt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b000, _) => {
                        AArch64Inst::FcvtzsScalarFixedPtSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b001, _) => {
                        AArch64Inst::FcvtzuScalarFixedPtSinglePrecisionTo32(data)
                    }

                    (0b0, 0b0, 0b01, 0b00, 0b010, _) => {
                        AArch64Inst::ScvtfScalarFixedPt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b011, _) => {
                        AArch64Inst::UcvtfScalarFixedPt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b000, _) => {
                        AArch64Inst::FcvtzsScalarFixedPtDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b001, _) => {
                        AArch64Inst::FcvtzuScalarFixedPtDoublePrecisionTo32(data)
                    }

                    (0b1, 0b0, 0b00, 0b00, 0b010, _) => {
                        AArch64Inst::ScvtfScalarFixedPt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b011, _) => {
                        AArch64Inst::UcvtfScalarFixedPt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b000, _) => {
                        AArch64Inst::FcvtzsScalarFixedPtSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b001, _) => {
                        AArch64Inst::FcvtzuScalarFixedPtSinglePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b01, 0b00, 0b010, _) => {
                        AArch64Inst::ScvtfScalarFixedPt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b011, _) => {
                        AArch64Inst::UcvtfScalarFixedPt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b000, _) => {
                        AArch64Inst::FcvtzsScalarFixedPtDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b001, _) => {
                        AArch64Inst::FcvtzuScalarFixedPtDoublePrecisionTo64(data)
                    }

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_floating_point_conditional_select(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_0_x_11110_xx_1_xxxxx_xxxx_11_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(m): Extract<u8, 31, 32>,
             Extract(s): Extract<u8, 29, 30>,
             Extract(ptype): Extract<u8, 22, 24>,
             Extract(rm): Extract<u8, 16, 21>,
             Extract(cond): Extract<u8, 12, 16>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = RmCondRnRd {
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    cond,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (m, s, ptype) {
                    (0b0, 0b0, 0b00) => AArch64Inst::FcselSinglePrecisionVar(data),
                    (0b0, 0b0, 0b01) => AArch64Inst::FcselDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_vec_x_indexed_elem(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_x_01111_xx_x_x_xxxx_xxxx_x_0_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(q): Extract<u8, 31, 32>,
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(l): Extract<u8, 21, 22>,
             Extract(m): Extract<u8, 20, 21>,
             Extract(rm): Extract<u8, 16, 20>,
             Extract(opcode): Extract<u8, 12, 16>,
             Extract(h): Extract<u8, 11, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvSimdXIndexedElem {
                    q,
                    size,
                    l,
                    m,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    h,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b0010) => AArch64Inst::SmlalByElem(data),
                    (0b0, _, 0b0011) => AArch64Inst::SqdmlalByElem(data),
                    (0b0, _, 0b0110) => AArch64Inst::SmlslByElem(data),
                    (0b0, _, 0b0111) => AArch64Inst::SqdmlslByElem(data),
                    (0b0, _, 0b1000) => AArch64Inst::MulByElem(data),
                    (0b0, _, 0b1010) => AArch64Inst::SmullByElem(data),
                    (0b0, _, 0b1011) => AArch64Inst::SqdmullByElem(data),
                    (0b0, _, 0b1100) => AArch64Inst::SqdmulhByElem(data),
                    (0b0, _, 0b1101) => AArch64Inst::SqrdmulhByElem(data),

                    (0b0, 0b10 | 0b11, 0b0001) => AArch64Inst::FmlaByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b0101) => AArch64Inst::FmlsByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b1001) => AArch64Inst::FmulByElemEncoding(data),

                    (0b1, _, 0b0000) => AArch64Inst::MlaByElem(data),
                    (0b1, _, 0b0010) => AArch64Inst::UmlalByElem(data),
                    (0b1, _, 0b0100) => AArch64Inst::MlsByElem(data),
                    (0b1, _, 0b0110) => AArch64Inst::UmlslByElem(data),
                    (0b1, _, 0b1010) => AArch64Inst::UmullByElem(data),

                    (0b1, 0b10 | 0b11, 0b1001) => AArch64Inst::FmulxByElemEncoding(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_adv_simd_scalar_x_indexed_elem(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("01_x_11111_xx_x_x_xxxx_xxxx_x_0_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(u): Extract<u8, 29, 30>,
             Extract(size): Extract<u8, 22, 24>,
             Extract(l): Extract<u8, 21, 22>,
             Extract(m): Extract<u8, 20, 21>,
             Extract(rm): Extract<u8, 16, 20>,
             Extract(opcode): Extract<u8, 12, 16>,
             Extract(h): Extract<u8, 11, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rd): Extract<u8, 0, 5>| {
                let data = AdvSimdXIndexedElem {
                    q: 0b1,
                    size,
                    l,
                    m,
                    rm: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rm),
                    h,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rn),
                    rd: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::V, rd),
                };

                match (u, size, opcode) {
                    (0b0, _, 0b0011) => AArch64Inst::SqdmlalByElem(data),
                    (0b0, _, 0b0111) => AArch64Inst::SqdmlslByElem(data),
                    (0b0, _, 0b1011) => AArch64Inst::SqdmullByElem(data),
                    (0b0, _, 0b1100) => AArch64Inst::SqdmulhByElem(data),
                    (0b0, _, 0b1101) => AArch64Inst::SqrdmulhByElem(data),

                    (0b0, 0b10 | 0b11, 0b0001) => AArch64Inst::FmlaByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b0101) => AArch64Inst::FmlsByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b1001) => AArch64Inst::FmulByElemEncoding(data),

                    (0b1, 0b10 | 0b11, 0b1001) => AArch64Inst::FmulxByElemEncoding(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_sys_instr_with_reg_arg(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("11010101000000110001_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Rt {
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (crm, op2) {
                    (0b0000, 0b000) => AArch64Inst::Wfet(data),
                    (0b0000, 0b001) => AArch64Inst::Wfit(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_pstate(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1101010100000_xxx_0100_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(op1): Extract<u8, 16, 19>,
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = PstateOp { op1, crm, op2 };

                match (op1, op2, rt) {
                    (0b000, 0b000, 0b11111) => AArch64Inst::Cfinv(data),
                    (0b000, 0b001, 0b11111) => AArch64Inst::Xaflag(data),
                    (0b000, 0b010, 0b11111) => AArch64Inst::Axflag(data),
                    (_, _, 0b11111) => AArch64Inst::MsrImm(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_sys_with_result(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1101010100000_xxx_0100_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(op1): Extract<u8, 16, 19>,
             Extract(crn): Extract<u8, 12, 16>,
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Rt {
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (op1, crn, crm, op2) {
                    (0b011, 0b0011, 0b0000, 0b011) => AArch64Inst::Tstart(data),
                    (0b011, 0b0011, 0b0001, 0b011) => AArch64Inst::Ttest(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_sys_instr(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1101010100000_xxx_0100_xxxx_xxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(l): Extract<u8, 21, 22>,
             Extract(op1): Extract<u8, 16, 19>,
             Extract(crn): Extract<u8, 12, 16>,
             Extract(crm): Extract<u8, 8, 12>,
             Extract(op2): Extract<u8, 5, 8>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = SystemInstructions {
                    op1,
                    crn,
                    crm,
                    op2,
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match l {
                    0b0 => AArch64Inst::Sys(data),
                    0b1 => AArch64Inst::Sysl(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_rot_right_into_flags(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010000_xxxxxx_00001_xxxxx_x_xxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(imm6): Extract<u8, 15, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(o2): Extract<u8, 4, 5>,
             Extract(mask): Extract<u8, 0, 4>| {
                let data = RotateRightIntoFlags {
                    imm6,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    mask,
                };

                match (sf_op_s, o2) {
                    (0b101, 0b0) => AArch64Inst::Rmif(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_eval_into_flags(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("x_x_x_11010000_xxxxxx_x_0010_xxxxx_x_xxxx"),
            |raw_instr: &[u8],
             Extract(sf_op_s): Extract<u8, 29, 32>,
             Extract(opcode2): Extract<u8, 15, 21>,
             Extract(sz): Extract<u8, 14, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(o3): Extract<u8, 4, 5>,
             Extract(mask): Extract<u8, 0, 4>| {
                let data = Rn {
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                };

                match (sf_op_s, opcode2, sz, o3, mask) {
                    (0b001, 0b000000, 0b0, 0b0, 0b1101) => AArch64Inst::SetfVar8(data),
                    (0b001, 0b000000, 0b1, 0b0, 0b1101) => AArch64Inst::SetfVar16(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_register_literal(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_011_x_00_xxxxxxxxxxxxxxxxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(imm19): Extract<u32, 5, 24>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Imm19Rt {
                    imm19,
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (opc, v) {
                    (0b00, 0b0) => AArch64Inst::LdrLitVar32(data),
                    (0b00, 0b1) => AArch64Inst::LdrLitSimdFPVar32(data),
                    (0b01, 0b0) => AArch64Inst::LdrLitVar64(data),
                    (0b01, 0b1) => AArch64Inst::LdrLitSimdFPVar64(data),
                    (0b10, 0b0) => AArch64Inst::LdrswLit(data),
                    (0b10, 0b1) => AArch64Inst::LdrLitSimdFPVar128(data),
                    (0b11, 0b0) => AArch64Inst::PrfmLit(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_compare_and_swap_pair(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("0_x_0010000_x_1_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sz): Extract<u8, 30, 31>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = CompareAndSwapPair {
                    rs,
                    rn: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rn),
                    rt,
                };

                match (sz, l, o0, rt2) {
                    (0b0, 0b0, 0b0, 0b11111) => AArch64Inst::CaspVar32(data),
                    (0b0, 0b0, 0b1, 0b11111) => AArch64Inst::CasplVar32(data),
                    (0b0, 0b1, 0b0, 0b11111) => AArch64Inst::CaspaVar32(data),
                    (0b0, 0b1, 0b1, 0b11111) => AArch64Inst::CaspalVar32(data),

                    (0b1, 0b0, 0b0, 0b11111) => AArch64Inst::CaspVar64(data),
                    (0b1, 0b0, 0b1, 0b11111) => AArch64Inst::CasplVar64(data),
                    (0b1, 0b1, 0b0, 0b11111) => AArch64Inst::CaspaVar64(data),
                    (0b1, 0b1, 0b1, 0b11111) => AArch64Inst::CaspalVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_memory_tags(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("11011001_xx_1_xxxxxxxxx_xx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm9): Extract<u16, 12, 221>,
             Extract(op2): Extract<u8, 10, 12>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreMemoryTags {
                    imm9,
                    op2,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rt,
                    ),
                };

                match (opc, imm9, op2) {
                    (0b00, _, 0b01 | 0b10 | 0b11) => AArch64Inst::StgEncoding(data),
                    (0b00, 0b000000000, 0b00) => AArch64Inst::Stzgm(data),
                    (0b01, _, 0b00) => AArch64Inst::Ldg(data),
                    (0b01, _, 0b01 | 0b10 | 0b11) => AArch64Inst::StzgEncoding(data),
                    (0b10, _, 0b01 | 0b10 | 0b11) => AArch64Inst::St2gEncoding(data),
                    (0b10, 0b000000000, 0b00) => AArch64Inst::Stgm(data),
                    (0b11, _, 0b01 | 0b10 | 0b11) => AArch64Inst::Stz2gEncoding(data),
                    (0b11, 0b000000000, 0b00) => AArch64Inst::Ldgm(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_exclusive_pair(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("1_x_0010000_x_1_xxxxx_x_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(sz): Extract<u8, 30, 31>,
             Extract(l): Extract<u8, 22, 23>,
             Extract(rs): Extract<u8, 16, 21>,
             Extract(o0): Extract<u8, 15, 16>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LoadStoreExclusivePair {
                    rs: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rs),
                    rt2: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt2),
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (sz, l, o0) {
                    (0b0, 0b0, 0b0) => AArch64Inst::StxpVar32(data),
                    (0b0, 0b0, 0b1) => AArch64Inst::StlxpVar32(data),
                    (0b0, 0b1, 0b0) => AArch64Inst::LdxpVar32(data),
                    (0b0, 0b1, 0b1) => AArch64Inst::LdaxpVar32(data),

                    (0b1, 0b0, 0b0) => AArch64Inst::StxpVar64(data),
                    (0b1, 0b0, 0b1) => AArch64Inst::StlxpVar64(data),
                    (0b1, 0b1, 0b0) => AArch64Inst::LdxpVar64(data),
                    (0b1, 0b1, 0b1) => AArch64Inst::LdaxpVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_ldapr_stlr_unscaled_imm(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_011001_xx_0_xxxxxxxxx_00_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm9): Extract<u16, 12, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Imm9RnRt {
                    imm9,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, opc) {
                    (0b00, 0b00) => AArch64Inst::Stlurb(data),
                    (0b00, 0b01) => AArch64Inst::Ldapurb(data),
                    (0b00, 0b10) => AArch64Inst::LdapursbVar64(data),
                    (0b00, 0b11) => AArch64Inst::LdapursbVar32(data),

                    (0b01, 0b00) => AArch64Inst::Stlurh(data),
                    (0b01, 0b01) => AArch64Inst::Ldapurh(data),
                    (0b01, 0b10) => AArch64Inst::LdapurshVar64(data),
                    (0b01, 0b11) => AArch64Inst::LdapurshVar32(data),

                    (0b10, 0b00) => AArch64Inst::StlurVar32(data),
                    (0b10, 0b01) => AArch64Inst::LdapurVar32(data),
                    (0b10, 0b10) => AArch64Inst::Ldapursw(data),

                    (0b11, 0b00) => AArch64Inst::StlurVar64(data),
                    (0b11, 0b01) => AArch64Inst::LdapurVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_ld_st_no_alloc_pair_offset(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_101_x_000_x_xxxxxxx_xxxxx_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(opc): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(l): Extract<u16, 22, 23>,
             Extract(imm7): Extract<u8, 15, 22>,
             Extract(rt2): Extract<u8, 10, 15>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = LdStNoAllocPairOffset {
                    imm7,
                    rt2: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt2),
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (opc, v, l) {
                    (0b00, 0b0, 0b0) => AArch64Inst::StnpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Inst::LdnpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Inst::StnpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Inst::LdnpSimdFPVar32(data),

                    (0b01, 0b1, 0b0) => AArch64Inst::StnpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Inst::LdnpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Inst::StnpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Inst::LdnpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Inst::StnpSimdFPVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Inst::LdnpSimdFPVar128(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}

fn parse_load_store_reg_unprivileged(raw_instr: &[u8]) -> Option<AArch64Inst> {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Inst>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            to_le("xx_111_x_00_xx_0_xxxxxxxxx_10_xxxxx_xxxxx"),
            |raw_instr: &[u8],
             Extract(size): Extract<u8, 30, 32>,
             Extract(v): Extract<u8, 26, 27>,
             Extract(opc): Extract<u8, 22, 24>,
             Extract(imm9): Extract<u16, 12, 21>,
             Extract(rn): Extract<u8, 5, 10>,
             Extract(rt): Extract<u8, 0, 5>| {
                let data = Imm9RnRt {
                    imm9,
                    rn: AArch64Architecture::get_register_by_mnemonic(
                        AArch64MnemonicHint::X_SP,
                        rn,
                    ),
                    rt: AArch64Architecture::get_register_by_mnemonic(AArch64MnemonicHint::X, rt),
                };

                match (size, v, opc) {
                    (0b00, 0b0, 0b00) => AArch64Inst::Sttrb(data),
                    (0b00, 0b0, 0b01) => AArch64Inst::Ldtrb(data),
                    (0b00, 0b0, 0b10) => AArch64Inst::LdtrsbVar64(data),
                    (0b00, 0b0, 0b11) => AArch64Inst::LdtrsbVar32(data),

                    (0b01, 0b0, 0b00) => AArch64Inst::Sttrh(data),
                    (0b01, 0b0, 0b01) => AArch64Inst::Ldtrh(data),
                    (0b01, 0b0, 0b10) => AArch64Inst::LdtrshVar64(data),
                    (0b01, 0b0, 0b11) => AArch64Inst::LdtrshVar32(data),

                    (0b10, 0b0, 0b00) => AArch64Inst::SttrVar32(data),
                    (0b10, 0b0, 0b01) => AArch64Inst::LdtrVar32(data),
                    (0b10, 0b0, 0b10) => AArch64Inst::Ldtrsw(data),

                    (0b11, 0b0, 0b00) => AArch64Inst::SttrVar64(data),
                    (0b11, 0b0, 0b01) => AArch64Inst::LdtrVar64(data),

                    _ => todo!("Unknown instruction {:?}", raw_instr),
                }
            },
        );

        m
    });

    MATCHER.try_match(raw_instr)
}
