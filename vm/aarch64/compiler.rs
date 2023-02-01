use crate::instruction::*;
use crate::jump_table::JumpId;
use crate::register::RegId;
use crate::*;

use machineinstr::aarch64::{AArch64Instr, AArch64InstrParserRule, SizeImm12RnRt};
use machineinstr::MachineInstParser;
use utility::extract_bits16;
use utility::BitReader;

use smallvec::SmallVec;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub fn compile_code(addr: u64, data: &[u8], compiler: &AArch64Compiler, vm_ctx: &mut VmContext) {
    // Compile instructions into VMIR and insert it to context.
    let mut ipr = addr;
    let ipv = vm_ctx.vm_instr.len();

    let parser =
        MachineInstParser::new(BitReader::new(data.iter().cloned()), AArch64InstrParserRule);

    let mut ip_lookup_table = IprIpvTable::new();

    // construct jump table.
    let mut prev_size = 0u8;
    let mut prev_check_point = ipr;
    let min_distance = vm_ctx.jump_table.checkpoint_min_distance();

    for native_instr in parser {
        // Constructing checkpoint table for "Jump to Register" and "Jump to relative" instructions,
        // which couldn't know destination during compile time
        if prev_check_point / min_distance < ipr / min_distance {
            let ipv = vm_ctx.vm_instr.len();
            vm_ctx.jump_table.new_checkpoint(ipr, ipv);
            prev_check_point = ipr;
        }

        if ip_lookup_table.check_ipr(ipr) {
            //IPV of jump instruction
            let jump_instr_ipv = ip_lookup_table.pop().unwrap();
            let instr = VmIr::from_ref(&vm_ctx.vm_instr[jump_instr_ipv..]);

            //Rewrite operand with found ipv
            let mut rewriter = JumpRewriter::new(
                vm_ctx.jump_table.new_jump(ipv),
                instr.real_size(),
                instr.curr_size(),
                instr.prev_size(),
            );

            instr.visit(&mut rewriter);
            let beg = ipv;
            let end = ipv + instr.curr_size() as usize;
            vm_ctx.vm_instr.as_mut_slice()[beg..end].copy_from_slice(&rewriter.finish());
        }

        let instr = compiler.compile_instr(native_instr.size, prev_size, native_instr.op);
        vm_ctx.insert_instr(&instr);

        let mut is_jump = IsJump(false);
        let vmir = VmIr::from_ref(&instr);
        println!("{vmir}");
        vmir.visit(&mut is_jump);
        //If current ir including jump instruction push ipr-ipv pair to table
        if is_jump.0 == true {
            ip_lookup_table.push(ipr, ipv)
        }

        ipr += native_instr.size as u64;
        prev_size = instr.len() as u8;
    }
}

struct IsJump(bool);
impl InstrVisitor for IsJump {
    fn visit_u32(&mut self, op: u8, _operand: Imm32) {
        //self.0 = op == BR_IPR_IMM32_REL
    }
}

struct JumpRewriter {
    out: SmallVec<[u8; 16]>,
    curr_size: u8,

    target: JumpId,
}

impl JumpRewriter {
    pub fn new(target: JumpId, orgn_size: u8, curr_size: u8, prev_size: u8) -> Self {
        let mut out = SmallVec::new();
        build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
        Self {
            out,
            curr_size,
            target,
        }
    }

    pub fn finish(self) -> SmallVec<[u8; 16]> {
        let mut out = self.out;
        out.resize(self.curr_size as usize, NOP); // Fill with nop.
        out
    }
}

impl InstrVisitor for JumpRewriter {
    fn visit_no_operand(&mut self, op: u8) {
        self.out.push(op);
    }
    fn visit_reg1(&mut self, op: u8, operand: Reg1) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg3(&mut self, op: u8, operand: Reg3) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1imm8(&mut self, op: u8, operand: Reg1Imm8) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg2imm8(&mut self, op: u8, operand: Reg2Imm8) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1imm32(&mut self, op: u8, operand: Reg1Imm32) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1imm64(&mut self, op: u8, operand: Reg1Imm64) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1imm16(&mut self, op: u8, operand: Reg1Imm16) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_u16(&mut self, op: u8, operand: Imm16) {
        match op {
            BR_IPR_IMM32_REL => {
                let op = Imm32 {
                    imm32: self.target.0,
                };

                self.out.extend_from_slice(&op.build(BR_IPV_IMM32));
            }
            _ => self.out.extend_from_slice(&operand.build(op)),
        }
    }
    fn visit_u32(&mut self, op: u8, operand: Imm32) {
        self.out.extend_from_slice(&operand.build(op))
    }
}

