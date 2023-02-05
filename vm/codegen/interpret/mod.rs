use crate::codegen::interpret::InterpretExeuctableBlock;
use crate::codegen::Codegen;
use crate::error::CodegenError;
use crate::ir::{Block, Ir, Operand, Type};
use crate::VmState;

mod code;
pub use code::*;
mod function;
pub use function::*;

pub struct InterpretCodegen;

impl Codegen for InterpretCodegen {
    type Executable = CodeBlock;

    fn compile(&self, blocks: Vec<Block>) -> Result<Self::Executable, CodegenError> {
        let mut codes = Vec::new();
        let mut sizes = Vec::new();

        for block in blocks {
            let size = block.original_size();
            let exe_block = compile_block(block)?;

            codes.push(exe_block);
            sizes.push(size);
        }

        Ok(CodeBlock { codes, sizes })
    }
}

pub fn compile_block(block: Block) -> Result<InterpretExeuctableBlock, CodegenError> {
    Ok(InterpretExeuctableBlock {
        code: compile_ir(block.ir_root(), false)?,
        code_dest: block.ir_dest(),
        code_type: block.ir_root().get_type(),
    })
}

fn compile_ir(ir: &Ir, set_flag: bool) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    match ir {
        Ir::Add(Type::U64, Operand::Eip, Operand::Immediate(imm, Type::I64)) => {
            let imm = *imm;
            Ok(Box::new(move |ctx| (ctx.eip() as i64 + imm as i64) as u64))
        }
        Ir::Add(Type::U64, Operand::Eip, Operand::Immediate(imm, Type::U64)) => {
            let imm = *imm;
            Ok(Box::new(move |ctx| ctx.eip() + imm))
        }

        Ir::Add(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;

            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 + rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 + rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 + rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) + rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 + rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 + rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 + rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 + rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 => Box::new(move |ctx| {
                        (f32::from_bits(lhs.execute(ctx) as u32)
                            + f32::from_bits(rhs.execute(ctx) as u32))
                        .to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) + f64::from_bits(rhs.execute(ctx)))
                            .to_bits()
                    }),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Sub(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;

            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 - rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 - rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 - rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) - rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 - rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 - rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 - rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 - rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 => Box::new(move |ctx| {
                        (f32::from_bits(lhs.execute(ctx) as u32)
                            - f32::from_bits(rhs.execute(ctx) as u32))
                        .to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) - f64::from_bits(rhs.execute(ctx)))
                            .to_bits()
                    }),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Mul(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 * rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 * rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 * rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) * rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 * rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 * rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 * rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 * rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 => Box::new(move |ctx| {
                        (f32::from_bits(lhs.execute(ctx) as u32)
                            * f32::from_bits(rhs.execute(ctx) as u32))
                        .to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) * f64::from_bits(rhs.execute(ctx)))
                            .to_bits()
                    }),

                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Div(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 / rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 / rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 / rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) / rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 / rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 / rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 / rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 / rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 => Box::new(move |ctx| {
                        (f32::from_bits(lhs.execute(ctx) as u32)
                            / f32::from_bits(rhs.execute(ctx) as u32))
                        .to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) / f64::from_bits(rhs.execute(ctx)))
                            .to_bits()
                    }),

                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::LShl(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u8 as u64)
                    }
                    Type::U16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u16 as u64)
                    }
                    Type::U32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u32 as u64)
                    }
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) << rhs.execute(ctx)),
                    Type::I8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i8 as u64)
                    }
                    Type::I16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i16 as u64)
                    }
                    Type::I32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i32 as u64)
                    }
                    Type::I64 => {
                        Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i64 as u64)
                    }
                    Type::F32 | Type::F64 => unreachable!("invalid type for lshl! {:?}", t),

                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::LShr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u8 as u64)
                    }
                    Type::U16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u16 as u64)
                    }
                    Type::U32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u32 as u64)
                    }
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) >> rhs.execute(ctx)),
                    Type::I8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i8 as u64)
                    }
                    Type::I16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i16 as u64)
                    }
                    Type::I32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i32 as u64)
                    }
                    Type::I64 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i64 as u64)
                    }
                    Type::F32 | Type::F64 => unreachable!("invalid type for lshr! {:?}", t),

                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::AShr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u8 as u64)
                    }
                    Type::U16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u16 as u64)
                    }
                    Type::U32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u32 as u64)
                    }
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) >> rhs.execute(ctx)),
                    Type::I8 => {
                        Box::new(move |ctx| (lhs.execute(ctx) as i8 >> rhs.execute(ctx)) as u64)
                    }
                    Type::I16 => {
                        Box::new(move |ctx| (lhs.execute(ctx) as i16 >> rhs.execute(ctx)) as u64)
                    }
                    Type::I32 => {
                        Box::new(move |ctx| (lhs.execute(ctx) as i32 >> rhs.execute(ctx)) as u64)
                    }
                    Type::I64 => {
                        Box::new(move |ctx| (lhs.execute(ctx) as i64 >> rhs.execute(ctx)) as u64)
                    }
                    Type::F32 | Type::F64 => unreachable!("invalid type for ashr! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Rotr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u8).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u16).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u32).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U64 => {
                        Box::new(move |ctx| lhs.execute(ctx).rotate_right(rhs.execute(ctx) as u32))
                    }
                    Type::I8 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as i8).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as i16).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as i32).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as i64).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::F32 | Type::F64 => unreachable!("invalid type for rotr! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Load(t, op) => {
            let op = compile_op(op, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8 | Type::I8 => Box::new(move |ctx| {
                        let var = op.execute(ctx);
                        ctx.mem(var).read_u8().unwrap() as u64
                    }),
                    Type::U16 | Type::I16 => Box::new(move |ctx| {
                        let var = op.execute(ctx);
                        ctx.mem(var).read_u16().unwrap() as u64
                    }),
                    Type::U32 | Type::I32 | Type::F32 => Box::new(move |ctx| {
                        let var = op.execute(ctx);
                        ctx.mem(var).read_u32().unwrap() as u64
                    }),
                    Type::U64 | Type::I64 | Type::F64 => Box::new(move |ctx| {
                        let var = op.execute(ctx);
                        ctx.mem(var).read_u64().unwrap()
                    }),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::ZextCast(t, op) => {
            let op = compile_op(op, set_flag)?;
            unsafe {
                Ok(match t {
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::I8
                    | Type::I16
                    | Type::I32
                    | Type::I64 => Box::new(move |ctx| op.execute(ctx)),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for zext! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::SextCast(to, op) => {
            let from = op.get_type();
            let to = type_mask(*to);
            let op = compile_op(op, set_flag)?;

            unsafe {
                Ok(match from {
                    Type::U8 | Type::U16 | Type::U32 | Type::U64 => op,
                    Type::I8 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i8).into();
                        v as u64 & to
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i16).into();
                        v as u64 & to
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i32).into();
                        v as u64 & to
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        let v: i64 = op.execute(ctx) as i64;
                        v as u64 & to
                    }),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for zext! {:?}", from),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::BitCast(t, op) => {
            let op = compile_op(op, set_flag)?;
            let to = type_mask(*t);

            Ok(Box::new(move |ctx| unsafe { op.execute(ctx) & to }))
        }

        Ir::And(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;

            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 & rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 & rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 & rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) & rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 & rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 & rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 & rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 & rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for and! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Or(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;

            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 | rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 | rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 | rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) & rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 | rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 | rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 | rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 | rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for and! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }

        Ir::Xor(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag)?;
            let rhs = compile_op(op2, set_flag)?;

            unsafe {
                Ok(match t {
                    Type::U8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u8 ^ rhs.execute(ctx) as u8) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u16 ^ rhs.execute(ctx) as u16) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as u32 ^ rhs.execute(ctx) as u32) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| lhs.execute(ctx) & rhs.execute(ctx)),
                    Type::I8 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i8 ^ rhs.execute(ctx) as i8) as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i16 ^ rhs.execute(ctx) as i16) as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i32 ^ rhs.execute(ctx) as i32) as u64
                    }),
                    Type::I64 => Box::new(move |ctx| {
                        (lhs.execute(ctx) as i64 ^ rhs.execute(ctx) as i64) as u64
                    }),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for and! {:?}", t),
                    Type::Void => return Err(CodegenError::InvalidType),
                })
            }
        }
        Ir::Value(op) => compile_op(op, set_flag),
        Ir::Nop => Ok(Box::new(|_| 0)),
    }
}

