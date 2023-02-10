use crate::aarch64::*;
use crate::bit_patterns::*;
use crate::instr::NativeInstr;
use crate::MachineInstrParserRule;

use utility::{extract_bits32, BitReader};

use once_cell::sync::Lazy;

/// AArch64 instruction parser
#[derive(Clone, Debug)]
pub struct AArch64InstrParserRule;

impl MachineInstrParserRule for AArch64InstrParserRule {
    type MachineInstr = AArch64Instr;

    fn parse<I>(&mut self, buf: &mut BitReader<I>) -> Option<NativeInstr<Self::MachineInstr>>
    where
        I: Iterator<Item = u8>,
    {
        // Todo features : FEAT_PAuth, FEAT_LSE
        parse_aarch64_instr(buf).map(|v| NativeInstr { op: v, size: 4 })
    }
}

fn parse_aarch64_instr<I>(reader: &mut BitReader<I>) -> Option<AArch64Instr>
where
    I: Iterator<Item = u8>,
{
    // AArch64 instruction has fixed length of 32 bits
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind("0_xx_0000_xxxxxxxxxxxxxxxxxxxxxxxxx", |raw_instr: u32| {
            let op0 = extract_bits32(29..31, raw_instr);
            let op1 = extract_bits32(16..25, raw_instr);
            let imm16 = Imm16 {
                imm16: extract_bits32(0..16, raw_instr) as u16,
            };

            match (op0, op1) {
                (0b00, 0b000000000) => AArch64Instr::Udf(imm16),
                _ => todo!("Unknown reserved instruction {:032b}", raw_instr),
            }
        })
        .bind("1_xx_0000_xxxxxxxxxxxxxxxxxxxxxxxxx", |_raw_instr: u32| {
            todo!("SME encodings")
        })
        .bind("x_xx_0010_xxxxxxxxxxxxxxxxxxxxxxxxx", |_raw_instr: u32| {
            todo!("SVE encodings")
        })
        .bind("x_xx_100x_xxxxxxxxxxxxxxxxxxxxxxxxx", parse_aarch64_d_p_i)
        .bind(
            "x_xx_101x_xxxxxxxxxxxxxxxxxxxxxxxxx",
            parse_aarch64_branches_exception_gen_and_sys_instr,
        )
        .bind(
            "x_xx_x1x0_xxxxxxxxxxxxxxxxxxxxxxxxx",
            parse_aarch64_load_and_stores,
        )
        .bind("x_xx_x101_xxxxxxxxxxxxxxxxxxxxxxxxx", parse_aarch64_d_p_r)
        .bind(
            "x_xx_x111_xxxxxxxxxxxxxxxxxxxxxxxxx",
            parse_aarch64_dp_sfp_adv_simd,
        );

        m
    });

    if let Some(raw_instr) = reader.read32() {
        if let Some(instr) = MATCHER.handle(raw_instr) {
            return Some(instr);
        } else {
            todo!("Unknown instruction {:032b}", raw_instr);
        }
    }

    None
}

// parse DPI(Data Processing Immediate) instructions in AArch64
fn parse_aarch64_d_p_i(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xxx_100_00x_xxxxxxxxxxxxxxxxxxxxxxx",
            parse_pc_rel_addressing,
        )
        .bind(
            "xxx_100_010_xxxxxxxxxxxxxxxxxxxxxxx",
            parse_add_sub_immediate,
        )
        .bind(
            "xxx_100_011_xxxxxxxxxxxxxxxxxxxxxxx",
            parse_add_sub_imm_with_tags,
        )
        .bind("xxx_100_100_xxxxxxxxxxxxxxxxxxxxxxx", parse_logical_imm)
        .bind("xxx_100_101_xxxxxxxxxxxxxxxxxxxxxxx", parse_move_wide_imm)
        .bind("xxx_100_110_xxxxxxxxxxxxxxxxxxxxxxx", parse_bitfield)
        .bind("xxx_100_111_xxxxxxxxxxxxxxxxxxxxxxx", parse_extract);

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