#[derive(PartialEq, PartialOrd, Eq)]
struct IprIpvPair(u64, usize);

impl Ord for IprIpvPair {
    fn cmp(&self, other: &Self) -> Ordering {
        //For min-heap
        match self.0.cmp(&other.0) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
        }
    }
}

struct IprIpvTable {
    b_heap: BinaryHeap<IprIpvPair>,
}

impl IprIpvTable {
    pub fn new() -> Self {
        Self {
            b_heap: BinaryHeap::new(),
        }
    }
    pub fn push(&mut self, ipr: u64, ipv: usize) {
        self.b_heap.push(IprIpvPair(ipr, ipv))
    }

    pub fn check_ipr(&self, ipr: u64) -> bool {
        let Some(peek) = self.b_heap.peek() else {
            return false;
        };

        peek.0 == ipr
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.b_heap.pop().map(|v| v.1)
    }
}

pub struct AArch64Compiler {
    gpr_registers: [RegId; 32],
    fpr_registers: [RegId; 32],

    stack_reg: RegId,
    pstate_reg: RegId,
    btype_next: RegId,
}

impl AArch64Compiler {
    pub fn new(
        gpr_registers: [RegId; 32],
        fpr_registers: [RegId; 32],
        stack_reg: RegId,
        pstate_reg: RegId,
        btype_next: RegId,
    ) -> Self {
        Self {
            gpr_registers,
            fpr_registers,
            stack_reg,
            pstate_reg,
            btype_next,
        }
    }

    pub fn gpr(&self, reg: u8) -> RegId {
        self.gpr_registers[reg as usize]
    }

    pub fn fpr(&self, reg: u8) -> RegId {
        self.fpr_registers[reg as usize]
    }

    pub fn compile_instr(
        &self,
        orgn_size: u8, // original instruction size
        prev_size: u8, // previous instruction size
        instr: AArch64Instr,
    ) -> SmallVec<[u8; 16]> {
        let mut out = SmallVec::new();

        match instr {
            AArch64Instr::MovzVar32(operand) | AArch64Instr::MovzVar64(operand) => {
                let op = Reg1Imm16 {
                    op1: self.gpr(operand.rd),
                    imm16: operand.imm16,
                }
                .build(MOV_REG1IMM16);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Nop => {
                let curr_size = 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
            }

            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let op1 = Reg1 {
                    op1: self.gpr(operand.rd),
                }
                .build(MOV_IPR_REG);

                let op2 = Reg2Imm32 {
                    op1: self.gpr(operand.rd),
                    op2: self.gpr(operand.rd),
                    imm32: imm as u32,
                }
                .build(IADD_REG2IMM32);

                let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
            }

            AArch64Instr::StpVar64(operand) => {
                // If rn is 31 we use stack register instead.
                let rn = if operand.rn == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let rt1 = self.gpr(operand.rt);
                let rt2 = self.gpr(operand.rt2);

                let signed_offs = operand.o == 010;
                let (wback, post_index) = match operand.o {
                    0b001 => (true, true),
                    0b011 => (true, false),
                    0b010 => (false, false),
                    _ => unreachable!("Invalid stp64 options {:03b}", operand.o),
                };

                if rt1 == rn || rt2 == rn {
                    unreachable!("Bad stp64 instruction");
                }

                let offs = if !post_index { operand.imm7 as i32 } else { 0 };

                let (store_instr, offs1, offs2) = if signed_offs {
                    let offs2 = (offs + 8) as u32;
                    (SSTORE_REL_REG2IMM32, offs as u32, offs2)
                } else {
                    (USTORE_REL_REG2IMM32, offs as u32, offs as u32 + 8)
                };

                let op1 = Reg2Imm32 {
                    op1: rt1,
                    op2: rn,
                    imm32: offs1,
                }
                .build(store_instr);

                let op2 = Reg2Imm32 {
                    op1: rt2,
                    op2: rn,
                    imm32: offs2,
                }
                .build(store_instr);

                if post_index && wback {
                    let op3 = Reg2Imm8 {
                        op1: rn,
                        op2: rn,
                        imm8: operand.imm7 as u8,
                    }
                    .build(UADD_REG2IMM8);

                    let curr_size = 2 + op1.len() as u8 + op2.len() as u8 + op3.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op1);
                    out.extend_from_slice(&op2);
                    out.extend_from_slice(&op3);
                } else {
                    let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op1);
                    out.extend_from_slice(&op2);
                }
            }

