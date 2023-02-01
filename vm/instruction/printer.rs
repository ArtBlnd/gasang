use crate::instruction::*;
use smallvec::SmallVec;

pub struct InstrPrinter(SmallVec<[String; 4]>);

impl InstrPrinter {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn into_inner(self) -> SmallVec<[String; 4]> {
        self.0
    }
}

impl InstrVisitor for InstrPrinter {
    fn visit_no_operand(&mut self, op: u8) {
        self.0.push(match op {
            NOP => "nop".to_string(),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1(&mut self, op: u8, operand: Reg1) {
        self.0.push(match op {
            MOV_IPR_REG => format!("mov ipr, {}", operand.op1),
            POP_REG => format!("pop {}", operand.op1),
            PSH_REG => format!("psh {}", operand.op1),
            BR_IPR_REG1 => format!("jmp_ipr {}", operand.op1),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        self.0.push(match op {
            MOV_REG2 => format!("mov {}, {}", operand.op1, operand.op2),
            NOT_REG2 => format!("not {}, {}", operand.op1, operand.op2),
            _ => unimplemented!(),
        });
    }

    fn visit_reg3(&mut self, op: u8, operand: Reg3) {
        self.0.push(match op {
            UADD_REG3 => format!("uadd {}, {}, {}", operand.op1, operand.op2, operand.op3),
            USUB_REG3 => format!("usub {}, {}, {}", operand.op1, operand.op2, operand.op3),
            UMUL_REG3 => format!("umul {}, {}, {}", operand.op1, operand.op2, operand.op3),
            UDIV_REG3 => format!("udiv {}, {}, {}", operand.op1, operand.op2, operand.op3),
            OR_REG3 => format!("or {}, {}, {}", operand.op1, operand.op2, operand.op3),
            AND_REG3 => format!("and {}, {}, {}", operand.op1, operand.op2, operand.op3),
            XOR_REG3 => format!("xor {}, {}, {}", operand.op1, operand.op2, operand.op3),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1imm8(&mut self, op: u8, operand: Reg1Imm8) {
        self.0.push(match op {
            _ => unimplemented!(),
        });
    }

    fn visit_reg1imm16(&mut self, op: u8, operand: Reg1Imm16) {
        self.0.push(match op {
            MOV_REG1IMM16 => format!("mov {}, {}", operand.op1, operand.imm16),
            _ => unimplemented!(),
        });
    }
    fn visit_reg1imm32(&mut self, op: u8, operand: Reg1Imm32) {}

    fn visit_reg1imm64(&mut self, op: u8, operand: Reg1Imm64) {
        self.0.push(match op {
            MOV_REG1IMM64 => format!("mov {}, {}", operand.imm64, operand.op1),
            ULOAD_REG1IMM64 => format!("uload {}, *({})", operand.op1, operand.imm64),
            USTORE_REG1IMM64 => format!("ustore *({}), {}", operand.op1, operand.imm64),
            NOT_REG1IMM64 => format!("not {}, {}", operand.op1, operand.imm64),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2imm8(&mut self, op: u8, operand: Reg2Imm8) {
        self.0.push(match op {
            LSHL_REG2IMM8 => format!("lshl {}, {} {}", operand.op1, operand.op2, operand.u8()),
            LSHR_REG2IMM8 => format!("lshr {}, {} {}", operand.op1, operand.op2, operand.u8()),
            RROT_REG2IMM8 => format!("rrot {}, {} {}", operand.op1, operand.op2, operand.u8()),
            ASHR_REG2IMM8 => format!("ashr {}, {} {}", operand.op1, operand.op2, operand.u8()),
            UADD_REG2IMM8 => format!("uadd {}, {} {}", operand.op1, operand.op2, operand.u8()),
            USUB_REG2IMM8 => format!("usub {}, {} {}", operand.op1, operand.op2, operand.u8()),
            UMUL_REG2IMM8 => format!("umul {}, {} {}", operand.op1, operand.op2, operand.u8()),
            UDIV_REG2IMM8 => format!("udiv {}, {} {}", operand.op1, operand.op2, operand.u8()),
            MOV_BIT_REG2IMM8 => format!("mov {}<{}>, {}", operand.op1, operand.u8(), operand.op2),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2imm16(&mut self, op: u8, operand: Reg2Imm16) {
        self.0.push(match op {
            REPL_REG2IMM16 => format!("repl {}, {} {}", operand.op1, operand.op2, operand.u16()),
            _ => unimplemented!(),
        })
    }

    fn visit_reg2imm32(&mut self, op: u8, operand: Reg2Imm32) {
        self.0.push(match op {
            SSTORE_REL_REG2IMM32 => format!(
                "store {}, *({} + {})",
                operand.op2,
                operand.op1,
                operand.i32()
            ),
            SLOAD_REL_REG2IMM32 => format!(
                "load *({} + {}), {}",
                operand.op1,
                operand.i32(),
                operand.op2
            ),
            USTORE_REL_REG2IMM32 => format!(
                "store {}, *({} + {})",
                operand.op2,
                operand.op1,
                operand.u32()
            ),
            ULOAD_REL_REG2IMM32 => format!(
                "load *({} + {}), {}",
                operand.op1,
                operand.u32(),
                operand.op2
            ),
            UADD_REG2IMM32 => format!("uadd {}, {} {}", operand.op1, operand.op2, operand.u32()),
            USUB_REG2IMM32 => format!("usub {}, {} {}", operand.op1, operand.op2, operand.u32()),
            UMUL_REG2IMM32 => format!("umul {}, {} {}", operand.op1, operand.op2, operand.u32()),
            UDIV_REG2IMM32 => format!("udiv {}, {} {}", operand.op1, operand.op2, operand.u32()),
            IADD_REG2IMM32 => format!("iadd {}, {} {}", operand.op1, operand.op2, operand.i32()),
            ISUB_REG2IMM32 => format!("isub {}, {} {}", operand.op1, operand.op2, operand.i32()),
            IMUL_REG2IMM32 => format!("imul {}, {} {}", operand.op1, operand.op2, operand.i32()),
            IDIV_REG2IMM32 => format!("idiv {}, {} {}", operand.op1, operand.op2, operand.i32()),

            _ => unimplemented!(),
        })
    }

    fn visit_reg2imm64(&mut self, op: u8, operand: Reg2Imm64) {
        self.0.push(match op {
            UADD_REG2IMM64 => format!("uadd {}, {} {}", operand.op1, operand.op2, operand.u64()),
            USUB_REG2IMM64 => format!("usub {}, {} {}", operand.op1, operand.op2, operand.u64()),
            UMUL_REG2IMM64 => format!("umul {}, {} {}", operand.op1, operand.op2, operand.u64()),
            UDIV_REG2IMM64 => format!("udiv {}, {} {}", operand.op1, operand.op2, operand.u64()),

            IADD_REG2IMM64 => format!("iadd {}, {} {}", operand.op1, operand.op2, operand.u64()),
            ISUB_REG2IMM64 => format!("isub {}, {} {}", operand.op1, operand.op2, operand.u64()),
            IMUL_REG2IMM64 => format!("imul {}, {} {}", operand.op1, operand.op2, operand.u64()),
            IDIV_REG2IMM64 => format!("idiv {}, {} {}", operand.op1, operand.op2, operand.u64()),

            OR_REG2IMM64 => format!("or {}, {} {}", operand.op1, operand.op2, operand.u64()),
            AND_REG2IMM64 => format!("and {}, {} {}", operand.op1, operand.op2, operand.u64()),
            XOR_REG2IMM64 => format!("xor {}, {} {}", operand.op1, operand.op2, operand.u64()),
            _ => unimplemented!(),
        });
    }

    fn visit_u16(&mut self, op: u8, operand: Imm16) {
        self.0.push(match op {
            SVC_IMM16 => format!("svc {}", operand.imm16),
            BRK_IMM16 => format!("brk {}", operand.imm16),

            _ => unimplemented!(),
        });
    }

    fn visit_u32(&mut self, op: u8, operand: Imm32) {
        self.0.push(match op {
            BR_IPV_IMM32 => format!("jmp_ipv {}", operand.imm32),
            BR_IPR_IMM32 => format!("jmp_ipr {}", operand.imm32),
            BR_IPR_IMM32_REL => format!("jmp_ipr {}", operand.imm32),
            _ => unimplemented!(),
        })
    }

    fn visit_slot1imm32(&mut self, op: u8, operand: SlotImm32) {
        self.0.push(match op {
            BR_IPR_IMM32_REL_IF_SLOT_ZERO => format!(
                "jmp_ipr {} if slot[{}] == 0",
                operand.imm32, operand.slot_id.0
            ),

            _ => unimplemented!(),
        })
    }

    fn visit_reg1slot1(&mut self, op: u8, operand: Reg1Slot1) {
        self.0.push(match op {
            LOAD_SLOT_REG => format!("mov slot[{}], {}", operand.slot_id.0, operand.op1),
            STORE_SLOT_REG => format!("mov {}, slot[{}]", operand.op1, operand.slot_id.0),
            UADD_REG1SLOT1 => format!("uadd {}, slot[{}]", operand.op1, operand.slot_id.0),
            USUB_REG1SLOT1 => format!("usub {}, slot[{}]", operand.op1, operand.slot_id.0),
            UMUL_REG1SLOT1 => format!("umul {}, slot[{}]", operand.op1, operand.slot_id.0),
            UDIV_REG1SLOT1 => format!("udiv {}, slot[{}]", operand.op1, operand.slot_id.0),
            OR_REG1SLOT1 => format!("or {}, slot[{}]", operand.op1, operand.slot_id.0),
            AND_REG1SLOT1 => format!("and {}, slot[{}]", operand.op1, operand.slot_id.0),
            XOR_REG1SLOT1 => format!("xor {}, slot[{}]", operand.op1, operand.slot_id.0),
            _ => unimplemented!(),
        })
    }

    fn visit_slot3(&mut self, op: u8, operand: Slot3) {
        self.0.push(match op {
            UADD_SLOT3 => format!(
                "uadd slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            USUB_SLOT3 => format!(
                "usub slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            UMUL_SLOT3 => format!(
                "umul slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            UDIV_SLOT3 => format!(
                "udiv slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),

            OR_SLOT3 => format!(
                "or slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            AND_SLOT3 => format!(
                "and slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            XOR_SLOT3 => format!(
                "xor slot[{}], slot[{}] slot[{}]",
                operand.slot1.0, operand.slot2.0, operand.slot3.0
            ),
            _ => unimplemented!(),
        })
    }
}