// parse DPI(Data Processing Register) instructions in AArch64
fn parse_aarch64_d_p_r(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_1_101_0110_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_data_proc_2src,
        )
        .bind(
            "x_1_x_1_101_0110_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_data_proc_1src,
        )
        .bind(
            "x_x_x_0_101_0xxx_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_logical_shifted_register,
        )
        .bind(
            "x_x_x_0_101_1xx0_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_add_sub_shifted_reg,
        )
        .bind(
            "x_x_x_0_101_1xx1_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_add_sub_ext_reg,
        )
        .bind(
            "x_x_x_1_101_0000_xxxxx_000000_xxxxxxxxxx",
            parse_add_sub_with_carry,
        )
        .bind(
            "x_x_x_1_101_0000_xxxxx_x00001_xxxxxxxxxx",
            parse_rot_right_into_flags,
        )
        .bind(
            "x_x_x_1_101_0000_xxxxx_xx0010_xxxxxxxxxx",
            parse_eval_into_flags,
        )
        .bind(
            "x_x_x_1_101_0010_xxxxx_xxxx0x_xxxxxxxxxx",
            parse_cond_cmp_reg,
        )
        .bind(
            "x_x_x_1_101_0010_xxxxx_xxxx1x_xxxxxxxxxx",
            parse_cond_cmp_imm,
        )
        .bind("x_x_x_1_101_0100_xxxxx_xxxxxx_xxxxxxxxxx", parse_cond_sel)
        .bind(
            "x_x_x_1_101_1xxx_xxxxx_xxxxxx_xxxxxxxxxx",
            parse_data_proccessing_3src,
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_aarch64_dp_sfp_adv_simd(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0100", "0x", "x101", "00xxxxx10"
            ),
            |_raw_instr: u32| todo!("Cryptographic AES"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0101", "0x", "x0xx", "xxx0xxx00"
            ),
            |_raw_instr: u32| todo!("Cryptographic three-register SHA"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0101", "0x", "x101", "00xxxxx10"
            ),
            |_raw_instr: u32| todo!("Cryptographic two-register SHA"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "00", "00xx", "xxx0xxxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar copy"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "10xx", "xxx00xxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar three same FP16"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "1111", "00xxxxx10"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar two-register miscellaneous FP16"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "x0xx", "xxx1xxxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar three same extra"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "x100", "00xxxxx10"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar two-register miscellaneous"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "x110", "00xxxxx10"
            ),
            parse_adv_simd_scalar_pairwise,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "x1xx", "xxxxxxx00"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar three different"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "0x", "x1xx", "xxxxxxxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar three same"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "10", "xxxx", "xxxxxxxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD scalar shifted by immediate"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "01x1", "1x", "xxxx", "xxxxxxxx0"
            ),
            parse_adv_simd_scalar_x_indexed_elem,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0x00", "0x", "x0xx", "xxx0xxx00"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD table lookup"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0x00", "0x", "x0xx", "xxx0xxx10"
            ),
            parse_advanced_simd_permute,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0x10", "0x", "x0xx", "xxx0xxxx0"
            ),
            parse_advanced_simd_extract,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "00", "00xx", "xxx0xxxx1"
            ),
            parse_advanced_simd_copy,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "10xx", "xxx00xxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD three same (FP16)"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "1111", "00xxxxx10"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD two-register miscellaneous (FP16)"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "x0xx", "xxx1xxxx1"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD three-register extension"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "x100", "00xxxxx10"
            ),
            parse_adv_simd_2reg_miscellaneous,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "x110", "00xxxxx10"
            ),
            parse_adv_simd_across_lanes,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "x1xx", "xxxxxxx00"
            ),
            |_raw_instr: u32| todo!("Advanced SIMD three different"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "0x", "x1xx", "xxxxxxxx1"
            ),
            parse_advanced_simd_three_same,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "10", "xxxx", "xxxxxxxx1"
            ),
            |raw_instr: u32, op2: Extract<BitRange<19, 23>, u8>| {
                if op2.value == 0b0000 {
                    parse_adv_simd_modified_imm(raw_instr)
                } else {
                    parse_adv_simd_shift_by_imm(raw_instr)
                }
            },
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "0xx0", "1x", "xxxx", "xxxxxxxx0"
            ),
            parse_adv_simd_vec_x_indexed_elem,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "1100", "00", "10xx", "xxx10xxxx"
            ),
            |_raw_instr: u32| todo!("Cryptographic three-register, imm2"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "1100", "00", "11xx", "xxx1x00xx"
            ),
            |_raw_instr: u32| todo!("Cryptographic three-reigster SHA 512"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "1100", "00", "xxxx", "xxx0xxxxx"
            ),
            |_raw_instr: u32| todo!("Cryptographic four-register"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "1100", "01", "00xx", "xxxxxxxxx"
            ),
            |_raw_instr: u32| todo!("XAR"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "1100", "01", "1000", "0001000xx"
            ),
            |_raw_instr: u32| todo!("Cryptographic two-register SHA 512"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x0xx", "xxxxxxxxx"
            ),
            parse_conv_between_float_and_fixed_point,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxx000000"
            ),
            parse_conv_between_float_and_int,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxx10000"
            ),
            parse_float_data_proc_1src,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxxx1000"
            ),
            parse_floating_point_compare,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxxxx100"
            ),
            parse_floating_point_immediate,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxxxxx01"
            ),
            |_raw_instr: u32| todo!("Floating-point conditional compare"),
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxxxxx10"
            ),
            parse_float_data_proc_2src,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "0x", "x1xx", "xxxxxxx11"
            ),
            parse_floating_point_conditional_select,
        )
        .bind(
            &format!(
                "{}_xxx_{}_{}_{}_xxxxxxxxxx",
                "x0x1", "1x", "xxxx", "xxxxxxxxx"
            ),
            parse_fp_data_processing_3src,
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

// parse Load and stores instructions i pairn AArch64
fn parse_aarch64_load_and_stores(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0x00_1_0_0_00_x_1xxxxx_xxxx_xx_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Compare and swap pair");
            },
        )
        .bind(
            "0x00_1_1_0_00_x_000000_xxxx_xx_xxxxxxxxxx",
            parse_adv_simd_ld_st_multi_structures,
        )
        .bind(
            "0x00_1_1_0_01_x_0xxxxx_xxxx_xx_xxxxxxxxxx",
            parse_adv_simd_ld_st_multi_structures_post_indexed,
        )
        .bind(
            "0x00_1_1_0_10_x_x00000_xxxx_xx_xxxxxxxxxx",
            parse_adv_simd_ld_st_single_structure,
        )
        .bind(
            "0x00_1_1_0_11_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Advanced SIMD Load/Store single structure(post-indexed)");
            },
        )
        .bind(
            "1101_1_0_0_1x_x_1xxxxx_xxxx_xx_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Load/store memory tags");
            },
        )
        .bind(
            "1x00_1_0_0_00_x_1xxxxx_xxxx_xx_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Load/store exclusive pair");
            },
        )
        .bind(
            "xx00_1_0_0_00_x_0xxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_exclusive_register,
        )
        .bind(
            "xx00_1_0_0_01_x_0xxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_ordered,
        )
        .bind(
            "xx00_1_0_0_01_x_1xxxxx_xxxx_xx_xxxxxxxxxx",
            parse_compare_and_swap,
        )
        .bind(
            "xx01_1_0_0_1x_x_0xxxxx_xxxx_00_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("LDAPR/STLR(unscaled immediate)");
            },
        )
        .bind(
            "xx01_1_x_0_0x_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_register_literal,
        )
        .bind(
            "xx01_1_x_0_1x_x_0xxxxx_xxxx_01_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Memory Copy and Memory Set");
            },
        )
        .bind(
            "xx10_1_x_0_00_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Load/Store no allocate pair (offset)");
            },
        )
        .bind(
            "xx10_1_x_0_01_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_reg_pair_post_indexed,
        )
        .bind(
            "xx10_1_x_0_10_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_reg_pair_offset,
        )
        .bind(
            "xx10_1_x_0_11_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_reg_pair_pre_indexed,
        )
        .bind(
            "xx11_1_x_0_0x_x_0xxxxx_xxxx_00_xxxxxxxxxx",
            parse_load_store_reg_unscaled_imm,
        )
        .bind(
            "xx11_1_x_0_0x_x_0xxxxx_xxxx_01_xxxxxxxxxx",
            parse_load_store_reg_imm_post_indexed,
        )
        .bind(
            "xx11_1_x_0_0x_x_0xxxxx_xxxx_10_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Load/Store register (unprevilaged)");
            },
        )
        .bind(
            "xx11_1_x_0_0x_x_0xxxxx_xxxx_11_xxxxxxxxxx",
            parse_load_store_reg_imm_pre_indexed,
        )
        .bind(
            "xx11_1_x_0_0x_x_1xxxxx_xxxx_00_xxxxxxxxxx",
            parse_atomic_memory_operations,
        )
        .bind(
            "xx11_1_x_0_0x_x_1xxxxx_xxxx_10_xxxxxxxxxx",
            parse_load_store_reg_reg_offset,
        )
        .bind(
            "xx11_1_x_0_0x_x_1xxxxx_xxxx_x1_xxxxxxxxxx",
            |_raw_instr: u32| {
                todo!("Load/Store register (pac)"); // Need to do FEAT_PAuth feature instructions
            },
        )
        .bind(
            "xx11_1_x_0_1x_x_xxxxxx_xxxx_xx_xxxxxxxxxx",
            parse_load_store_reg_unsigned_imm,
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_aarch64_branches_exception_gen_and_sys_instr(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        //--------------------------------------------
        //      |op1|101|      op2     |       | op3 |
        m.bind(
            "010_101_0xxxxxxxxxxxxx_xxxxxxx_xxxxx",
            parse_cond_branch_imm,
        )
        .bind("110_101_00xxxxxxxxxxxx_xxxxxxx_xxxxx", parse_exception_gen)
        .bind(
            "110_101_01000000110001_xxxxxxx_xxxxx",
            parse_sys_instr_with_reg_arg,
        )
        .bind("110_101_01000000110010_xxxxxxx_11111", parse_hints)
        .bind("110_101_01000000110011_xxxxxxx_xxxxx", parse_barriers)
        .bind("110_101_0100000xxx0100_xxxxxxx_xxxxx", parse_pstate)
        .bind(
            "110_101_0100100xxxxxxx_xxxxxxx_xxxxx",
            parse_sys_with_result,
        )
        .bind("110_101_0100x01xxxxxxx_xxxxxxx_xxxxx", parse_sys_instr)
        .bind("110_101_0100x1xxxxxxxx_xxxxxxx_xxxxx", parse_sys_reg_mov)
        .bind(
            "110_101_1xxxxxxxxxxxxx_xxxxxxx_xxxxx",
            parse_uncond_branch_reg,
        )
        .bind(
            "x00_101_xxxxxxxxxxxxxx_xxxxxxx_xxxxx",
            parse_uncond_branch_imm,
        )
        .bind(
            "x01_101_0xxxxxxxxxxxxx_xxxxxxx_xxxxx",
            parse_cmp_and_branch_imm,
        )
        .bind(
            "x01_101_1xxxxxxxxxxxxx_xxxxxxx_xxxxx",
            parse_test_and_branch_imm,
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_add_sub_shifted_reg(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xxx_01011_xx_0_xxxxxxxxxxxxxxxxxxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             shift: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             imm6: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = ShiftRmImm6RnRd {
                    shift: shift.value,
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                    imm6: imm6.value,
                };

                match (sf_op_s.value, shift.value, imm6.value) {
                    (0b000, _, _) => AArch64Instr::AddShiftedReg32(data),
                    (0b001, _, _) => AArch64Instr::AddsShiftedReg32(data),
                    (0b010, _, _) => AArch64Instr::SubShiftedReg32(data),
                    (0b011, _, _) => AArch64Instr::SubsShiftedReg32(data),
                    (0b100, _, _) => AArch64Instr::AddShiftedReg64(data),
                    (0b101, _, _) => AArch64Instr::AddsShiftedReg64(data),
                    (0b110, _, _) => AArch64Instr::SubShiftedReg64(data),
                    (0b111, _, _) => AArch64Instr::SubsShiftedReg64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_add_sub_immediate(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_100010_x_xxxxxxxxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             sh: Extract<BitRange<22, 23>, u8>,
             imm12: Extract<BitRange<10, 22>, u16>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = ShImm12RnRd {
                    sh: sh.value,
                    imm12: imm12.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match sf_op_s.value {
                    0b000 => AArch64Instr::AddImm32(data),
                    0b001 => AArch64Instr::AddsImm32(data),
                    0b010 => AArch64Instr::SubImm32(data),
                    0b011 => AArch64Instr::SubsImm32(data),
                    0b100 => AArch64Instr::AddImm64(data),
                    0b101 => AArch64Instr::AddsImm64(data),
                    0b110 => AArch64Instr::SubImm64(data),
                    0b111 => AArch64Instr::SubsImm64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_fp_data_processing_3src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11111_xx_x_xxxxx_x_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             o1: Extract<BitRange<21, 22>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             o0: Extract<BitRange<15, 16>, u8>,
             ra: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RmRaRnRd {
                    rm: rm.value,
                    ra: ra.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (m.value, s.value, ptype.value, o1.value, o0.value) {
                    (0b0, 0b0, 0b00, 0b0, 0b0) => AArch64Instr::FmAddSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b0, 0b1) => AArch64Instr::FmSubSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b1, 0b0) => AArch64Instr::FnmAddSinglePrecision(data),
                    (0b0, 0b0, 0b00, 0b1, 0b1) => AArch64Instr::FnmSubSinglePrecision(data),
                    (0b0, 0b0, 0b01, 0b0, 0b0) => AArch64Instr::FmAddDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b0, 0b1) => AArch64Instr::FmSubDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b1, 0b0) => AArch64Instr::FnmAddDoublePrecision(data),
                    (0b0, 0b0, 0b01, 0b1, 0b1) => AArch64Instr::FnmSubDoublePrecision(data),
                    (0b0, 0b0, 0b11, 0b0, 0b0) => AArch64Instr::FmAddHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b0, 0b1) => AArch64Instr::FmSubHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b1, 0b0) => AArch64Instr::FnmAddHalfPrecision(data),
                    (0b0, 0b0, 0b11, 0b1, 0b1) => AArch64Instr::FnmSubHalfPrecision(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_unsigned_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_01_xx_xxxxxxxxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             idxt: Extract<BitRange<24, 26>, u8>,
             opc: Extract<BitRange<22, 24>, u8>,
             imm12: Extract<BitRange<10, 22>, u16>,
             rm: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SizeImm12RnRt {
                    idxt: idxt.value,
                    size: size.value,
                    imm12: imm12.value,
                    rn: rm.value,
                    rt: rt.value,
                };

                match (size.value, v.value, opc.value) {
                    (0b00, 0b0, 0b00) => AArch64Instr::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Instr::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Instr::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Instr::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Instr::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Instr::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Instr::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Instr::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Instr::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Instr::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Instr::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Instr::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP16(data),
                    (0b10, 0b0, 0b00) => AArch64Instr::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Instr::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Instr::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Instr::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Instr::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Instr::LdrImm64(data),
                    (0b11, 0b0, 0b10) => AArch64Instr::PrfmImm(data),
                    (0b11, 0b1, 0b00) => AArch64Instr::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_move_wide_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_100101_xx_xxxxxxxxxxxxxxxx_xxxxx",
            |raw_instr: u32,
             sf_opc: Extract<BitRange<29, 32>, u8>,
             hw: Extract<BitRange<21, 23>, u8>,
             imm16: Extract<BitRange<5, 21>, u16>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = HwImm16Rd {
                    hw: hw.value,
                    imm16: imm16.value,
                    rd: rd.value,
                };

                match (sf_opc.value, hw.value) {
                    (0b000, 0b00 | 0b01) => AArch64Instr::MovnVar32(data),
                    (0b010, 0b00 | 0b01) => AArch64Instr::MovzVar32(data),
                    (0b011, 0b00 | 0b01) => AArch64Instr::MovkVar32(data),
                    (0b100, _) => AArch64Instr::MovnVar64(data),
                    (0b110, _) => AArch64Instr::MovzVar64(data),
                    (0b111, _) => AArch64Instr::MovkVar64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_uncond_branch_reg(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "1101011_xxxx_xxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             opc: Extract<BitRange<21, 25>, u8>,
             op2: Extract<BitRange<16, 21>, u8>,
             op3: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             op4: Extract<BitRange<0, 5>, u8>| {
                let z = extract_bits32(24..25, raw_instr);
                let op = extract_bits32(21..23, raw_instr);
                let a = extract_bits32(11..12, raw_instr);
                let rn = rn.value;
                let rm = op4.value;

                let data = UncondBranchReg {
                    z: z as u8,
                    op: op as u8,
                    a: a as u8,
                    rn,
                    rm,
                };

                match (opc.value, op2.value, op3.value, rn, rm) {
                    (0b0000, 0b11111, 0b000000, _, 0b00000) => AArch64Instr::Br(data),
                    (0b0000, 0b11111, 0b000010, _, 0b11111) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ. Key A, zero modifier")
                    }
                    (0b0000, 0b11111, 0b000011, _, 0b11111) => {
                        todo!("BRAA, BRAAZ, BRAB, BRABZ. Key B, zero modifier")
                    }
                    (0b0001, 0b11111, 0b000000, _, 0b00000) => AArch64Instr::Blr(data),
                    (0b0001, 0b11111, 0b000010, _, 0b11111) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ. Key A, zero modifier")
                    }
                    (0b0001, 0b11111, 0b000011, _, 0b11111) => {
                        todo!("BLRAA, BLRAAZ, BLRAB, BLRABZ. Key B, zero modifier")
                    }
                    (0b0010, 0b11111, 0b000000, _, 0b00000) => AArch64Instr::Ret(data),
                    (0b0010, 0b11111, 0b000010, 0b11111, 0b11111) => {
                        todo!("RETAA, RETAB - RETAA variant")
                    }
                    (0b0010, 0b11111, 0b000011, 0b11111, 0b11111) => {
                        todo!("RETAA, RETAB - RETAB variant")
                    }
                    (0b0100, 0b11111, 0b000000, 0b11111, 0b00000) => AArch64Instr::ERet(data),
                    (0b0100, 0b11111, 0b000010, 0b11111, 0b11111) => {
                        todo!("ERETAA, ERETAB - ERETAA variant")
                    }
                    (0b0100, 0b11111, 0b000011, 0b11111, 0b11111) => {
                        todo!("ERETAA, ERETAB - ERETAB variant")
                    }
                    (0b0101, 0b11111, 0b000000, 0b11111, 0b00000) => AArch64Instr::Drps(data),
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
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_uncond_branch_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_00101_xxxxxxxxxxxxxxxxxxxxxxxxxx",
            |raw_instr: u32,
             op: Extract<BitRange<31, 32>, u8>,
             imm26: Extract<BitRange<0, 26>, u32>| {
                let data = Imm26 { imm26: imm26.value };

                match op.value {
                    0b0 => AArch64Instr::BImm(data),
                    0b1 => AArch64Instr::BlImm(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_cond_branch_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0101010_x_xxxxxxxxxxxxxxxxxxx_x_xxxx",
            |raw_instr: u32,
             o1: Extract<BitRange<24, 25>, u8>,
             imm19: Extract<BitRange<5, 24>, u32>,
             o0: Extract<BitRange<4, 5>, u8>,
             cond: Extract<BitRange<0, 4>, u8>| {
                let data = Imm19Cond {
                    imm19: imm19.value,
                    cond: cond.value,
                };

                match (o1.value, o0.value) {
                    (0b0, 0b0) => AArch64Instr::BCond(data),
                    (0b0, 0b1) => AArch64Instr::BcCond(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_cond_sel(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010100_xxxxx_xxxx_xx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             cond: Extract<BitRange<12, 16>, u8>,
             op2: Extract<BitRange<10, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RmCondRnRd {
                    rm: rm.value,
                    cond: cond.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf_op_s.value, op2.value) {
                    (0b000, 0b00) => AArch64Instr::Csel32(data),
                    (0b000, 0b01) => AArch64Instr::Csel32(data),
                    (0b010, 0b00) => AArch64Instr::Csel32(data),
                    (0b010, 0b01) => AArch64Instr::Csel32(data),
                    (0b100, 0b00) => AArch64Instr::Csel32(data),
                    (0b100, 0b01) => AArch64Instr::Csel32(data),
                    (0b110, 0b00) => AArch64Instr::Csel32(data),
                    (0b110, 0b01) => AArch64Instr::Csel32(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_test_and_branch_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_011011_x_xxxxx_xxxxxxxxxxxxxx_xxxxx",
            |raw_instr: u32,
             b5: Extract<BitRange<31, 32>, u8>,
             op: Extract<BitRange<24, 25>, u8>,
             b40: Extract<BitRange<19, 24>, u8>,
             imm14: Extract<BitRange<5, 19>, u16>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = B5B40Imm14Rt {
                    b5: b5.value,
                    b40: b40.value,
                    imm14: imm14.value,
                    rt: rt.value,
                };

                match op.value {
                    0b0 => AArch64Instr::Tbz(data),
                    0b1 => AArch64Instr::Tbnz(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_logical_shifted_register(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_01010_xx_x_xxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             opc: Extract<BitRange<29, 31>, u8>,
             shift: Extract<BitRange<22, 24>, u8>,
             n: Extract<BitRange<21, 22>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             imm6: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = ShiftRmImm6RnRd {
                    shift: shift.value,
                    rm: rm.value,
                    imm6: imm6.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, opc.value, n.value) {
                    (0b0, _, _) if imm6.value & 0b100000 == 0b100000 => unreachable!(),
                    (0b0, 0b00, 0b0) => AArch64Instr::AndShiftedReg32(data),
                    (0b0, 0b00, 0b1) => AArch64Instr::BicShiftedReg32(data),
                    (0b0, 0b01, 0b0) => AArch64Instr::OrrShiftedReg32(data),
                    (0b0, 0b01, 0b1) => AArch64Instr::OrnShiftedReg32(data),
                    (0b0, 0b10, 0b0) => AArch64Instr::EorShiftedReg32(data),
                    (0b0, 0b10, 0b1) => AArch64Instr::EonShiftedReg32(data),
                    (0b0, 0b11, 0b0) => AArch64Instr::AndsShiftedReg32(data),
                    (0b0, 0b11, 0b1) => AArch64Instr::BicsShiftedReg32(data),
                    (0b1, 0b00, 0b0) => AArch64Instr::AndShiftedReg64(data),
                    (0b1, 0b00, 0b1) => AArch64Instr::BicShiftedReg64(data),
                    (0b1, 0b01, 0b0) => AArch64Instr::OrrShiftedReg64(data),
                    (0b1, 0b01, 0b1) => AArch64Instr::OrnShiftedReg64(data),
                    (0b1, 0b10, 0b0) => AArch64Instr::EorShiftedReg64(data),
                    (0b1, 0b10, 0b1) => AArch64Instr::EonShiftedReg64(data),
                    (0b1, 0b11, 0b0) => AArch64Instr::AndsShiftedReg64(data),
                    (0b1, 0b11, 0b1) => AArch64Instr::BicsShiftedReg64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_hints(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "11010101000000110010_xxxx_xxx_11111",
            |raw_instr: u32,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>| {
                match (crm.value, op2.value) {
                    (0b0000, 0b000) => AArch64Instr::Nop,
                    (0b0000, 0b001) => AArch64Instr::Yield,
                    (0b0000, 0b010) => AArch64Instr::Wfe,
                    (0b0000, 0b011) => AArch64Instr::Wfi,
                    (0b0000, 0b100) => AArch64Instr::Sev,
                    (0b0000, 0b101) => AArch64Instr::Sevl,

                    (0b0000, 0b111) => AArch64Instr::Xpaclri,
                    (0b0001, 0b000) => AArch64Instr::Pacia1716Var,
                    (0b0001, 0b010) => AArch64Instr::Pacib1716Var,
                    (0b0001, 0b100) => AArch64Instr::Autia1716Var,
                    (0b0001, 0b110) => AArch64Instr::Autib1716Var,

                    (0b0011, 0b000) => AArch64Instr::PaciazVar,
                    (0b0011, 0b001) => AArch64Instr::PaciaspVar,
                    (0b0011, 0b010) => AArch64Instr::PacibzVar,
                    (0b0011, 0b011) => AArch64Instr::PacibspVar,
                    (0b0011, 0b100) => AArch64Instr::AutiazVar,
                    (0b0011, 0b101) => AArch64Instr::AutiaspVar,
                    (0b0011, 0b110) => AArch64Instr::AutibzVar,
                    (0b0011, 0b111) => AArch64Instr::AutibspVar,
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_pc_rel_addressing(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_10000_xxxxxxxxxxxxxxxxxxx_xxxxx",
            |raw_instr: u32,
             op: Extract<BitRange<31, 32>, u8>,
             immlo: Extract<BitRange<29, 31>, u8>,
             immhi: Extract<BitRange<5, 24>, u32>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = PcRelAddressing {
                    immlo: immlo.value,
                    immhi: immhi.value,
                    rd: rd.value,
                };

                match op.value {
                    0b0 => AArch64Instr::Adr(data),
                    0b1 => AArch64Instr::Adrp(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_exception_gen(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "11010100_xxx_xxxxxxxxxxxxxxxx_xxx_xx",
            |raw_instr: u32,
             opc: Extract<BitRange<21, 24>, u8>,
             imm16: Extract<BitRange<5, 21>, u16>,
             op2: Extract<BitRange<2, 5>, u8>,
             ll: Extract<BitRange<0, 2>, u8>| {
                let data = ExceptionGen {
                    opc: opc.value,
                    imm16: imm16.value,
                    op2: op2.value,
                    ll: ll.value,
                };

                match (opc.value, op2.value, ll.value) {
                    (0b000, 0b000, 0b01) => AArch64Instr::Svc(data),
                    (0b000, 0b000, 0b10) => AArch64Instr::Hvc(data),
                    (0b000, 0b000, 0b11) => AArch64Instr::Smc(data),
                    (0b001, 0b000, 0b00) => AArch64Instr::Brk(data),
                    (0b010, 0b000, 0b00) => AArch64Instr::Hlt(data),
                    (0b011, 0b000, 0b00) => AArch64Instr::TCancle(data),
                    (0b101, 0b000, 0b01) => AArch64Instr::DcpS1(data),
                    (0b101, 0b000, 0b10) => AArch64Instr::DcpS2(data),
                    (0b101, 0b000, 0b11) => AArch64Instr::DcpS3(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_reg_offset(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_00_xx_1_xxxxx_xxx_x_10_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             opc: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             option: Extract<BitRange<13, 16>, u8>,
             s: Extract<BitRange<12, 13>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreRegRegOffset {
                    size: size.value,
                    v: v.value,
                    opc: opc.value,
                    rm: rm.value,
                    option: option.value,
                    s: s.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, v.value, opc.value, option.value) {
                    (0b00, 0b0, 0b00, _) if option.value != 0b011 => {
                        AArch64Instr::StrbRegExtReg(data)
                    }
                    (0b00, 0b0, 0b00, 0b011) => AArch64Instr::StrbRegShiftedReg(data),
                    (0b00, 0b0, 0b01, _) if option.value != 0b011 => {
                        AArch64Instr::LdrbRegExtReg(data)
                    }
                    (0b00, 0b0, 0b01, 0b011) => AArch64Instr::LdrbRegShiftedReg(data),
                    (0b00, 0b0, 0b10, _) if option.value != 0b011 => {
                        AArch64Instr::LdrsbRegExtReg64(data)
                    }
                    (0b00, 0b0, 0b10, 0b011) => AArch64Instr::LdrsbRegShiftedReg64(data),
                    (0b00, 0b0, 0b11, _) if option.value != 0b011 => {
                        AArch64Instr::LdrsbRegExtReg32(data)
                    }
                    (0b00, 0b0, 0b11, 0b011) => AArch64Instr::LdrsbRegShiftedReg32(data),
                    (_, 0b1, 0b00, _) | (0b00, 0b1, 0b10, _) => AArch64Instr::StrRegSimdFP(data),
                    (_, 0b1, 0b01, _) | (0b00, 0b1, 0b11, _) => AArch64Instr::LdrRegSimdFP(data),
                    (0b01, 0b0, 0b00, _) => AArch64Instr::StrhReg(data),
                    (0b01, 0b0, 0b01, _) => AArch64Instr::LdrhReg(data),
                    (0b01, 0b0, 0b10, _) => AArch64Instr::LdrshReg64(data),
                    (0b01, 0b0, 0b11, _) => AArch64Instr::LdrshReg32(data),
                    (0b10, 0b0, 0b00, _) => AArch64Instr::StrReg32(data),
                    (0b10, 0b0, 0b01, _) => AArch64Instr::LdrReg32(data),
                    (0b10, 0b0, 0b10, _) => AArch64Instr::LdrswReg(data),
                    (0b11, 0b0, 0b00, _) => AArch64Instr::StrReg64(data),
                    (0b11, 0b0, 0b01, _) => AArch64Instr::LdrReg64(data),
                    (0b11, 0b0, 0b10, _) => AArch64Instr::PrfmReg(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_add_sub_ext_reg(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_01011_xx_1_xxxxx_xxx_xxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             opt: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             option: Extract<BitRange<13, 16>, u8>,
             imm3: Extract<BitRange<10, 13>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AddSubtractExtReg {
                    rm: rm.value,
                    option: option.value,
                    imm3: imm3.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf_op_s.value, opt.value) {
                    (0b000, 0b00) => AArch64Instr::AddExtReg32(data),
                    (0b001, 0b00) => AArch64Instr::AddsExtReg32(data),
                    (0b010, 0b00) => AArch64Instr::SubExtReg32(data),
                    (0b011, 0b00) => AArch64Instr::SubsExtReg32(data),
                    (0b100, 0b00) => AArch64Instr::AddExtReg64(data),
                    (0b101, 0b00) => AArch64Instr::AddsExtReg64(data),
                    (0b110, 0b00) => AArch64Instr::SubExtReg64(data),
                    (0b111, 0b00) => AArch64Instr::SubsExtReg64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_bitfield(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_100110_x_xxxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             opc: Extract<BitRange<29, 31>, u8>,
             n: Extract<BitRange<22, 23>, u8>,
             immr: Extract<BitRange<16, 22>, u8>,
             imms: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = Bitfield {
                    n: n.value,
                    immr: immr.value,
                    imms: imms.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, opc.value, n.value) {
                    (0b0, 0b00, 0b0) => AArch64Instr::Sbfm32(data),
                    (0b0, 0b01, 0b0) => AArch64Instr::Bfm32(data),
                    (0b0, 0b10, 0b0) => AArch64Instr::Ubfm32(data),
                    (0b1, 0b00, 0b1) => AArch64Instr::Sbfm64(data),
                    (0b1, 0b01, 0b1) => AArch64Instr::Bfm64(data),
                    (0b1, 0b10, 0b1) => AArch64Instr::Ubfm64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_logical_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_100100_x_xxxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             opc: Extract<BitRange<29, 31>, u8>,
             n: Extract<BitRange<22, 23>, u8>,
             immr: Extract<BitRange<16, 22>, u8>,
             imms: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = LogicalImm {
                    n: n.value,
                    immr: immr.value,
                    imms: imms.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, opc.value, n.value) {
                    (0b0, 0b00, 0b0) => AArch64Instr::AndImm32(data),
                    (0b0, 0b01, 0b0) => AArch64Instr::OrrImm32(data),
                    (0b0, 0b10, 0b0) => AArch64Instr::EorImm32(data),
                    (0b0, 0b11, 0b0) => AArch64Instr::AndsImm32(data),
                    (0b1, 0b00, _) => AArch64Instr::AndImm64(data),
                    (0b1, 0b01, _) => AArch64Instr::OrrImm64(data),
                    (0b1, 0b10, _) => AArch64Instr::EorImm64(data),
                    (0b1, 0b11, _) => AArch64Instr::AndsImm64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_pair_offset(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_101_x_010_x_xxxxxxx_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             opc: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             imm7: Extract<BitRange<15, 22>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreRegPair {
                    imm7: imm7.value,
                    o: 0b010,
                    rt2: rt2.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (opc.value, v.value, l.value) {
                    (0b00, 0b0, 0b0) => AArch64Instr::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Instr::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Instr::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Instr::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Instr::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Instr::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Instr::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Instr::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Instr::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Instr::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_add_sub_imm_with_tags(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_100011_x_xxxxxx_xx_xxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             o2: Extract<BitRange<22, 23>, u8>,
             uimm6: Extract<BitRange<16, 22>, u8>,
             op3: Extract<BitRange<14, 16>, u8>,
             uimm4: Extract<BitRange<10, 14>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AddSubImmWithTags {
                    o2: o2.value,
                    uimm6: uimm6.value,
                    op3: op3.value,
                    uimm4: uimm4.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf_op_s.value, o2.value) {
                    (0b100, 0b0) => AArch64Instr::Addg(data),
                    (0b110, 0b0) => AArch64Instr::Subg(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_extract(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_100111_x_x_xxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op21: Extract<BitRange<29, 32>, u8>,
             n: Extract<BitRange<22, 23>, u8>,
             o0: Extract<BitRange<21, 22>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             imms: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = ExtractImm {
                    rm: rm.value,
                    imms: imms.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf_op21.value, n.value, o0.value, imms.value) {
                    (0b000, 0b0, 0b0, imms) if (imms & 0b100000) == 0b000000 => {
                        AArch64Instr::Extr32(data)
                    }
                    (0b100, 1, 0, _) => AArch64Instr::Extr64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_data_proc_1src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_1_x_11010110_xxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             opcode2: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RnRd {
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, s.value, opcode2.value, opcode.value) {
                    (0b0, 0b0, 0b00000, 0b000000) => AArch64Instr::RbitVar32(data),
                    (0b0, 0b0, 0b00000, 0b000001) => AArch64Instr::Rev16Var32(data),
                    (0b0, 0b0, 0b00000, 0b000010) => AArch64Instr::RevVar32(data),
                    (0b0, 0b0, 0b00000, 0b000100) => AArch64Instr::ClzVar32(data),
                    (0b0, 0b0, 0b00000, 0b000101) => AArch64Instr::ClsVar32(data),
                    (0b1, 0b0, 0b00000, 0b000000) => AArch64Instr::RbitVar64(data),
                    (0b1, 0b0, 0b00000, 0b000001) => AArch64Instr::Rev16Var64(data),
                    (0b1, 0b0, 0b00000, 0b000010) => AArch64Instr::Rev32(data),
                    (0b1, 0b0, 0b00000, 0b000011) => AArch64Instr::RevVar64(data),
                    (0b1, 0b0, 0b00000, 0b000100) => AArch64Instr::ClzVar64(data),
                    (0b1, 0b0, 0b00000, 0b000101) => AArch64Instr::ClsVar64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_cmp_and_branch_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_011010_x_xxxxxxxxxxxxxxxxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             op: Extract<BitRange<24, 25>, u8>,
             imm19: Extract<BitRange<5, 24>, u32>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Imm19Rt {
                    imm19: imm19.value,
                    rt: rt.value,
                };

                match (sf.value, op.value) {
                    (0b0, 0b0) => AArch64Instr::Cbz32(data),
                    (0b0, 0b1) => AArch64Instr::Cbnz32(data),
                    (0b1, 0b0) => AArch64Instr::Cbz64(data),
                    (0b1, 0b1) => AArch64Instr::Cbnz64(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_data_proccessing_3src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_xx_11011_xxx_xxxxx_x_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             op54: Extract<BitRange<29, 31>, u8>,
             op31: Extract<BitRange<21, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             o0: Extract<BitRange<15, 16>, u8>,
             ra: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = DataProc3Src {
                    rm: rm.value,
                    ra: ra.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, op54.value, op31.value, o0.value) {
                    (0b0, 0b00, 0b000, 0b0) => AArch64Instr::Madd32(data),
                    (0b0, 0b00, 0b000, 0b1) => AArch64Instr::Msub32(data),
                    (0b1, 0b00, 0b000, 0b0) => AArch64Instr::Madd64(data),
                    (0b1, 0b00, 0b000, 0b1) => AArch64Instr::Msub64(data),
                    (0b1, 0b00, 0b001, 0b0) => AArch64Instr::Smaddl(data),
                    (0b1, 0b00, 0b001, 0b1) => AArch64Instr::Smsubl(data),
                    (0b1, 0b00, 0b010, 0b0) => AArch64Instr::Smulh(data),
                    (0b1, 0b00, 0b101, 0b0) => AArch64Instr::Umaddl(data),
                    (0b1, 0b00, 0b101, 0b1) => AArch64Instr::Umsubl(data),
                    (0b1, 0b00, 0b110, 0b0) => AArch64Instr::Umulh(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_unscaled_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_00_xx_0_xxxxxxxxx_00_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             idxt: Extract<BitRange<24, 26>, u8>,
             opc: Extract<BitRange<22, 24>, u8>,
             imm9: Extract<BitRange<12, 21>, u16>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SizeImm12RnRt {
                    idxt: idxt.value,
                    size: size.value,
                    imm12: imm9.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, v.value, opc.value) {
                    (0b00, 0b0, 0b00) => AArch64Instr::Sturb(data),
                    (0b00, 0b0, 0b01) => AArch64Instr::Ldurb(data),
                    (0b00, 0b0, 0b10) => AArch64Instr::Ldursb64(data),
                    (0b00, 0b0, 0b11) => AArch64Instr::Ldursb32(data),
                    (0b00, 0b1, 0b00) => AArch64Instr::SturSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Instr::LdurSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Instr::SturSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Instr::LdurSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Instr::Sturh(data),
                    (0b01, 0b0, 0b01) => AArch64Instr::Ldurh(data),
                    (0b01, 0b0, 0b10) => AArch64Instr::Ldursh64(data),
                    (0b01, 0b0, 0b11) => AArch64Instr::Ldursh32(data),
                    (0b01, 0b1, 0b00) => AArch64Instr::SturSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Instr::LdurSimdFP16(data),
                    (0b10, 0b0, 0b00) => AArch64Instr::Stur32(data),
                    (0b10, 0b0, 0b01) => AArch64Instr::Ldur32(data),
                    (0b10, 0b0, 0b10) => AArch64Instr::Ldursw(data),
                    (0b10, 0b1, 0b00) => AArch64Instr::SturSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Instr::LdurSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Instr::Stur64(data),
                    (0b11, 0b0, 0b01) => AArch64Instr::Ldur64(data),
                    (0b11, 0b0, 0b10) => AArch64Instr::Prefum(data),
                    (0b11, 0b1, 0b00) => AArch64Instr::SturSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Instr::LdurSimdFP64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_sys_reg_mov(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "1101010100_x_1_x_xxx_xxxx_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             l: Extract<BitRange<21, 22>, u8>,
             o0: Extract<BitRange<19, 20>, u8>,
             op1: Extract<BitRange<16, 19>, u8>,
             crn: Extract<BitRange<12, 16>, u8>,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SysRegMov {
                    o0: o0.value,
                    op1: op1.value,
                    crn: crn.value,
                    crm: crm.value,
                    op2: op2.value,
                    rt: rt.value,
                };

                match l.value {
                    0 => AArch64Instr::MsrReg(data),
                    1 => AArch64Instr::Mrs(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_pair_pre_indexed(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_101_x_011_x_xxxxxxx_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             opc: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             imm7: Extract<BitRange<15, 22>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreRegPair {
                    o: 0b011,
                    imm7: imm7.value,
                    rt2: rt2.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (opc.value, v.value, l.value) {
                    (0b00, 0b0, 0b0) => AArch64Instr::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Instr::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Instr::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Instr::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Instr::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Instr::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Instr::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Instr::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Instr::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Instr::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_pair_post_indexed(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_101_x_001_x_xxxxxxx_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             opc: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             imm7: Extract<BitRange<15, 22>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreRegPair {
                    o: 0b001,
                    imm7: imm7.value,
                    rt2: rt2.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (opc.value, v.value, l.value) {
                    (0b00, 0b0, 0b0) => AArch64Instr::StpVar32(data),
                    (0b00, 0b0, 0b1) => AArch64Instr::LdpVar32(data),
                    (0b00, 0b1, 0b0) => AArch64Instr::StpSimdFPVar32(data),
                    (0b00, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar32(data),
                    (0b01, 0b0, 0b0) => AArch64Instr::Stgp(data),
                    (0b01, 0b0, 0b1) => AArch64Instr::Ldpsw(data),
                    (0b01, 0b1, 0b0) => AArch64Instr::StpSimdFPVar64(data),
                    (0b01, 0b1, 0b1) => AArch64Instr::LdpSimdFPVar64(data),
                    (0b10, 0b0, 0b0) => AArch64Instr::StpVar64(data),
                    (0b10, 0b0, 0b1) => AArch64Instr::LdpVar64(data),
                    (0b10, 0b1, 0b0) => AArch64Instr::StpSimdFpVar128(data),
                    (0b10, 0b1, 0b1) => AArch64Instr::LdpSimdFpVar128(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_data_proc_2src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11010110_xxxxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = DataProc2Src {
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, s.value, opcode.value) {
                    (0b0, 0b0, 0b000010) => AArch64Instr::UdivVar32(data),
                    (0b0, 0b0, 0b000011) => AArch64Instr::SdivVar32(data),
                    (0b0, 0b0, 0b001000) => AArch64Instr::LslvVar32(data),
                    (0b0, 0b0, 0b001001) => AArch64Instr::LsrvVar32(data),
                    (0b0, 0b0, 0b001010) => AArch64Instr::AsrvVar32(data),
                    (0b0, 0b0, 0b001011) => AArch64Instr::RorvVar32(data),
                    (0b1, 0b0, 0b000010) => AArch64Instr::UdivVar64(data),
                    (0b1, 0b0, 0b000011) => AArch64Instr::SdivVar64(data),
                    (0b1, 0b0, 0b001000) => AArch64Instr::LslvVar64(data),
                    (0b1, 0b0, 0b001001) => AArch64Instr::LsrvVar64(data),
                    (0b1, 0b0, 0b001010) => AArch64Instr::AsrvVar64(data),
                    (0b1, 0b0, 0b001011) => AArch64Instr::RorvVar64(data),

                    (0b1, 0b0, 0b001100) => AArch64Instr::Pacga(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_imm_pre_indexed(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_00_xx_0_xxxxxxxxx_11_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             idxt: Extract<BitRange<24, 26>, u8>, // Indexing type
             opc: Extract<BitRange<22, 24>, u8>,
             imm9: Extract<BitRange<12, 21>, u16>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SizeImm12RnRt {
                    idxt: idxt.value,
                    size: size.value,
                    imm12: imm9.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, v.value, opc.value) {
                    (0b00, 0b0, 0b00) => AArch64Instr::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Instr::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Instr::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Instr::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Instr::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Instr::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Instr::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Instr::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Instr::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Instr::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Instr::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Instr::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP16(data),

                    (0b10, 0b0, 0b00) => AArch64Instr::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Instr::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Instr::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Instr::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Instr::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Instr::LdrImm64(data),
                    (0b11, 0b1, 0b00) => AArch64Instr::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_reg_imm_post_indexed(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_00_xx_0_xxxxxxxxx_01_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             idxt: Extract<BitRange<24, 26>, u8>,
             opc: Extract<BitRange<22, 24>, u8>,
             imm9: Extract<BitRange<12, 21>, u16>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SizeImm12RnRt {
                    idxt: idxt.value,
                    size: size.value,
                    imm12: imm9.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, v.value, opc.value) {
                    (0b00, 0b0, 0b00) => AArch64Instr::StrbImm(data),
                    (0b00, 0b0, 0b01) => AArch64Instr::LdrbImm(data),
                    (0b00, 0b0, 0b10) => AArch64Instr::LdrsbImm64(data),
                    (0b00, 0b0, 0b11) => AArch64Instr::LdrsbImm32(data),
                    (0b00, 0b1, 0b00) => AArch64Instr::StrImmSimdFP8(data),
                    (0b00, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP8(data),
                    (0b00, 0b1, 0b10) => AArch64Instr::StrImmSimdFP128(data),
                    (0b00, 0b1, 0b11) => AArch64Instr::LdrImmSimdFP128(data),
                    (0b01, 0b0, 0b00) => AArch64Instr::StrhImm(data),
                    (0b01, 0b0, 0b01) => AArch64Instr::LdrhImm(data),
                    (0b01, 0b0, 0b10) => AArch64Instr::LdrshImm64(data),
                    (0b01, 0b0, 0b11) => AArch64Instr::LdrshImm32(data),
                    (0b01, 0b1, 0b00) => AArch64Instr::StrImmSimdFP16(data),
                    (0b01, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP16(data),

                    (0b10, 0b0, 0b00) => AArch64Instr::StrImm32(data),
                    (0b10, 0b0, 0b01) => AArch64Instr::LdrImm32(data),
                    (0b10, 0b0, 0b10) => AArch64Instr::LdrswImm(data),
                    (0b10, 0b1, 0b00) => AArch64Instr::StrImmSimdFP32(data),
                    (0b10, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP32(data),
                    (0b11, 0b0, 0b00) => AArch64Instr::StrImm64(data),
                    (0b11, 0b0, 0b01) => AArch64Instr::LdrImm64(data),
                    (0b11, 0b1, 0b00) => AArch64Instr::StrImmSimdFP64(data),
                    (0b11, 0b1, 0b01) => AArch64Instr::LdrImmSimdFP64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_barriers(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "11010101000000110011_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Barriers { crm: crm.value };

                match (crm.value, op2.value, rt.value) {
                    (_, 0b010, 0b11111) => AArch64Instr::Clrex(data),
                    (_, 0b100, 0b11111) => AArch64Instr::DsbEncoding(data),
                    (_, 0b101, 0b11111) => AArch64Instr::Dmb(data),
                    (_, 0b110, 0b11111) => AArch64Instr::Isb(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_advanced_simd_copy(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01110000_xxxxx_0_xxxx_1_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             op: Extract<BitRange<29, 30>, u8>,
             imm5: Extract<BitRange<16, 21>, u8>,
             imm4: Extract<BitRange<11, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvancedSimdCopy {
                    imm5: imm5.value,
                    imm4: imm4.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (q.value, op.value, imm5.value, imm4.value) {
                    (_, 0b0, _, 0b0000) => AArch64Instr::DupElement(data),
                    (_, 0b0, _, 0b0001) => AArch64Instr::DupGeneral(data),
                    (0b0 | 0b1, 0b0, _, 0b0101) => AArch64Instr::Smov(data),
                    (0b0, 0b0, _, 0b0111) => AArch64Instr::Umov(data),
                    (0b1, 0b0, 0b01000 | 0b11000, 0b0111) => AArch64Instr::Umov(data),
                    (0b1, 0b0, _, 0b0011) => AArch64Instr::InsGeneral(data),
                    (0b1, 0b1, _, _) => AArch64Instr::InsElement(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_cond_cmp_reg(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010010_xxxxx_xxxx_0_x_xxxxx_x_xxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             cond: Extract<BitRange<12, 16>, u8>,
             o2: Extract<BitRange<10, 11>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             o3: Extract<BitRange<4, 5>, u8>,
             nzcv: Extract<BitRange<0, 4>, u8>| {
                let data = CondCmpReg {
                    rm: rm.value,
                    cond: cond.value,
                    rn: rn.value,
                    nzcv: nzcv.value,
                };

                match (sf_op_s.value, o2.value, o3.value) {
                    (0b001, 0b0, 0b0) => AArch64Instr::CcmnRegVar32(data),
                    (0b011, 0b0, 0b0) => AArch64Instr::CcmpRegVar32(data),
                    (0b101, 0b0, 0b0) => AArch64Instr::CcmnRegVar64(data),
                    (0b111, 0b0, 0b0) => AArch64Instr::CcmpRegVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_ld_st_multi_structures(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_0011000_x_000000_xxxx_xx_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             opcode: Extract<BitRange<12, 16>, u8>,
             size: Extract<BitRange<10, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdLdStMultiStructures {
                    q: q.value,
                    size: size.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (l.value, opcode.value) {
                    (0b0, 0b0000) => AArch64Instr::St4MulStructures(data),
                    (0b0, 0b0010) => AArch64Instr::St1MulStructures4RegsVar(data),
                    (0b0, 0b0100) => AArch64Instr::St3MulStructures(data),
                    (0b0, 0b0110) => AArch64Instr::St1MulStructures3RegsVar(data),
                    (0b0, 0b0111) => AArch64Instr::St1MulStructures1RegsVar(data),
                    (0b0, 0b1000) => AArch64Instr::St2MulStructures(data),
                    (0b0, 0b1010) => AArch64Instr::St1MulStructures2RegsVar(data),

                    (0b1, 0b0000) => AArch64Instr::Ld4MulStructures(data),
                    (0b1, 0b0010) => AArch64Instr::Ld1MulStructures4RegsVar(data),
                    (0b1, 0b0100) => AArch64Instr::Ld3MulStructures(data),
                    (0b1, 0b0110) => AArch64Instr::Ld1MulStructures3RegsVar(data),
                    (0b1, 0b0111) => AArch64Instr::Ld1MulStructures1RegsVar(data),
                    (0b1, 0b1000) => AArch64Instr::Ld2MulStructures(data),
                    (0b1, 0b1010) => AArch64Instr::Ld1MulStructures2RegsVar(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_advanced_simd_extract(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_101110_xx_0_xxxxx_0_xxxx_0_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             op2: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             imm4: Extract<BitRange<11, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvancedSimdExtract {
                    q: q.value,
                    rm: rm.value,
                    imm4: imm4.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match op2.value {
                    0b00 => AArch64Instr::Ext(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_ld_st_multi_structures_post_indexed(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_0011001_x_0_xxxxx_xxxx_xx_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<12, 16>, u8>,
             size: Extract<BitRange<10, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdLdStMultiStructuresPostIndexed {
                    q: q.value,
                    rm: rm.value,
                    size: size.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (l.value, rm.value, opcode.value) {
                    (0b0, rm, 0b0000) if rm != 0b11111 => {
                        AArch64Instr::St4MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0010) if rm != 0b11111 => {
                        AArch64Instr::St1MulStructures4RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0100) if rm != 0b11111 => {
                        AArch64Instr::St3MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0110) if rm != 0b11111 => {
                        AArch64Instr::St1MulStructures3RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b0111) if rm != 0b11111 => {
                        AArch64Instr::St1MulStructures1RegRegOffsetVar(data)
                    }
                    (0b0, rm, 0b1000) if rm != 0b11111 => {
                        AArch64Instr::St2MulStructuresRegOffsetVar(data)
                    }
                    (0b0, rm, 0b1010) if rm != 0b11111 => {
                        AArch64Instr::St1MulStructures2RegRegOffsetVar(data)
                    }

                    (0b0, 0b11111, 0b0000) => AArch64Instr::St4MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b0010) => AArch64Instr::St1MulStructures4RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b0100) => AArch64Instr::St3MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b0110) => AArch64Instr::St1MulStructures3RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b0111) => AArch64Instr::St1MulStructures1RegImmOffsetVar(data),
                    (0b0, 0b11111, 0b1000) => AArch64Instr::St2MulStructuresImmOffsetVar(data),
                    (0b0, 0b11111, 0b1010) => AArch64Instr::St1MulStructures2RegImmOffsetVar(data),

                    (0b1, rm, 0b0000) if rm != 0b11111 => {
                        AArch64Instr::Ld4MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0010) if rm != 0b11111 => {
                        AArch64Instr::Ld1MulStructures4RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0100) if rm != 0b11111 => {
                        AArch64Instr::Ld3MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0110) if rm != 0b11111 => {
                        AArch64Instr::Ld1MulStructures3RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b0111) if rm != 0b11111 => {
                        AArch64Instr::Ld1MulStructures1RegRegOffsetVar(data)
                    }
                    (0b1, rm, 0b1000) if rm != 0b11111 => {
                        AArch64Instr::Ld2MulStructuresRegOffsetVar(data)
                    }
                    (0b1, rm, 0b1010) if rm != 0b11111 => {
                        AArch64Instr::Ld1MulStructures2RegRegOffsetVar(data)
                    }

                    (0b1, 0b11111, 0b0000) => AArch64Instr::Ld4MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b0010) => AArch64Instr::Ld1MulStructures4RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b0100) => AArch64Instr::Ld3MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b0110) => AArch64Instr::Ld1MulStructures3RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b0111) => AArch64Instr::Ld1MulStructures1RegImmOffsetVar(data),
                    (0b1, 0b11111, 0b1000) => AArch64Instr::Ld2MulStructuresImmOffsetVar(data),
                    (0b1, 0b11111, 0b1010) => AArch64Instr::Ld1MulStructures2RegImmOffsetVar(data),
                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_conv_between_float_and_int(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xx_xxx_000000_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             rmode: Extract<BitRange<19, 21>, u8>,
             opcode: Extract<BitRange<16, 19>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RnRd {
                    rn: rn.value,
                    rd: rd.value,
                };

                match (sf.value, s.value, ptype.value, rmode.value, opcode.value) {
                    (0b0, 0b0, 0b00, 0b00, 0b000) => {
                        AArch64Instr::FcvtnsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b001) => {
                        AArch64Instr::FcvtnuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b010) => {
                        AArch64Instr::ScvtfScalarInt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b011) => {
                        AArch64Instr::UcvtfScalarInt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b100) => {
                        AArch64Instr::FcvtasScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b101) => {
                        AArch64Instr::FcvtauScalarSinglePrecisionTo32(data)
                    }

                    (0b0, 0b0, 0b00, 0b00, 0b110) => {
                        AArch64Instr::FmovGeneralSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b111) => {
                        AArch64Instr::FmovGeneral32ToSinglePrecision(data)
                    }

                    (0b0, 0b0, 0b00, 0b01, 0b000) => {
                        AArch64Instr::FcvtpsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b01, 0b001) => {
                        AArch64Instr::FcvtpuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b10, 0b000) => {
                        AArch64Instr::FcvtmsScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b10, 0b001) => {
                        AArch64Instr::FcvtmuScalarSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b000) => {
                        AArch64Instr::FcvtzsScalarIntSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b001) => {
                        AArch64Instr::FcvtzuScalarIntSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b000) => {
                        AArch64Instr::FcvtnsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b001) => {
                        AArch64Instr::FcvtnuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b010) => {
                        AArch64Instr::ScvtfScalarInt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b011) => {
                        AArch64Instr::UcvtfScalarInt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b100) => {
                        AArch64Instr::FcvtasScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b101) => {
                        AArch64Instr::FcvtauScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b01, 0b000) => {
                        AArch64Instr::FcvtpsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b01, 0b001) => {
                        AArch64Instr::FcvtpuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b10, 0b000) => {
                        AArch64Instr::FcvtmsScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b10, 0b001) => {
                        AArch64Instr::FcvtmuScalarDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b000) => {
                        AArch64Instr::FcvtzsScalarIntDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b001) => {
                        AArch64Instr::FcvtzsScalarIntDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b110) => AArch64Instr::Fjcvtzs(data),

                    (0b1, 0b0, 0b00, 0b00, 0b000) => {
                        AArch64Instr::FcvtnsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b001) => {
                        AArch64Instr::FcvtnuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b010) => {
                        AArch64Instr::ScvtfScalarInt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b011) => {
                        AArch64Instr::UcvtfScalarInt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b100) => {
                        AArch64Instr::FcvtasScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b101) => {
                        AArch64Instr::FcvtauScalarSinglePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b01, 0b00, 0b110) => {
                        AArch64Instr::FmovGeneralDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b111) => {
                        AArch64Instr::FmovGeneral64ToDoublePrecision(data)
                    }

                    (0b1, 0b0, 0b00, 0b01, 0b000) => {
                        AArch64Instr::FcvtpsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b01, 0b001) => {
                        AArch64Instr::FcvtpuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b10, 0b000) => {
                        AArch64Instr::FcvtmsScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b10, 0b001) => {
                        AArch64Instr::FcvtmuScalarSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b000) => {
                        AArch64Instr::FcvtzsScalarIntSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b001) => {
                        AArch64Instr::FcvtzuScalarIntSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b000) => {
                        AArch64Instr::FcvtnsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b001) => {
                        AArch64Instr::FcvtnuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b010) => {
                        AArch64Instr::ScvtfScalarInt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b011) => {
                        AArch64Instr::UcvtfScalarInt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b100) => {
                        AArch64Instr::FcvtasScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b101) => {
                        AArch64Instr::FcvtauScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b01, 0b000) => {
                        AArch64Instr::FcvtpsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b01, 0b001) => {
                        AArch64Instr::FcvtpuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b10, 0b000) => {
                        AArch64Instr::FcvtmsScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b10, 0b001) => {
                        AArch64Instr::FcvtmuScalarDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b000) => {
                        AArch64Instr::FcvtzsScalarIntDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b001) => {
                        AArch64Instr::FcvtzsScalarIntDoublePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b10, 0b01, 0b110) => {
                        AArch64Instr::FmovGeneralTopHalfOf128To64(data)
                    }
                    (0b1, 0b0, 0b10, 0b01, 0b111) => {
                        AArch64Instr::FmovGeneral64toTopHalfOf128(data)
                    }

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_modified_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01111_00000_x_x_x_xxxx_x_1_x_x_x_x_x_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             op: Extract<BitRange<29, 30>, u8>,
             a: Extract<BitRange<18, 19>, u8>,
             b: Extract<BitRange<17, 18>, u8>,
             c: Extract<BitRange<16, 17>, u8>,
             cmode: Extract<BitRange<12, 16>, u8>,
             o2: Extract<BitRange<11, 12>, u8>,
             d: Extract<BitRange<9, 10>, u8>,
             e: Extract<BitRange<8, 9>, u8>,
             f: Extract<BitRange<7, 8>, u8>,
             g: Extract<BitRange<6, 7>, u8>,
             h: Extract<BitRange<5, 6>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdModifiedImm {
                    q: q.value,
                    a: a.value,
                    b: b.value,
                    c: c.value,
                    cmode: cmode.value,
                    d: d.value,
                    e: e.value,
                    f: f.value,
                    g: g.value,
                    h: h.value,
                    rd: rd.value,
                };
                let cmode = cmode.value as u32;
                let cmode0 = extract_bits32(3..4, cmode);
                let cmode1 = extract_bits32(2..3, cmode);
                let cmode2 = extract_bits32(1..2, cmode);
                let cmode3 = extract_bits32(0..1, cmode);

                match (q.value, op.value, cmode0, cmode1, cmode2, cmode3, o2.value) {
                    (_, 0b0, 0, _, _, 0, 0b0) => AArch64Instr::MoviShiftedImmVar32(data),
                    (_, 0b0, 0, _, _, 1, 0b0) => AArch64Instr::OrrVecImmVar32(data),
                    (_, 0b0, 1, 0, _, 0, 0b0) => AArch64Instr::MoviShiftedImmVar16(data),
                    (_, 0b0, 1, 0, _, 1, 0b0) => AArch64Instr::OrrVecImmVar16(data),
                    (_, 0b0, 1, 1, 0, _, 0b0) => AArch64Instr::MoviShiftingOnesVar32(data),
                    (_, 0b0, 1, 1, 1, 0, 0b0) => AArch64Instr::MoviVar8(data),
                    (_, 0b0, 1, 1, 1, 1, 0b0) => AArch64Instr::FmovVecImmSinglePrecisionVar(data),

                    (_, 0b1, 0, _, _, 0, 0b0) => AArch64Instr::MvniShiftedImmVar32(data),
                    (_, 0b1, 0, _, _, 1, 0b0) => AArch64Instr::BicVecImmVar32(data),
                    (_, 0b1, 1, 0, _, 0, 0b0) => AArch64Instr::MvniShiftedImmVar16(data),
                    (_, 0b1, 1, 0, _, 1, 0b0) => AArch64Instr::BicVecImmVar16(data),

                    (_, 0b1, 1, 1, 0, _, 0b0) => AArch64Instr::MvniShiftingOnesVar32(data),
                    (0b0, 0b1, 1, 1, 1, 0, 0b0) => AArch64Instr::MoviScalarVar64(data),

                    (0b1, 0b1, 1, 1, 1, 0, 0b0) => AArch64Instr::MoviVectorVar64(data),
                    (0b1, 0b1, 1, 1, 1, 1, 0b0) => AArch64Instr::FmovVecImmDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_cond_cmp_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010010_xxxxx_xxxx_1_x_xxxxx_x_xxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             imm5: Extract<BitRange<16, 21>, u8>,
             cond: Extract<BitRange<12, 16>, u8>,
             o2: Extract<BitRange<10, 11>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             o3: Extract<BitRange<4, 5>, u8>,
             nzcv: Extract<BitRange<0, 4>, u8>| {
                let data = CondCmpImm {
                    imm5: imm5.value,
                    cond: cond.value,
                    rn: rn.value,
                    nzcv: nzcv.value,
                };

                match (sf_op_s.value, o2.value, o3.value) {
                    (0b001, 0b0, 0b0) => AArch64Instr::CcmnImmVar32(data),
                    (0b011, 0b0, 0b0) => AArch64Instr::CcmpImmVar32(data),
                    (0b101, 0b0, 0b0) => AArch64Instr::CcmnImmVar64(data),
                    (0b111, 0b0, 0b0) => AArch64Instr::CcmpImmVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_exclusive_register(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_0010000_x_0_xxxxx_x_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             rs: Extract<BitRange<16, 21>, u8>,
             o0: Extract<BitRange<15, 16>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreRegExclusive {
                    rs: rs.value,
                    rt2: rt2.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, l.value, o0.value) {
                    (0b00, 0b0, 0b0) => AArch64Instr::Stxrb(data),
                    (0b00, 0b1, 0b0) => AArch64Instr::Ldxrb(data),
                    (0b01, 0b0, 0b0) => AArch64Instr::Stxrh(data),
                    (0b01, 0b1, 0b0) => AArch64Instr::Ldxrh(data),
                    (0b10, 0b0, 0b0) => AArch64Instr::StxrVar32(data),
                    (0b10, 0b1, 0b0) => AArch64Instr::LdxrVar32(data),
                    (0b11, 0b0, 0b0) => AArch64Instr::StxrVar64(data),
                    (0b11, 0b1, 0b0) => AArch64Instr::LdxrVar64(data),

                    (0b00, 0b0, 0b1) => AArch64Instr::Stlxrb(data),
                    (0b00, 0b1, 0b1) => AArch64Instr::Ldaxrb(data),
                    (0b01, 0b0, 0b1) => AArch64Instr::Stlxrh(data),
                    (0b01, 0b1, 0b1) => AArch64Instr::Ldaxrh(data),
                    (0b10, 0b0, 0b1) => AArch64Instr::StlxrVar32(data),
                    (0b10, 0b1, 0b1) => AArch64Instr::LdaxrVar32(data),
                    (0b11, 0b0, 0b1) => AArch64Instr::StlxrVar64(data),
                    (0b11, 0b1, 0b1) => AArch64Instr::LdaxrVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_store_ordered(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_0010001_x_0_xxxxx_x_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             rs: Extract<BitRange<16, 21>, u8>,
             o0: Extract<BitRange<15, 16>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = LoadStoreOrdered {
                    rs: rs.value,
                    rt2: rt2.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, l.value, o0.value) {
                    (0b00, 0b0, 0b1) => AArch64Instr::Stlrb(data),
                    (0b00, 0b1, 0b1) => AArch64Instr::Ldarb(data),
                    (0b01, 0b0, 0b1) => AArch64Instr::Stlrh(data),
                    (0b01, 0b1, 0b1) => AArch64Instr::Ldarh(data),
                    (0b10, 0b0, 0b1) => AArch64Instr::StlrVar32(data),
                    (0b10, 0b1, 0b1) => AArch64Instr::LdarVar32(data),
                    (0b11, 0b0, 0b1) => AArch64Instr::StlrVar64(data),
                    (0b11, 0b1, 0b1) => AArch64Instr::LdarVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_advanced_simd_three_same(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01110_xx_1_xxxxx_xxxxx_1_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<11, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = QSizeRmRnRd {
                    q: q.value,
                    size: size.value,
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b00000) => AArch64Instr::Shadd(data),
                    (0b0, _, 0b00001) => AArch64Instr::Sqadd(data),
                    (0b0, _, 0b00010) => AArch64Instr::Srhadd(data),
                    (0b0, _, 0b00100) => AArch64Instr::Shsub(data),
                    (0b0, _, 0b00101) => AArch64Instr::Sqsub(data),
                    (0b0, _, 0b00110) => AArch64Instr::CmgtReg(data),
                    (0b0, _, 0b00111) => AArch64Instr::CmgeReg(data),
                    (0b0, _, 0b01000) => AArch64Instr::Sshl(data),
                    (0b0, _, 0b01001) => AArch64Instr::SqshlReg(data),
                    (0b0, _, 0b01010) => AArch64Instr::Srshl(data),
                    (0b0, _, 0b01011) => AArch64Instr::Sqrshl(data),
                    (0b0, _, 0b01100) => AArch64Instr::Smax(data),
                    (0b0, _, 0b01101) => AArch64Instr::Smin(data),
                    (0b0, _, 0b01110) => AArch64Instr::Sabd(data),
                    (0b0, _, 0b01111) => AArch64Instr::Saba(data),
                    (0b0, _, 0b10000) => AArch64Instr::AddVec(data),
                    (0b0, _, 0b10001) => AArch64Instr::Cmtst(data),
                    (0b0, _, 0b10010) => AArch64Instr::MlaVec(data),
                    (0b0, _, 0b10011) => AArch64Instr::MulVec(data),
                    (0b0, _, 0b10100) => AArch64Instr::Smaxp(data),
                    (0b0, _, 0b10101) => AArch64Instr::Sminp(data),
                    (0b0, _, 0b10110) => AArch64Instr::SqdmulhVec(data),
                    (0b0, _, 0b10111) => AArch64Instr::AddpVec(data),

                    (0b0, 0b00 | 0b01, 0b11000) => AArch64Instr::FmaxnmVec(data),
                    (0b0, 0b00 | 0b01, 0b11001) => AArch64Instr::FmlaVec(data),
                    (0b0, 0b00 | 0b01, 0b11010) => AArch64Instr::FaddVec(data),
                    (0b0, 0b00 | 0b01, 0b11011) => AArch64Instr::Fmulx(data),
                    (0b0, 0b00 | 0b01, 0b11100) => AArch64Instr::FcmeqReg(data),
                    (0b0, 0b00 | 0b01, 0b11110) => AArch64Instr::FmaxVec(data),
                    (0b0, 0b00 | 0b01, 0b11111) => AArch64Instr::Frecps(data),

                    (0b0, 0b00, 0b00011) => AArch64Instr::AndVec(data),
                    (0b0, 0b01, 0b00011) => AArch64Instr::AndVec(data),

                    (0b0, 0b10 | 0b11, 0b11000) => AArch64Instr::FminnmVec(data),
                    (0b0, 0b10 | 0b11, 0b11001) => AArch64Instr::FmlsVec(data),
                    (0b0, 0b10 | 0b11, 0b11010) => AArch64Instr::FsubVec(data),
                    (0b0, 0b10 | 0b11, 0b11110) => AArch64Instr::FminVec(data),
                    (0b0, 0b10 | 0b11, 0b11111) => AArch64Instr::Frsqrts(data),

                    (0b0, 0b10, 0b00011) => AArch64Instr::OrrVecReg(data),
                    (0b0, 0b11, 0b00011) => AArch64Instr::OrnVec(data),

                    (0b1, _, 0b00000) => AArch64Instr::Uhadd(data),
                    (0b1, _, 0b00001) => AArch64Instr::Uqadd(data),
                    (0b1, _, 0b00010) => AArch64Instr::Urhadd(data),
                    (0b1, _, 0b00100) => AArch64Instr::Uhsub(data),
                    (0b1, _, 0b00101) => AArch64Instr::Uqsub(data),
                    (0b1, _, 0b00110) => AArch64Instr::CmhiReg(data),
                    (0b1, _, 0b00111) => AArch64Instr::CmhsReg(data),
                    (0b1, _, 0b01000) => AArch64Instr::Ushl(data),
                    (0b1, _, 0b01001) => AArch64Instr::UqshlReg(data),
                    (0b1, _, 0b01010) => AArch64Instr::Urshl(data),
                    (0b1, _, 0b01011) => AArch64Instr::Uqrshl(data),
                    (0b1, _, 0b01100) => AArch64Instr::Umax(data),
                    (0b1, _, 0b01101) => AArch64Instr::Umin(data),
                    (0b1, _, 0b01110) => AArch64Instr::Uabd(data),
                    (0b1, _, 0b01111) => AArch64Instr::Uaba(data),

                    (0b1, _, 0b10000) => AArch64Instr::SubVec(data),
                    (0b1, _, 0b10001) => AArch64Instr::CmeqReg(data),
                    (0b1, _, 0b10010) => AArch64Instr::MlsVec(data),
                    (0b1, _, 0b10011) => AArch64Instr::Pmul(data),
                    (0b1, _, 0b10100) => AArch64Instr::Umaxp(data),
                    (0b1, _, 0b10101) => AArch64Instr::Uminp(data),
                    (0b1, _, 0b10110) => AArch64Instr::SqrdmulhVec(data),

                    (0b1, 0b00 | 0b01, 0b11000) => AArch64Instr::FmaxnmpVec(data),
                    (0b1, 0b00 | 0b01, 0b11010) => AArch64Instr::FaddpVec(data),
                    (0b1, 0b00 | 0b01, 0b11011) => AArch64Instr::FmulVec(data),
                    (0b1, 0b00 | 0b01, 0b11100) => AArch64Instr::FcmgeReg(data),
                    (0b1, 0b00 | 0b01, 0b11101) => AArch64Instr::Facge(data),
                    (0b1, 0b00 | 0b01, 0b11110) => AArch64Instr::FmaxpVec(data),
                    (0b1, 0b00 | 0b01, 0b11111) => AArch64Instr::FdivVec(data),

                    (0b1, 0b00, 0b00011) => AArch64Instr::EorVec(data),
                    (0b1, 0b01, 0b00011) => AArch64Instr::Bsl(data),

                    (0b1, 0b10 | 0b11, 0b11000) => AArch64Instr::FminnmpVec(data),
                    (0b1, 0b10 | 0b11, 0b11010) => AArch64Instr::Fabd(data),
                    (0b1, 0b10 | 0b11, 0b11100) => AArch64Instr::FcmgtReg(data),
                    (0b1, 0b10 | 0b11, 0b11101) => AArch64Instr::Facgt(data),
                    (0b1, 0b10 | 0b11, 0b11110) => AArch64Instr::FminpVec(data),

                    (0b1, 0b10, 0b00011) => AArch64Instr::Bit(data),
                    (0b1, 0b11, 0b00011) => AArch64Instr::Bif(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_shift_by_imm(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_011110_xxxx_xxx_xxxxx_1_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             u: Extract<BitRange<29, 30>, u8>,
             immb: Extract<BitRange<16, 19>, u8>,
             opcode: Extract<BitRange<11, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdShiftByImm {
                    q: q.value,
                    immb: immb.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, opcode.value) {
                    (0b0, 0b00000) => AArch64Instr::Sshr(data),
                    (0b0, 0b00010) => AArch64Instr::Ssra(data),
                    (0b0, 0b00100) => AArch64Instr::Srshr(data),
                    (0b0, 0b00110) => AArch64Instr::Srsra(data),
                    (0b0, 0b01010) => AArch64Instr::Shl(data),
                    (0b0, 0b01110) => AArch64Instr::SqshlImm(data),
                    (0b0, 0b10000) => AArch64Instr::Shrn(data),
                    (0b0, 0b10001) => AArch64Instr::Rshrn(data),
                    (0b0, 0b10010) => AArch64Instr::Sqshrn(data),
                    (0b0, 0b10011) => AArch64Instr::Sqrshrn(data),
                    (0b0, 0b10100) => AArch64Instr::Sshll(data),
                    (0b0, 0b11100) => AArch64Instr::ScvtfVecFixedPt(data),
                    (0b0, 0b11111) => AArch64Instr::FcvtzsVecFixedPt(data),

                    (0b1, 0b00000) => AArch64Instr::Ushr(data),
                    (0b1, 0b00010) => AArch64Instr::Usra(data),
                    (0b1, 0b00100) => AArch64Instr::Urshr(data),
                    (0b1, 0b00110) => AArch64Instr::Ursra(data),

                    (0b1, 0b01000) => AArch64Instr::Sri(data),
                    (0b1, 0b01010) => AArch64Instr::Sli(data),

                    (0b1, 0b01100) => AArch64Instr::Sqshlu(data),
                    (0b1, 0b01110) => AArch64Instr::UqshlImm(data),

                    (0b1, 0b10000) => AArch64Instr::Sqshrun(data),
                    (0b1, 0b10001) => AArch64Instr::Sqrshrun(data),
                    (0b1, 0b10010) => AArch64Instr::Uqshrn(data),
                    (0b1, 0b10011) => AArch64Instr::Uqrshrn(data),
                    (0b1, 0b10100) => AArch64Instr::Ushll(data),
                    (0b1, 0b11100) => AArch64Instr::UcvtfVecFixedPt(data),
                    (0b1, 0b11111) => AArch64Instr::FcvtzuVecFixedPt(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_float_data_proc_1src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xxxxxx_10000_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             opcode: Extract<BitRange<15, 21>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RnRd {
                    rn: rn.value,
                    rd: rd.value,
                };

                match (m.value, s.value, ptype.value, opcode.value) {
                    (0b0, 0b0, 0b00, 0b000000) => AArch64Instr::FmovRegSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000001) => AArch64Instr::FabsScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000010) => AArch64Instr::FnegScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b000011) => AArch64Instr::FsqrtScalarSinglePrecisionVar(data),

                    (0b0, 0b0, 0b00, 0b000101) => {
                        AArch64Instr::FcvtSingleToDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b000111) => AArch64Instr::FcvtSingleToHalfPrecisionVar(data),

                    (0b0, 0b0, 0b00, 0b001000) => {
                        AArch64Instr::FrintnScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001001) => {
                        AArch64Instr::FrintpScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001010) => {
                        AArch64Instr::FrintmScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001011) => {
                        AArch64Instr::FrintzScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001100) => {
                        AArch64Instr::FrintaScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001110) => {
                        AArch64Instr::FrintxScalarSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b00, 0b001111) => {
                        AArch64Instr::FrintiScalarSinglePrecisionVar(data)
                    }

                    (0b0, 0b0, 0b01, 0b000000) => AArch64Instr::FmovRegDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000001) => AArch64Instr::FabsScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000010) => AArch64Instr::FnegScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b000011) => AArch64Instr::FsqrtScalarDoublePrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b000100) => {
                        AArch64Instr::FcvtDoubleToSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b000111) => AArch64Instr::FcvtDoubleToHalfPrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b001000) => {
                        AArch64Instr::FrintnScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001001) => {
                        AArch64Instr::FrintpScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001010) => {
                        AArch64Instr::FrintmScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001011) => {
                        AArch64Instr::FrintzScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001100) => {
                        AArch64Instr::FrintaScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001110) => {
                        AArch64Instr::FrintxScalarDoublePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b001111) => {
                        AArch64Instr::FrintiScalarDoublePrecisionVar(data)
                    }

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_scalar_pairwise(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "01_x_11110_xx_11000_xxxxx_10_xxxxx_xxxxx",
            |raw_instr: u32,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             opcode: Extract<BitRange<12, 17>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdScalarPairwise {
                    size: size.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b11011) => AArch64Instr::AddpScalar(data),
                    (0b0, 0b00 | 0b01, 0b01100) => AArch64Instr::FmaxnmpScalarEncoding(data),
                    (0b0, 0b00 | 0b01, 0b01101) => AArch64Instr::FaddpScalarEncoding(data),
                    (0b0, 0b00 | 0b01, 0b01111) => AArch64Instr::FmaxpScalarEncoding(data),
                    (0b0, 0b10 | 0b11, 0b01100) => AArch64Instr::FminnmpScalarEncoding(data),
                    (0b0, 0b10 | 0b11, 11) => AArch64Instr::FminpScalarEncoding(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_ld_st_single_structure(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_0011010_x_x_00000_xxx_x_xx_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             r: Extract<BitRange<21, 22>, u8>,
             opcode: Extract<BitRange<13, 16>, u8>,
             s: Extract<BitRange<12, 13>, u8>,
             size: Extract<BitRange<10, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdLdStSingleStructure {
                    q: q.value,
                    s: s.value,
                    size: size.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (l.value, r.value, opcode.value, s.value, size.value) {
                    (0b0, 0b0, 0b000, _, _) => AArch64Instr::St1SingleStructureVar8(data),
                    (0b0, 0b0, 0b001, _, _) => AArch64Instr::St3SingleStructureVar8(data),
                    (0b0, 0b0, 0b010, _, 0b00 | 0b10) => {
                        AArch64Instr::St1SingleStructureVar16(data)
                    }
                    (0b0, 0b0, 0b011, _, 0b00 | 0b10) => {
                        AArch64Instr::St3SingleStructureVar16(data)
                    }

                    (0b0, 0b0, 0b100, _, 0b00) => AArch64Instr::St1SingleStructureVar32(data),
                    (0b0, 0b0, 0b100, 0b0, 0b01) => AArch64Instr::St1SingleStructureVar64(data),
                    (0b0, 0b0, 0b101, _, 0b00) => AArch64Instr::St3SingleStructureVar32(data),
                    (0b0, 0b0, 0b101, 0b0, 0b01) => AArch64Instr::St3SingleStructureVar64(data),

                    (0b0, 0b1, 0b000, _, _) => AArch64Instr::St2SingleStructureVar8(data),
                    (0b0, 0b1, 0b001, _, _) => AArch64Instr::St4SingleStructureVar8(data),
                    (0b0, 0b1, 0b010, _, 0b00 | 0b10) => {
                        AArch64Instr::St2SingleStructureVar16(data)
                    }

                    (0b0, 0b1, 0b011, _, 0b00 | 0b10) => {
                        AArch64Instr::St4SingleStructureVar16(data)
                    }

                    (0b0, 0b1, 0b100, _, 0b00) => AArch64Instr::St2SingleStructureVar32(data),
                    (0b0, 0b1, 0b100, 0b0, 0b01) => AArch64Instr::St2SingleStructureVar64(data),
                    (0b0, 0b1, 0b101, _, 0b00) => AArch64Instr::St4SingleStructureVar32(data),
                    (0b0, 0b1, 0b101, 0b0, 0b01) => AArch64Instr::St4SingleStructureVar64(data),

                    (0b1, 0b0, 0b000, _, _) => AArch64Instr::Ld1SingleStructureVar8(data),
                    (0b1, 0b0, 0b001, _, _) => AArch64Instr::Ld3SingleStructureVar8(data),
                    (0b1, 0b0, 0b010, _, 0b00 | 0b10) => {
                        AArch64Instr::Ld1SingleStructureVar16(data)
                    }

                    (0b1, 0b0, 0b011, _, 0b00 | 0b10) => {
                        AArch64Instr::Ld3SingleStructureVar16(data)
                    }

                    (0b1, 0b0, 0b100, _, 0b00) => AArch64Instr::Ld1SingleStructureVar32(data),
                    (0b1, 0b0, 0b100, 0b0, 0b01) => AArch64Instr::Ld1SingleStructureVar64(data),
                    (0b1, 0b0, 0b101, _, 0b00) => AArch64Instr::Ld3SingleStructureVar32(data),
                    (0b1, 0b0, 0b101, 0b0, 0b01) => AArch64Instr::Ld3SingleStructureVar64(data),

                    (0b1, 0b0, 0b110, 0b0, _) => AArch64Instr::Ld1r(data),
                    (0b1, 0b0, 0b111, 0b0, _) => AArch64Instr::Ld3r(data),

                    (0b1, 0b1, 0b000, _, _) => AArch64Instr::Ld2SingleStructureVar8(data),
                    (0b1, 0b1, 0b001, _, _) => AArch64Instr::Ld4SingleStructureVar8(data),
                    (0b1, 0b1, 0b010, _, 0b00 | 0b10) => {
                        AArch64Instr::Ld2SingleStructureVar16(data)
                    }

                    (0b1, 0b1, 0b011, _, 0b00 | 0b10) => {
                        AArch64Instr::Ld4SingleStructureVar16(data)
                    }

                    (0b1, 0b1, 0b100, _, 0b00) => AArch64Instr::Ld2SingleStructureVar32(data),
                    (0b1, 0b1, 0b100, 0b0, 0b01) => AArch64Instr::Ld2SingleStructureVar64(data),
                    (0b1, 0b1, 0b101, _, 0b00) => AArch64Instr::Ld4SingleStructureVar32(data),
                    (0b1, 0b1, 0b101, 0b0, 0b01) => AArch64Instr::Ld4SingleStructureVar64(data),

                    (0b1, 0b1, 0b110, 0b0, _) => AArch64Instr::Ld2r(data),
                    (0b1, 0b1, 0b111, 0b0, _) => AArch64Instr::Ld2r(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_2reg_miscellaneous(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01110_xx_10000_xxxxx_10_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             opcode: Extract<BitRange<12, 17>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = QSizeRnRd {
                    q: q.value,
                    size: size.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b00000) => AArch64Instr::Rev64(data),
                    (0b0, _, 0b00001) => AArch64Instr::Rev16Vec(data),
                    (0b0, _, 0b00010) => AArch64Instr::Saddlp(data),
                    (0b0, _, 0b00011) => AArch64Instr::Suqadd(data),
                    (0b0, _, 0b00100) => AArch64Instr::ClsVec(data),
                    (0b0, _, 0b00101) => AArch64Instr::Cnt(data),
                    (0b0, _, 0b00110) => AArch64Instr::Sadalp(data),
                    (0b0, _, 0b00111) => AArch64Instr::Sqabs(data),
                    (0b0, _, 0b01000) => AArch64Instr::CmgtZero(data),
                    (0b0, _, 0b01001) => AArch64Instr::CmeqZero(data),
                    (0b0, _, 0b01010) => AArch64Instr::CmltZero(data),
                    (0b0, _, 0b01011) => AArch64Instr::Abs(data),
                    (0b0, _, 0b10010) => AArch64Instr::XtnXtn2(data),
                    (0b0, _, 0b10100) => AArch64Instr::Sqxtn(data),

                    (0b0, 0b00 | 0b01, 0b10110) => AArch64Instr::Fcvtn(data),
                    (0b0, 0b00 | 0b01, 0b10111) => AArch64Instr::Fcvtl(data),
                    (0b0, 0b00 | 0b01, 0b11000) => AArch64Instr::FrintnVec(data),
                    (0b0, 0b00 | 0b01, 0b11001) => AArch64Instr::FrintmVec(data),
                    (0b0, 0b00 | 0b01, 0b11010) => AArch64Instr::FcvtnsVec(data),
                    (0b0, 0b00 | 0b01, 0b11011) => AArch64Instr::FcvtmsVec(data),
                    (0b0, 0b00 | 0b01, 0b11100) => AArch64Instr::FcvtasVec(data),
                    (0b0, 0b00 | 0b01, 0b11101) => AArch64Instr::ScvtfVecInt(data),

                    (0b0, 0b10 | 0b11, 0b01100) => AArch64Instr::FcmgtZero(data),
                    (0b0, 0b10 | 0b11, 0b01101) => AArch64Instr::FcmeqZero(data),
                    (0b0, 0b10 | 0b11, 0b01110) => AArch64Instr::FcmltZero(data),
                    (0b0, 0b10 | 0b11, 0b01111) => AArch64Instr::FabsVec(data),
                    (0b0, 0b10 | 0b11, 0b11000) => AArch64Instr::FrintpVec(data),
                    (0b0, 0b10 | 0b11, 0b11001) => AArch64Instr::FrintzVec(data),
                    (0b0, 0b10 | 0b11, 0b11010) => AArch64Instr::FcvtpsVec(data),
                    (0b0, 0b10 | 0b11, 0b11011) => AArch64Instr::FcvtzsVecInt(data),
                    (0b0, 0b10 | 0b11, 0b11100) => AArch64Instr::Urecpe(data),
                    (0b0, 0b10 | 0b11, 0b11101) => AArch64Instr::Frecpe(data),

                    (0b1, _, 0b00000) => AArch64Instr::Rev32Vec(data),
                    (0b1, _, 0b00010) => AArch64Instr::Uaddlp(data),
                    (0b1, _, 0b00011) => AArch64Instr::Usqadd(data),
                    (0b1, _, 0b00100) => AArch64Instr::ClzVec(data),
                    (0b1, _, 0b00110) => AArch64Instr::Uadalp(data),
                    (0b1, _, 0b00111) => AArch64Instr::Sqneg(data),
                    (0b1, _, 0b01000) => AArch64Instr::CmgeZero(data),
                    (0b1, _, 0b01001) => AArch64Instr::CmleZero(data),
                    (0b1, _, 0b01011) => AArch64Instr::NegVec(data),
                    (0b1, _, 0b10010) => AArch64Instr::Sqxtun(data),
                    (0b1, _, 0b10011) => AArch64Instr::Shll(data),
                    (0b1, _, 0b10100) => AArch64Instr::Uqxtn(data),

                    (0b1, 0b00 | 0b01, 0b10110) => AArch64Instr::Fcvtxn(data),
                    (0b1, 0b00 | 0b01, 0b11000) => AArch64Instr::FrintaVec(data),
                    (0b1, 0b00 | 0b01, 0b11001) => AArch64Instr::FrintxVec(data),
                    (0b1, 0b00 | 0b01, 0b11010) => AArch64Instr::FcvtnuVec(data),
                    (0b1, 0b00 | 0b01, 0b11011) => AArch64Instr::FcvtmuVec(data),
                    (0b1, 0b00 | 0b01, 0b11100) => AArch64Instr::FcvtauVec(data),
                    (0b1, 0b00 | 0b01, 0b11101) => AArch64Instr::UcvtfVecInt(data),

                    (0b1, 0b00, 0b00101) => AArch64Instr::Not(data),
                    (0b1, 0b01, 0b00101) => AArch64Instr::RbitVec(data),

                    (0b1, 0b10 | 0b11, 0b01100) => AArch64Instr::FcmgeZero(data),
                    (0b1, 0b10 | 0b11, 0b01101) => AArch64Instr::FcmleZero(data),
                    (0b1, 0b10 | 0b11, 0b01111) => AArch64Instr::FnegVec(data),
                    (0b1, 0b10 | 0b11, 0b11001) => AArch64Instr::FrintiVec(data),
                    (0b1, 0b10 | 0b11, 0b11010) => AArch64Instr::FcvtpuVec(data),
                    (0b1, 0b10 | 0b11, 0b11011) => AArch64Instr::FcvtzuVecInt(data),
                    (0b1, 0b10 | 0b11, 0b11100) => AArch64Instr::Ursqrte(data),
                    (0b1, 0b10 | 0b11, 0b11101) => AArch64Instr::Frsqrte(data),
                    (0b1, 0b10 | 0b11, 0b11111) => AArch64Instr::FsqrtVec(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_across_lanes(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01110_xx_11000_xxxxx_10_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             opcode: Extract<BitRange<12, 17>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = QSizeRnRd {
                    q: q.value,
                    size: size.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b00011) => AArch64Instr::Saddlv(data),
                    (0b0, _, 0b01010) => AArch64Instr::Smaxv(data),
                    (0b0, _, 0b11010) => AArch64Instr::Sminv(data),
                    (0b0, _, 0b11011) => AArch64Instr::Addv(data),

                    (0b1, _, 0b00011) => AArch64Instr::Uaddlv(data),
                    (0b1, _, 0b01010) => AArch64Instr::Uaddlv(data),
                    (0b1, _, 0b11010) => AArch64Instr::Uminv(data),

                    (0b1, 0b00 | 0b01, 0b01100) => AArch64Instr::FmaxnvmEncoding(data),
                    (0b1, 0b00 | 0b01, 0b01111) => AArch64Instr::FmaxvEncoding(data),

                    (0b1, 0b10 | 0b11, 0b01100) => AArch64Instr::FminnmvEncoding(data),
                    (0b1, 0b10 | 0b11, 0b01111) => AArch64Instr::FminvEncoding(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_compare_and_swap(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_0010001_x_1_xxxxx_x_xxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             l: Extract<BitRange<22, 23>, u8>,
             rs: Extract<BitRange<16, 21>, u8>,
             o0: Extract<BitRange<15, 16>, u8>,
             rt2: Extract<BitRange<10, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = RsRnRt {
                    rs: rs.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (size.value, l.value, o0.value, rt2.value) {
                    (0b00, 0b0, 0b0, 0b11111) => AArch64Instr::Casb(data),
                    (0b00, 0b0, 0b1, 0b11111) => AArch64Instr::Caslb(data),
                    (0b00, 0b1, 0b0, 0b11111) => AArch64Instr::Casab(data),
                    (0b00, 0b1, 0b1, 0b11111) => AArch64Instr::Casalb(data),

                    (0b01, 0b0, 0b0, 0b11111) => AArch64Instr::Cash(data),
                    (0b01, 0b0, 0b1, 0b11111) => AArch64Instr::Caslh(data),
                    (0b01, 0b1, 0b0, 0b11111) => AArch64Instr::Casah(data),
                    (0b01, 0b1, 0b1, 0b11111) => AArch64Instr::Casalh(data),

                    (0b10, 0b0, 0b0, 0b11111) => AArch64Instr::CasVar32(data),
                    (0b10, 0b0, 0b1, 0b11111) => AArch64Instr::CaslVar32(data),
                    (0b10, 0b1, 0b0, 0b11111) => AArch64Instr::CasaVar32(data),
                    (0b10, 0b1, 0b1, 0b11111) => AArch64Instr::CasalVar32(data),

                    (0b11, 0b0, 0b0, 0b11111) => AArch64Instr::CasVar64(data),
                    (0b11, 0b0, 0b1, 0b11111) => AArch64Instr::CaslVar64(data),
                    (0b11, 0b1, 0b0, 0b11111) => AArch64Instr::CasaVar64(data),
                    (0b11, 0b1, 0b1, 0b11111) => AArch64Instr::CasalVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_atomic_memory_operations(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_111_x_00_x_x_1_xxxxx_x_xxx_00_xxxxx_xxxxx",
            |raw_instr: u32,
             size: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             a: Extract<BitRange<23, 24>, u8>,
             r: Extract<BitRange<22, 23>, u8>,
             rs: Extract<BitRange<16, 21>, u8>,
             o3: Extract<BitRange<15, 16>, u8>,
             opc: Extract<BitRange<12, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = RsRnRt {
                    rs: rs.value,
                    rn: rn.value,
                    rt: rt.value,
                };

                match (
                    size.value, v.value, a.value, r.value, rs.value, o3.value, opc.value,
                ) {
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminbVar(data),
                    (0b00, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpbVar(data),

                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclrlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeorlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminlbVar(data),
                    (0b00, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Instr::SwplbVar(data),

                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminabVar(data),
                    (0b00, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpabVar(data),

                    (0b00, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Instr::Ldaprb(data),

                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclralbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeoralbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminalbVar(data),
                    (0b00, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Instr::SwpalbVar(data),

                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsethVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminhVar(data),
                    (0b01, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Instr::SwphVar(data),

                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclrlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeorlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminlhVar(data),
                    (0b01, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Instr::SwplhVar(data),

                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminahVar(data),
                    (0b01, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpahVar(data),

                    (0b01, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Instr::Ldaprh(data),

                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclralhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeoralhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminalhVar(data),
                    (0b01, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Instr::SwpalhVar(data),

                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminVar32(data),
                    (0b10, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpVar32(data),

                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclrlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeorlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminlVar32(data),
                    (0b10, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Instr::SwplVar32(data),

                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclraVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeoraVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpaVar32(data),
                    (0b10, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Instr::LdaprVar32(data),

                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclralVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeoralVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminalVar32(data),
                    (0b10, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Instr::SwpalVar32(data),

                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclrVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeorVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminVar64(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpVar64(data),

                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b010) => AArch64Instr::St64bv0(data),
                    (0b11, 0b0, 0b0, 0b0, _, 0b1, 0b011) => AArch64Instr::St64bv(data),
                    (0b11, 0b0, 0b0, 0b0, 0b11111, 0b1, 0b001) => AArch64Instr::St64b(data),
                    (0b11, 0b0, 0b0, 0b0, 0b11111, 0b1, 0b101) => AArch64Instr::Ld64b(data),

                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclrlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeorlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminlVar64(data),
                    (0b11, 0b0, 0b0, 0b1, _, 0b1, 0b000) => AArch64Instr::SwplVar64(data),

                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b000) => AArch64Instr::LdaddaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b001) => AArch64Instr::LdclraVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b010) => AArch64Instr::LdeoraVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b011) => AArch64Instr::LdsetaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b100) => AArch64Instr::LdsmaxaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b101) => AArch64Instr::LdsminaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b110) => AArch64Instr::LdumaxaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b0, 0b111) => AArch64Instr::LduminaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b1, 0b000) => AArch64Instr::SwpaVar64(data),
                    (0b11, 0b0, 0b1, 0b0, _, 0b1, 0b100) => AArch64Instr::LdaprVar64(data),

                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b000) => AArch64Instr::LdaddalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b001) => AArch64Instr::LdclralVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b010) => AArch64Instr::LdeoralVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b011) => AArch64Instr::LdsetalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b100) => AArch64Instr::LdsmaxalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b101) => AArch64Instr::LdsminalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b110) => AArch64Instr::LdumaxalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b0, 0b111) => AArch64Instr::LduminalVar64(data),
                    (0b11, 0b0, 0b1, 0b1, _, 0b1, 0b000) => AArch64Instr::SwpalVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_add_sub_with_carry(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010000_xxxxx_000000_xxxxx_xxxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RmRnRd {
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match sf_op_s.value {
                    0b000 => AArch64Instr::AdcVar32(data),
                    0b001 => AArch64Instr::AdcsVar32(data),
                    0b010 => AArch64Instr::SbcVar32(data),
                    0b011 => AArch64Instr::SbcsVar32(data),

                    0b100 => AArch64Instr::AdcVar64(data),
                    0b101 => AArch64Instr::AdcsVar64(data),
                    0b110 => AArch64Instr::SbcVar64(data),
                    0b111 => AArch64Instr::SbcsVar64(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_floating_point_compare(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xxxxx_xx_1000_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             op: Extract<BitRange<14, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             opcode2: Extract<BitRange<0, 5>, u8>| {
                let data = FloatingPointCompare {
                    ptype: ptype.value,
                    rm: rm.value,
                    rn: rn.value,
                    opcode2: opcode2.value,
                };

                match (m.value, s.value, ptype.value, op.value, opcode2.value) {
                    (0b0, 0b0, 0b00, 0b00, 0b00000 | 0b01000)
                    | (0b0, 0b0, 0b01, 0b00, 0b00000 | 0b01000)
                    | (0b0, 0b0, 0b11, 0b01, 0b00000 | 0b01000) => AArch64Instr::Fcmp(data),

                    (0b0, 0b0, 0b00, 0b00, 0b10000 | 0b11000)
                    | (0b0, 0b0, 0b01, 0b00, 0b10000 | 0b11000)
                    | (0b0, 0b0, 0b11, 0b01, 0b10000 | 0b11000) => AArch64Instr::Fcmp(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_advanced_simd_permute(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_001110_xx_0_xxxxx_0_xxx_10_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<30, 31>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<12, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = QSizeRmRnRd {
                    q: q.value,
                    size: size.value,
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match opcode.value {
                    0b001 => AArch64Instr::Uzp1(data),
                    0b010 => AArch64Instr::Trn1(data),
                    0b011 => AArch64Instr::Zip1(data),

                    0b101 => AArch64Instr::Uzp2(data),
                    0b110 => AArch64Instr::Trn2(data),
                    0b111 => AArch64Instr::Zip2(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_float_data_proc_2src(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xxxxx_xxxx_10_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             opcode: Extract<BitRange<12, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RmRnRd {
                    rm: rm.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (m.value, s.value, ptype.value, opcode.value) {
                    (0b0, 0b0, 0b00, 0b0000) => AArch64Instr::FmulScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0001) => AArch64Instr::FdivScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0010) => AArch64Instr::FaddScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0011) => AArch64Instr::FsubScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0100) => AArch64Instr::FmaxScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0101) => AArch64Instr::FminScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0110) => AArch64Instr::FmaxnmScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b0111) => AArch64Instr::FminnmScalarSinglePrecisionVar(data),
                    (0b0, 0b0, 0b00, 0b1000) => AArch64Instr::FnmulScalarSinglePrecisionVar(data),

                    (0b0, 0b0, 0b01, 0b0000) => AArch64Instr::FmulScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0001) => AArch64Instr::FdivScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0010) => AArch64Instr::FaddScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0011) => AArch64Instr::FsubScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0100) => AArch64Instr::FmaxScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0101) => AArch64Instr::FminScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0110) => AArch64Instr::FmaxnmScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b0111) => AArch64Instr::FminnmScalarDoublePrecisionVar(data),
                    (0b0, 0b0, 0b01, 0b1000) => AArch64Instr::FnmulScalarDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_floating_point_immediate(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xxxxxxxx_100_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             imm8: Extract<BitRange<13, 21>, u8>,
             imm5: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = FloatingPointImmediate {
                    imm8: imm8.value,
                    rd: rd.value,
                };

                match (m.value, s.value, ptype.value, imm5.value) {
                    (0b0, 0b0, 0b00, 0b00000) => {
                        AArch64Instr::FmovScalarImmSinglePrecisionVar(data)
                    }
                    (0b0, 0b0, 0b01, 0b00000) => {
                        AArch64Instr::FmovScalarImmDoublePrecisionVar(data)
                    }

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_conv_between_float_and_fixed_point(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_0_xx_xxx_xxxxxx_xxxxx_xxxxx",
            |raw_instr: u32,
             sf: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             rmode: Extract<BitRange<19, 21>, u8>,
             opcode: Extract<BitRange<16, 19>, u8>,
             scale: Extract<BitRange<10, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = ConvBetweenFloatAndFixedPoint {
                    scale: scale.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (
                    sf.value,
                    s.value,
                    ptype.value,
                    rmode.value,
                    opcode.value,
                    scale.value,
                ) {
                    (0b0, 0b0, 0b00, 0b00, 0b010, _) => {
                        AArch64Instr::ScvtfScalarFixedPt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b00, 0b011, _) => {
                        AArch64Instr::UcvtfScalarFixedPt32ToSinglePrecision(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b000, _) => {
                        AArch64Instr::FcvtzsScalarFixedPtSinglePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b00, 0b11, 0b001, _) => {
                        AArch64Instr::FcvtzuScalarFixedPtSinglePrecisionTo32(data)
                    }

                    (0b0, 0b0, 0b01, 0b00, 0b010, _) => {
                        AArch64Instr::ScvtfScalarFixedPt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b00, 0b011, _) => {
                        AArch64Instr::UcvtfScalarFixedPt32ToDoublePrecision(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b000, _) => {
                        AArch64Instr::FcvtzsScalarFixedPtDoublePrecisionTo32(data)
                    }
                    (0b0, 0b0, 0b01, 0b11, 0b001, _) => {
                        AArch64Instr::FcvtzuScalarFixedPtDoublePrecisionTo32(data)
                    }

                    (0b1, 0b0, 0b00, 0b00, 0b010, _) => {
                        AArch64Instr::ScvtfScalarFixedPt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b00, 0b011, _) => {
                        AArch64Instr::UcvtfScalarFixedPt64ToSinglePrecision(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b000, _) => {
                        AArch64Instr::FcvtzsScalarFixedPtSinglePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b00, 0b11, 0b001, _) => {
                        AArch64Instr::FcvtzuScalarFixedPtSinglePrecisionTo64(data)
                    }

                    (0b1, 0b0, 0b01, 0b00, 0b010, _) => {
                        AArch64Instr::ScvtfScalarFixedPt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b00, 0b011, _) => {
                        AArch64Instr::UcvtfScalarFixedPt64ToDoublePrecision(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b000, _) => {
                        AArch64Instr::FcvtzsScalarFixedPtDoublePrecisionTo64(data)
                    }
                    (0b1, 0b0, 0b01, 0b11, 0b001, _) => {
                        AArch64Instr::FcvtzuScalarFixedPtDoublePrecisionTo64(data)
                    }

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_floating_point_conditional_select(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_0_x_11110_xx_1_xxxxx_xxxx_11_xxxxx_xxxxx",
            |raw_instr: u32,
             m: Extract<BitRange<31, 32>, u8>,
             s: Extract<BitRange<29, 30>, u8>,
             ptype: Extract<BitRange<22, 24>, u8>,
             rm: Extract<BitRange<16, 21>, u8>,
             cond: Extract<BitRange<12, 16>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = RmCondRnRd {
                    rm: rm.value,
                    cond: cond.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (m.value, s.value, ptype.value) {
                    (0b0, 0b0, 0b00) => AArch64Instr::FcselSinglePrecisionVar(data),
                    (0b0, 0b0, 0b01) => AArch64Instr::FcselDoublePrecisionVar(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_vec_x_indexed_elem(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "0_x_x_01111_xx_x_x_xxxx_xxxx_x_0_xxxxx_xxxxx",
            |raw_instr: u32,
             q: Extract<BitRange<31, 32>, u8>,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             l: Extract<BitRange<21, 22>, u8>,
             m: Extract<BitRange<20, 21>, u8>,
             rm: Extract<BitRange<16, 20>, u8>,
             opcode: Extract<BitRange<12, 16>, u8>,
             h: Extract<BitRange<11, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdXIndexedElem {
                    q: q.value,
                    size: size.value,
                    l: l.value,
                    m: m.value,
                    rm: rm.value,
                    h: h.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b0010) => AArch64Instr::SmlalByElem(data),
                    (0b0, _, 0b0011) => AArch64Instr::SqdmlalByElem(data),
                    (0b0, _, 0b0110) => AArch64Instr::SmlslByElem(data),
                    (0b0, _, 0b0111) => AArch64Instr::SqdmlslByElem(data),
                    (0b0, _, 0b1000) => AArch64Instr::MulByElem(data),
                    (0b0, _, 0b1010) => AArch64Instr::SmullByElem(data),
                    (0b0, _, 0b1011) => AArch64Instr::SqdmullByElem(data),
                    (0b0, _, 0b1100) => AArch64Instr::SqdmulhByElem(data),
                    (0b0, _, 0b1101) => AArch64Instr::SqrdmulhByElem(data),

                    (0b0, 0b10 | 0b11, 0b0001) => AArch64Instr::FmlaByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b0101) => AArch64Instr::FmlsByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b1001) => AArch64Instr::FmulByElemEncoding(data),

                    (0b1, _, 0b0000) => AArch64Instr::MlaByElem(data),
                    (0b1, _, 0b0010) => AArch64Instr::UmlalByElem(data),
                    (0b1, _, 0b0100) => AArch64Instr::MlsByElem(data),
                    (0b1, _, 0b0110) => AArch64Instr::UmlslByElem(data),
                    (0b1, _, 0b1010) => AArch64Instr::UmullByElem(data),

                    (0b1, 0b10 | 0b11, 0b1001) => AArch64Instr::FmulxByElemEncoding(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_adv_simd_scalar_x_indexed_elem(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "01_x_11111_xx_x_x_xxxx_xxxx_x_0_xxxxx_xxxxx",
            |raw_instr: u32,
             u: Extract<BitRange<29, 30>, u8>,
             size: Extract<BitRange<22, 24>, u8>,
             l: Extract<BitRange<21, 22>, u8>,
             m: Extract<BitRange<20, 21>, u8>,
             rm: Extract<BitRange<16, 20>, u8>,
             opcode: Extract<BitRange<12, 16>, u8>,
             h: Extract<BitRange<11, 12>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             rd: Extract<BitRange<0, 5>, u8>| {
                let data = AdvSimdXIndexedElem {
                    q: 0b1,
                    size: size.value,
                    l: l.value,
                    m: m.value,
                    rm: rm.value,
                    h: h.value,
                    rn: rn.value,
                    rd: rd.value,
                };

                match (u.value, size.value, opcode.value) {
                    (0b0, _, 0b0011) => AArch64Instr::SqdmlalByElem(data),
                    (0b0, _, 0b0111) => AArch64Instr::SqdmlslByElem(data),
                    (0b0, _, 0b1011) => AArch64Instr::SqdmullByElem(data),
                    (0b0, _, 0b1100) => AArch64Instr::SqdmulhByElem(data),
                    (0b0, _, 0b1101) => AArch64Instr::SqrdmulhByElem(data),

                    (0b0, 0b10 | 0b11, 0b0001) => AArch64Instr::FmlaByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b0101) => AArch64Instr::FmlsByElemEncoding(data),
                    (0b0, 0b10 | 0b11, 0b1001) => AArch64Instr::FmulByElemEncoding(data),

                    (0b1, 0b10 | 0b11, 0b1001) => AArch64Instr::FmulxByElemEncoding(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_sys_instr_with_reg_arg(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "11010101000000110001_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Rt { rt: rt.value };

                match (crm.value, op2.value) {
                    (0b0000, 0b000) => AArch64Instr::Wfet(data),
                    (0b0000, 0b001) => AArch64Instr::Wfit(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_pstate(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "1101010100000_xxx_0100_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             op1: Extract<BitRange<16, 19>, u8>,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Pstate {
                    op1: op1.value,
                    crm: crm.value,
                    op2: op2.value,
                };

                match (op1.value, op2.value, rt.value) {
                    (0b000, 0b000, 0b11111) => AArch64Instr::Cfinv(data),
                    (0b000, 0b001, 0b11111) => AArch64Instr::Xaflag(data),
                    (0b000, 0b010, 0b11111) => AArch64Instr::Axflag(data),
                    (_, _, 0b11111) => AArch64Instr::MsrImm(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_sys_with_result(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "1101010100000_xxx_0100_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             op1: Extract<BitRange<16, 19>, u8>,
             crn: Extract<BitRange<12, 16>, u8>,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Rt { rt: rt.value };

                match (op1.value, crn.value, crm.value, op2.value) {
                    (0b011, 0b0011, 0b0000, 0b011) => AArch64Instr::Tstart(data),
                    (0b011, 0b0011, 0b0001, 0b011) => AArch64Instr::Ttest(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_sys_instr(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "1101010100000_xxx_0100_xxxx_xxx_xxxxx",
            |raw_instr: u32,
             l: Extract<BitRange<21, 22>, u8>,
             op1: Extract<BitRange<16, 19>, u8>,
             crn: Extract<BitRange<12, 16>, u8>,
             crm: Extract<BitRange<8, 12>, u8>,
             op2: Extract<BitRange<5, 8>, u8>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = SystemInstructions {
                    op1: op1.value,
                    crn: crn.value,
                    crm: crm.value,
                    op2: op2.value,
                    rt: rt.value,
                };

                match l.value {
                    0b0 => AArch64Instr::Sys(data),
                    0b1 => AArch64Instr::Sysl(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_rot_right_into_flags(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010000_xxxxxx_00001_xxxxx_x_xxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             imm6: Extract<BitRange<15, 21>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             o2: Extract<BitRange<4, 5>, u8>,
             mask: Extract<BitRange<0, 4>, u8>| {
                let data = RotateRightIntoFlags {
                    imm6: imm6.value,
                    rn: rn.value,
                    mask: mask.value,
                };

                match (sf_op_s.value, o2.value) {
                    (0b101, 0b0) => AArch64Instr::Rmif(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_eval_into_flags(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "x_x_x_11010000_xxxxxx_x_0010_xxxxx_x_xxxx",
            |raw_instr: u32,
             sf_op_s: Extract<BitRange<29, 32>, u8>,
             opcode2: Extract<BitRange<15, 21>, u8>,
             sz: Extract<BitRange<14, 15>, u8>,
             rn: Extract<BitRange<5, 10>, u8>,
             o3: Extract<BitRange<4, 5>, u8>,
             mask: Extract<BitRange<0, 4>, u8>| {
                let data = Rn { rn: rn.value };

                match (sf_op_s.value, opcode2.value, sz.value, o3.value, mask.value) {
                    (0b001, 0b000000, 0b0, 0b0, 0b1101) => AArch64Instr::SetfVar8(data),
                    (0b001, 0b000000, 0b1, 0b0, 0b1101) => AArch64Instr::SetfVar16(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}

fn parse_load_register_literal(raw_instr: u32) -> AArch64Instr {
    pub static MATCHER: Lazy<BitPatternMatcher<AArch64Instr>> = Lazy::new(|| {
        let mut m = BitPatternMatcher::new();
        m.bind(
            "xx_011_x_00_xxxxxxxxxxxxxxxxxxx_xxxxx",
            |raw_instr: u32,
             opc: Extract<BitRange<30, 32>, u8>,
             v: Extract<BitRange<26, 27>, u8>,
             imm19: Extract<BitRange<5, 24>, u32>,
             rt: Extract<BitRange<0, 5>, u8>| {
                let data = Imm19Rt {
                    imm19: imm19.value,
                    rt: rt.value,
                };

                match (opc.value, v.value) {
                    (0b00, 0b0) => AArch64Instr::LdrLitVar32(data),
                    (0b00, 0b1) => AArch64Instr::LdrLitSimdFPVar32(data),
                    (0b01, 0b0) => AArch64Instr::LdrLitVar64(data),
                    (0b01, 0b1) => AArch64Instr::LdrLitSimdFPVar64(data),
                    (0b10, 0b0) => AArch64Instr::LdrswLit(data),
                    (0b10, 0b1) => AArch64Instr::LdrLitSimdFPVar128(data),
                    (0b11, 0b0) => AArch64Instr::PrfmLit(data),

                    _ => todo!("Unknown instruction {:032b}", raw_instr),
                }
            },
        );

        m
    });

    if let Some(instr) = MATCHER.handle(raw_instr) {
        instr
    } else {
        todo!("Unknown instruction {:032b}", raw_instr);
    }
}
