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
            MOV_IPR_REG => format!("mov irp, {}", operand.op1),
            POP_REG => format!("pop {}", operand.op1),
            PSH_REG => format!("psh {}", operand.op1),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        self.0.push(match op {
            MOV_REG2 => format!("mov {}, {}", operand.op1, operand.op2),
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
            UADD_REG2IMM8 => format!("uadd {}, {}", operand.op1, operand.imm8),
            USUB_REG2IMM8 => format!("usub {}, {}", operand.op1, operand.imm8),
            UMUL_REG2IMM8 => format!("umul {}, {}", operand.op1, operand.imm8),
            UDIV_REG2IMM8 => format!("udiv {}, {}", operand.op1, operand.imm8),
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
            MOV_REG1IMM64 => format!("mov {}, {}", operand.op1, operand.imm64),
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
            _ => unimplemented!(),
        });
    }

    fn visit_reg2imm16(&mut self, op: u8, operand: Reg2Imm16) {
        self.0.push(match op {
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
            BR_IRP_IMM32_REL => format!("jmp_ipr {}", operand.imm32),
            _ => unimplemented!(),
        })
    }
}
