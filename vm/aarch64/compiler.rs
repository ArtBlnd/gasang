use crate::instruction::*;
use crate::jump_table::{Checkpoint, JumpId, JumpTable};
use crate::register::RegId;
use crate::VmContext;

use machineinstr::aarch64::*;
use machineinstr::MachineInstParser;
use utility::extract_bits16;
use utility::BitReader;

use smallvec::SmallVec;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Deref;

pub fn compile_code(addr: u64, data: &[u8], compiler: &AArch64Compiler, vm_ctx: &mut VmContext) {
    // Compile instructions into VMIR and insert it to context.
    let mut ipr = addr;
    let parser =
        MachineInstParser::new(BitReader::new(data.iter().cloned()), AArch64InstrParserRule);

    let mut ip_lookup_table = IprIpvTable::new();

    // construct jump table.
    let mut prev_size = 0u8;
    for native_instr in parser {
        if ip_lookup_table.check_ipr(ipr) {
            //IPV of jump instruction
            let jump_instr_ipv = ip_lookup_table.pop().unwrap();
            let instr = VmIr::from_ref(&vm_ctx.vm_instr[jump_instr_ipv..]);

            let mut rewriter = JumpRewriter::new(
                vm_ctx.vm_instr.len(),
                instr.real_size(),
                instr.curr_size(),
                instr.prev_size(),
            );

            instr.visit(&mut rewriter);
            let beg = vm_ctx.vm_instr.len();
            let end = vm_ctx.vm_instr.len() + instr.curr_size() as usize;
            vm_ctx.vm_instr.as_mut_slice()[beg..end].copy_from_slice(&rewriter.finish());
        }

        let instr = compiler.compile_instr(native_instr.size, prev_size, native_instr.op);
        vm_ctx.insert_instr(&instr);

        let mut is_jump = IsJump(false);
        VmIr::from_ref(&instr).visit(&mut is_jump);
        if is_jump.0 == true {
            ip_lookup_table.push(ipr, vm_ctx.vm_instr.len())
        }

        ipr += native_instr.size as u64;
        prev_size = instr.len() as u8;
    }
}

struct IsJump(bool);
impl InstrVisitor for IsJump {
    fn visit_i32(&mut self, op: u8, _operand: I32) {
        self.0 = op == IROP_BR_IPR_REL32
    }
}

struct JumpRewriter {
    out: SmallVec<[u8; 16]>,
    curr_size: u8,
    target_ipv: usize,
}

impl JumpRewriter {
    pub fn new(target_ipv: usize, orgn_size: u8, curr_size: u8, prev_size: u8) -> Self {
        let mut out = SmallVec::new();
        build_instr_sig(&mut out, orgn_size, curr_size, prev_size);

        Self {
            out,
            curr_size,
            target_ipv,
        }
    }