fn compile_op(op: &Operand, set_flag: bool) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    Ok(match op {
        Operand::Ir(ir) => compile_ir(ir, set_flag)?,
        Operand::Register(id, t) => {
            let id = *id;
            let t = *t;
            Box::new(move |vm: &mut VmState| vm.gpr(id).get() & type_mask(t))
        }
        Operand::Immediate(imm, t) => {
            let imm = *imm;
            let t = *t;
            Box::new(move |_| imm & type_mask(t))
        }
        Operand::Eip => Box::new(move |ctx| ctx.eip()),
    })
}

const fn type_mask(t: Type) -> u64 {
    match t {
        Type::U8 | Type::I8 => u8::max_value() as u64,
        Type::U16 | Type::I16 => u16::max_value() as u64,
        Type::U32 | Type::I32 | Type::F32 => u32::max_value() as u64,
        Type::U64 | Type::I64 | Type::F64 => u64::max_value(),
        Type::Void => panic!("Void type has no mask"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::image::Image;
    use crate::interrupt::NoModel;
    use crate::vm_builder::VmBuilder;

    #[test]
    fn test_compile_simple_imm_ir() {
        let image = Image::from_image(vec![]);
        let mut vm = VmBuilder::new(&image).build(0, NoModel);

        //Test UADD
        let ir = Ir::Add(
            Type::U64,
            Operand::Immediate(30, Type::U64),
            Operand::Immediate(50, Type::U64),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 80);

        //Test IADD
        let ir = Ir::Add(
            Type::I64,
            Operand::Immediate((-10i64) as u64, Type::I64),
            Operand::Immediate(10, Type::I64),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 0);

        //Test USUB
        let ir = Ir::Sub(
            Type::U8,
            Operand::Immediate(10, Type::U8),
            Operand::Immediate(9, Type::U8),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 1_u64);

        //Test subtract minus value ISUB
        let ir = Ir::Sub(
            Type::I16,
            Operand::Immediate(10, Type::I16),
            Operand::Immediate((-10i16) as u64, Type::I16),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 20);

        //Test IMUL
        let ir = Ir::Mul(
            Type::I32,
            Operand::Immediate(10, Type::I32),
            Operand::Immediate((-10i64) as u64, Type::I32),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result as i64, -100);

        //Test UMUL-F64
        let ir = Ir::Mul(
            Type::F64,
            Operand::Immediate((10f64).to_bits(), Type::F64),
            Operand::Immediate((4.5f64).to_bits(), Type::F64),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert!(f64::from_bits(result) > 44.9 && f64::from_bits(result) < 45.1);

        //Test IDIV
        let ir = Ir::Div(
            Type::I64,
            Operand::Immediate(10_u64, Type::I64),
            Operand::Immediate((-5i64) as u64, Type::I64),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result as i64, -2);

        //Test UDIV-F64
        let ir = Ir::Div(
            Type::F64,
            Operand::Immediate((10f64).to_bits(), Type::F64),
            Operand::Immediate((3.3f64).to_bits(), Type::F64),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert!(f64::from_bits(result) > 3.0 && f64::from_bits(result) < 3.1);

        //Test LSHL
        let ir = Ir::LShl(
            Type::U8,
            Operand::Immediate(0b1111_1111, Type::U8),
            Operand::Immediate(4, Type::U8),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 0b1111_0000);

        //Test LSHR
        let ir = Ir::LShr(
            Type::U8,
            Operand::Immediate(0b0000_1111, Type::U8),
            Operand::Immediate(4, Type::U8),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 0b0000_0000);

        //Test Rotr
        let ir = Ir::Rotr(
            Type::U8,
            Operand::Immediate(0b1010_1111, Type::U8),
            Operand::Immediate(2, Type::U8),
        );
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result, 0b1110_1011);

        //Test Zext
        let ir = Ir::ZextCast(Type::U16, Operand::Immediate((-10i8) as u64, Type::U16));
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result as u16, 65526);

        //Test Sext
        let ir = Ir::SextCast(Type::I64, Operand::Immediate(0xFF, Type::I8));
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result as i64, -1);

        //Test Sext-32
        let ir = Ir::SextCast(Type::I32, Operand::Immediate(0xFF, Type::I8));
        let result = unsafe { compile_ir(&ir, false).unwrap().execute(&mut vm) };
        assert_eq!(result as i64, u32::max_value() as i64);
    }
}