            AArch64Instr::OrrShiftedReg64(operand) | AArch64Instr::OrrShiftedReg32(operand) => {
                let rm = self.gpr(operand.rm);
                let rn = self.gpr(operand.rn);
                let rd = self.gpr(operand.rd);

                if operand.imm6 == 0 && operand.shift == 0 && operand.rn == 0b11111 {
                    let op = Reg2 { op1: rm, op2: rd }.build(MOV_REG2);

                    let curr_size = 2 + op.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op);
                } else {
                    let i1 = match decode_shift(operand.shift) {
                        ShiftType::LSL => LSHL_REG2IMM8,
                        ShiftType::LSR => LSHR_REG2IMM8,
                        ShiftType::ASR => ASHR_REG2IMM8,
                        ShiftType::ROR => RROT_REG2IMM8,
                    };

                    let op1 = Reg2Imm8 {
                        op1: rm,
                        op2: rd,
                        imm8: operand.imm6,
                    }
                    .build(i1);

                    let op2 = Reg3 {
                        op1: rd,
                        op2: rd,
                        op3: rn,
                    }
                    .build(OR_REG3);

                    let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op1);
                    out.extend_from_slice(&op2);
                }
            }

            AArch64Instr::Svc(operand) => {
                let op = Imm16 {
                    imm16: operand.imm16,
                }
                .build(SVC_IMM16);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Brk(operand) => {
                let op = Imm16 {
                    imm16: operand.imm16,
                }
                .build(BRK_IMM16);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::LdrImm64(operand) => {
                let (mut wback, post_index, _scale, offset) = decode_operand_for_ld_st(operand);

                if wback && operand.rn == operand.rt && operand.rn != 31 {
                    wback = false;
                }

                let src = if operand.rn == 31 {
                    // If rn is 31, we use stack register instead of gpr registers.
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };
                let dst = self.gpr(operand.rt);

                let offset_temp = if !post_index { offset } else { 0 };

                let mut ops = SmallVec::<[u8; 16]>::new();

                let v = Reg2Imm32 {
                    op1: src,
                    op2: dst,
                    imm32: offset_temp as u32,
                }
                .build(SLOAD_REL_REG2IMM32);
                ops.extend_from_slice(&v);

                let curr_size = ops.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&ops);
            }

            AArch64Instr::StrImm64(operand) => {
                let (wback, post_index, _scale, offset) = decode_operand_for_ld_st(operand);

                let src = self.gpr(operand.rt);

                let dst = if operand.rn == 31 {
                    // If rn is 31, we use stack register instead of gpr registers.
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let offset_temp = if !post_index { offset } else { 0 };

                let mut ops = SmallVec::<[u8; 16]>::new();

                let op = Reg2Imm32 {
                    op1: src,
                    op2: dst,
                    imm32: offset_temp as u32,
                }
                .build(SSTORE_REL_REG2IMM32);
                ops.extend_from_slice(&op);

                if wback {
                    let op = Reg2Imm32 {
                        op1: dst,
                        op2: dst,
                        imm32: offset as u32,
                    }
                    .build(IADD_REG2IMM32);
                    ops.extend_from_slice(&op);
                }

                let curr_size = ops.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&ops);
            }

            AArch64Instr::LdrbImm(operand) => {
                let (wback, post_index, _, offset) = decode_operand_for_ld_st(operand);

                let src = if operand.rn == 31 {
                    // If rn is 31, we use stack register instead of gpr registers.
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };
                let dst = self.gpr(operand.rt);

                let mut ops = SmallVec::<[u8; 16]>::new();

                let offset_temp = if !post_index { offset } else { 0 };

                let op1 = Reg2Imm32 {
                    op1: src,
                    op2: dst,
                    imm32: offset_temp as u32,
                }
                .build(SLOAD_REL_REG2IMM32);
                ops.extend_from_slice(&op1);

                let op2 = Reg2Imm64 {
                    op1: dst,
                    op2: dst,
                    imm64: 0x0000_0000_0000_00FF,
                }
                .build(AND_REG2IMM64);
                ops.extend_from_slice(&op2);

                if wback {
                    let op = Reg2Imm32 {
                        op1: src,
                        op2: src,
                        imm32: offset as u32,
                    }
                    .build(IADD_REG2IMM32);
                    ops.extend_from_slice(&op);
                }

                let curr_size = ops.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&ops);
            }

            AArch64Instr::AddImm64(operand) => {
                let imm = (operand.imm12 as u32) << (operand.sh * 12);

                let src = if operand.rn == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let dst = if operand.rd == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let op = Reg2Imm32 {
                    op1: src,
                    op2: dst,
                    imm32: imm,
                }
                .build(UADD_REG2IMM32);

                let curr_size = op.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::SubImm64(operand) => {
                let imm = !((operand.imm12 as u32) << (operand.sh * 12)) + 1;

                let src = if operand.rn == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let dst = if operand.rd == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let op = Reg2Imm32 {
                    op1: src,
                    op2: dst,
                    imm32: imm,
                }
                .build(IADD_REG2IMM32);

                let curr_size = op.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::BlImm(operand) => {
                let x30 = self.gpr(30);
                let op1 = Reg1 { op1: x30 }.build(MOV_IPR_REG);

                let op2 = Reg2Imm64 {
                    op1: x30,
                    op2: x30,
                    imm64: 4,
                }
                .build(UADD_REG2IMM64);

                let op3 = Imm32 {
                    imm32: sign_extend((operand.imm26 << 2) as i64, 28) as u32,
                }
                .build(BR_IPR_IMM32_REL);

                let curr_size = (op1.len() + op2.len() + op3.len() + 2) as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
                out.extend_from_slice(&op3);
            }

            AArch64Instr::BImm(operand) => {
                let op = Imm32 {
                    imm32: sign_extend((operand.imm26 << 2) as i64, 28) as u32,
                }
                .build(BR_IPR_IMM32_REL);

                let curr_size = (op.len() + 2) as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Adrp(operand) => {
                let imm = sign_extend(
                    (operand.immhi << 14 | (operand.immlo as u32) << 12) as i64,
                    33,
                );
                let rd = self.gpr(operand.rd);

                let op1 = Reg1 { op1: rd }.build(MOV_IPR_REG);

                let op2 = Reg2Imm64 {
                    op1: rd,
                    op2: rd,
                    imm64: 0xFFFF_FFFF_FFFF_F000,
                }
                .build(AND_REG2IMM64);

                let op3 = Reg2Imm64 {
                    op1: rd,
                    op2: rd,
                    imm64: imm as u64,
                }
                .build(IADD_REG2IMM64);

                let curr_size = (op1.len() + op2.len() + op3.len() + 2) as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
                out.extend_from_slice(&op3);
            }

            AArch64Instr::Cbz64(operand) => {
                let offset = operand.imm19 << 2;

                let op1 = Reg1Slot1 {
                    op1: self.gpr(operand.rt),
                    slot_id: SLOT0,
                }
                .build(STORE_SLOT_REG);

                let op2 = SlotImm32 {
                    slot_id: SLOT0,
                    imm32: offset,
                }
                .build(BR_IPR_IMM32_REL_IF_SLOT_ZERO);

                let curr_size = op1.len() + op2.len() + 2;
                build_instr_sig(&mut out, orgn_size, curr_size as u8, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
            }

            AArch64Instr::Ret(operand) => {
                let op1 = Reg1Imm16 {
                    op1: self.btype_next,
                    imm16: 0b00,
                }
                .build(MOV_REG1IMM16);

                let op2 = Reg1 {
                    op1: self.gpr(operand.rn),
                }
                .build(BR_IPR_REG1);

                let curr_size = op1.len() + op2.len() + 2;
                build_instr_sig(&mut out, orgn_size, curr_size as u8, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
            }

            AArch64Instr::Sbfm64(operand) => {
                let immr = operand.immr;
                let imms = operand.imms;

                let (wmask, tmask) = decode_bit_masks(operand.n, imms, immr, false, 64);

                let src = self.gpr(operand.rn);
                let dst = self.gpr(operand.rd);

                let op1 = Reg2Imm8 {
                    op1: src,
                    op2: dst,
                    imm8: immr,
                }
                .build(RROT_REG2IMM8);

                let op2 = Reg2Imm64 {
                    op1: dst,
                    op2: dst,
                    imm64: wmask,
                }
                .build(AND_REG2IMM64);

                let op3 = Reg2Imm64 {
                    op1: dst,
                    op2: dst,
                    imm64: tmask,
                }
                .build(AND_REG2IMM64);

                let op4 = Reg1Slot1 {
                    op1: dst,
                    slot_id: SLOT0,
                }
                .build(STORE_SLOT_REG); // SLOT0 == bot & tmask

                let op5 = Reg2Imm8 {
                    op1: src,
                    op2: dst,
                    imm8: imms,
                }
                .build(MOV_BIT_REG2IMM8);

                let op6 = Reg2Imm16 {
                    op1: dst,
                    op2: dst,
                    imm16: 64 << 8 | 1,
                }
                .build(REPL_REG2IMM16); // top

                let op7 = Reg2Imm64 {
                    op1: dst,
                    op2: dst,
                    imm64: !tmask,
                }
                .build(AND_REG2IMM64);

                let op8 = Reg1Slot1 {
                    op1: dst,
                    slot_id: SLOT0,
                }
                .build(OR_REG1SLOT1);

                let curr_size = op1.len()
                    + op2.len()
                    + op3.len()
                    + op4.len()
                    + op5.len()
                    + op6.len()
                    + op7.len()
                    + op8.len()
                    + 2;

                build_instr_sig(&mut out, orgn_size, curr_size as u8, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
                out.extend_from_slice(&op3);
                out.extend_from_slice(&op4);
                out.extend_from_slice(&op5);
                out.extend_from_slice(&op6);
                out.extend_from_slice(&op7);
                out.extend_from_slice(&op8);
            }

            _ => unimplemented!("unknown instruction: {:?}", instr),
        }

        out
    }
}

enum ShiftType {
    LSL, // Logical shift left
    LSR, // Logical shift right
    ASR, // Arithmetic shift right
    ROR, // Rotate right
}

const fn decode_shift(shift: u8) -> ShiftType {
    match shift {
        0b00 => ShiftType::LSL,
        0b01 => ShiftType::LSR,
        0b10 => ShiftType::ASR,
        0b11 => ShiftType::ROR,
        _ => unreachable!(),
    }
}

const fn highest_set_bit(x: u64) -> u64 {
    63 - x.leading_zeros() as u64
}

const fn ones(n: u64) -> u64 {
    replicate(1, n, 1)
}

const fn ror(x: u64, shift: u64, size: u64) -> u64 {
    let m = shift % size;
    x >> m | ((x << (size - m)) & ones(shift))
}

const fn replicate(x: u64, n: u64, size: u64) -> u64 {
    let mut result = 0b0;
    let mut i = n;

    while i > 0 {
        result |= x;
        result <<= size;
        i -= 1;
    }

    result
}

const fn decode_bit_masks(immn: u8, imms: u8, immr: u8, immediate: bool, m: u8) -> (u64, u64) {
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
    let diff = s - r;

    let esize = 1 << len;
    let d = extract_bits16(0..len as usize, diff as u16);

    let welem = ones(s as u64 + 1);
    let telem = ones(d as u64 + 1);

    let wmask = replicate(ror(welem, r as u64, esize), m as u64, esize);
    let tmask = replicate(telem, m as u64, esize);

    (wmask, tmask)
}

const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

const fn decode_operand_for_ld_st(operand: SizeImm12RnRt) -> (bool, bool, u8, i16) {
    if extract_bits16(11..12, operand.imm12) == 0b0 {
        let imm9 = extract_bits16(2..11, operand.imm12) as i64;
        let post = extract_bits16(0..2, operand.imm12) == 0b01;

        (true, post, operand.size, sign_extend(imm9, 9) as i16)
    } else {
        //Unsigned offset
        (
            false,
            false,
            operand.size,
            (operand.imm12 << operand.size) as i16,
        )
    }
}