    pub fn finish(self) -> SmallVec<[u8; 16]> {
        let mut out = self.out;
        out.resize(self.curr_size as usize, IROP_NOP); // Fill with nop.
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
    fn visit_reg1u8(&mut self, op: u8, operand: Reg1U8) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg2u8(&mut self, op: u8, operand: Reg2U8) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1u32(&mut self, op: u8, operand: Reg1U32) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1i32(&mut self, op: u8, operand: Reg1I32) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1u64(&mut self, op: u8, operand: Reg1U64) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1u16(&mut self, op: u8, operand: Reg1U16) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_u16(&mut self, op: u8, operand: U16) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_i32(&mut self, op: u8, operand: I32) {
        match op {
            IROP_BR_IPR_REL32 => {
                todo!()
            }
            _ => self.out.extend_from_slice(&operand.build(op)),
        }
    }
    fn visit_u32(&mut self, op: u8, operand: U32) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg2i64(&mut self, op: u8, operand: Reg2I64) {
        self.out.extend_from_slice(&operand.build(op))
    }
    fn visit_reg1i64(&mut self, op: u8, operand: Reg1I64) {
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
}

impl AArch64Compiler {
    pub fn new(
        gpr_registers: [RegId; 32],
        fpr_registers: [RegId; 32],
        stack_reg: RegId,
        pstate_reg: RegId,
    ) -> Self {
        Self {
            gpr_registers,
            fpr_registers,
            stack_reg,
            pstate_reg,
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
                let op = Reg1U16 {
                    op1: self.gpr(operand.rd),
                    imm16: operand.imm16,
                }
                .build(IROP_MOV_16CST2REG);

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
                .build(IROP_MOV_IPR2REG);

                let op2 = Reg1I32 {
                    op1: self.gpr(operand.rd),
                    imm32: imm as i32,
                }
                .build(IROP_IADD_CST32);

                let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op1);
                out.extend_from_slice(&op2);
            }

            AArch64Instr::OrrShiftedReg64(operand) => {
                let rm = self.gpr(operand.rm);
                let rn = self.gpr(operand.rn);
                let rd = self.gpr(operand.rd);

                if operand.imm6 == 0 && operand.shift == 0 && operand.rn == 0b11111 {
                    let op = Reg2 { op1: rm, op2: rd }.build(IROP_MOV_REG2REG);

                    let curr_size = 2 + op.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op);
                } else {
                    let i1 = match decode_shift(operand.shift) {
                        ShiftType::LSL => IROP_LLEFT_SHIFT_IMM8,
                        ShiftType::LSR => IROP_LRIGHT_SHIFT_IMM8,
                        ShiftType::ASR => IROP_ARIGHT_SHIFT_IMM8,
                        ShiftType::ROR => IROP_ROTATE_IMM8,
                    };

                    let op1 = Reg2U8 {
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
                    .build(IROP_OR_REG3);

                    let curr_size = 2 + op1.len() as u8 + op2.len() as u8;
                    build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                    out.extend_from_slice(&op1);
                    out.extend_from_slice(&op2);
                }
            }

            AArch64Instr::Svc(operand) => {
                let op = U16 {
                    imm16: operand.imm16,
                }
                .build(IROP_SVC);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::Brk(operand) => {
                let op = U16 {
                    imm16: operand.imm16,
                }
                .build(IROP_BRK);

                let curr_size = 2 + op.len() as u8;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
            }

            AArch64Instr::LdrImm64(operand) => {
                let (mut wback, _post_index, _scale, offset) =
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
                    };

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

                let mut ops = SmallVec::<[u8; 16]>::new();

                let v = Reg2I16 {
                    op1: src,
                    op2: dst,
                    imm16: offset,
                }
                .build(IROP_SLOAD_REG2REG);
                ops.extend_from_slice(&v);

                if wback {
                    let w = Reg1I32 {
                        op1: src,
                        imm32: offset as i32,
                    }
                    .build(IROP_IADD_CST32);
                    ops.extend_from_slice(&w);
                }

                let curr_size = out.len() as u8 + 2;
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

                let op = Reg2U32 {
                    op1: src,
                    op2: dst,
                    imm32: imm,
                }
                .build(IROP_UADD_CST32);

                let curr_size = op.len() as u8 + 2;
                build_instr_sig(&mut out, orgn_size, curr_size, prev_size);
                out.extend_from_slice(&op);
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

const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

const fn add_with_carry(x: u64, y: u64, carry_in: u64) -> (u64, u64) {
    let unsigned_sum = x + y + carry_in;
    let signed_sum = x as i64 + y as i64 + carry_in as i64;
    let result = unsigned_sum;

    let n = result & 0x8000000000000000 >> 63;
    let z = if result == 0 { 1 } else { 0 };
    let c = if result == unsigned_sum { 0 } else { 1 };
    let v = if (result as i64) == signed_sum { 0 } else { 1 };

    (result, n << 3 | z << 2 | c << 1 | v)
}
