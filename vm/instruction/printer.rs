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
    fn visit_reg1(&mut self, op: u8, operand: Reg1) {
        self.0.push(match op {
            IROP_MOV_IPR2REG => format!("mov irp, {}", operand.op1),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        self.0.push(match op {
            IROP_MOV_REG2MEM_REG => format!("mov {}, *({})", operand.op1, operand.op2),
            IROP_MOV_REG2REG => format!("mov {}, {}", operand.op1, operand.op2),
            _ => unimplemented!(),
        });
    }

    fn visit_reg3(&mut self, op: u8, operand: Reg3) {
        self.0.push(match op {
            IROP_UADD_REG3 => format!("uadd {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_USUB_REG3 => format!("usub {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_UMUL_REG3 => format!("umul {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_UDIV_REG3 => format!("udiv {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_OR_REG3 => format!("or {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_AND_REG3 => format!("and {}, {}, {}", operand.op1, operand.op2, operand.op3),
            IROP_XOR_REG3 => format!("xor {}, {}, {}", operand.op1, operand.op2, operand.op3),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1u8(&mut self, op: u8, operand: Reg1U8) {
        self.0.push(match op {
            IROP_UADD_CST8 => format!("uadd {}, {}", operand.op1, operand.imm8),
            IROP_USUB_CST8 => format!("usub {}, {}", operand.op1, operand.imm8),
            IROP_UMUL_CST8 => format!("umul {}, {}", operand.op1, operand.imm8),
            IROP_UDIV_CST8 => format!("udiv {}, {}", operand.op1, operand.imm8),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2u8(&mut self, op: u8, operand: Reg2U8) {
        self.0.push(match op {
            IROP_LLEFT_SHIFT_IMM8 => {
                format!("lshl {}, {} {}", operand.op1, operand.op2, operand.imm8)
            }
            IROP_LRIGHT_SHIFT_IMM8 => {
                format!("lshr {}, {} {}", operand.op1, operand.op2, operand.imm8)
            }
            IROP_ROTATE_IMM8 => format!("rot {}, {} {}", operand.op1, operand.op2, operand.imm8),
            IROP_ARIGHT_SHIFT_IMM8 => {
                format!("ashr {}, {} {}", operand.op1, operand.op2, operand.imm8)
            }
            _ => unimplemented!(),
        });
    }

    fn visit_reg1u32(&mut self, op: u8, operand: Reg1U32) {
        self.0.push(match op {
            IROP_UADD_CST32 => format!("uadd {}, {}", operand.op1, operand.imm32),
            IROP_USUB_CST32 => format!("usub {}, {}", operand.op1, operand.imm32),
            IROP_UMUL_CST32 => format!("umul {}, {}", operand.op1, operand.imm32),
            IROP_UDIV_CST32 => format!("udiv {}, {}", operand.op1, operand.imm32),
            IROP_MOV_REG2MEM_CST => format!("mov {}, *({})", operand.op1, operand.imm32),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1i32(&mut self, op: u8, operand: Reg1I32) {
        self.0.push(match op {
            IROP_IADD_CST32 => format!("iadd {}, {}", operand.op1, operand.imm32),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1u64(&mut self, op: u8, operand: Reg1U64) {
        self.0.push(match op {
            IROP_UADD_CST64 => format!("uadd {}, {}", operand.op1, operand.imm64),
            IROP_USUB_CST64 => format!("usub {}, {}", operand.op1, operand.imm64),
            IROP_UMUL_CST64 => format!("umul {}, {}", operand.op1, operand.imm64),
            IROP_UDIV_CST64 => format!("udiv {}, {}", operand.op1, operand.imm64),
            IROP_MOV_64CST2REG => format!("mov {}, {}", operand.op1, operand.imm64),
            _ => unimplemented!(),
        });
    }

    fn visit_reg1u16(&mut self, op: u8, operand: Reg1U16) {
        self.0.push(match op {
            IROP_MOV_16CST2REG => format!("mov {}, {}", operand.op1, operand.imm16),
            _ => unimplemented!(),
        });
    }

    fn visit_u16(&mut self, op: u8, operand: U16) {
        self.0.push(match op {
            IROP_SVC => format!("svc {}", operand.imm16),
            IROP_BRK => format!("brk {}", operand.imm16),
            _ => unimplemented!(),
        });
    }

    fn visit_reg2i64(&mut self, op: u8, operand: Reg2I64) {}

    fn visit_reg1i64(&mut self, op: u8, operand: Reg1I64) {}

    fn visit_no_operand(&mut self, op: u8) {
        self.0.push(match op {
            IROP_NOP => format!("nop"),
            _ => unimplemented!(),
        });
    }
}
