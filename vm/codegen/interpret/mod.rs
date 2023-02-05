use crate::ir::{Block, BlockDestination, Ir, Operand, Type};
use crate::codegen::interpret::InterpretExeuctableBlock;
use crate::VmState;

mod code;
pub use code::*;
mod function;
pub use function::*;



pub fn compile_block(block: Block) -> InterpretExeuctableBlock {
    InterpretExeuctableBlock {
        code: compile_ir(block.ir_root(), false),
        code_dest: block.ir_dest(),
        code_type: block.ir_root().get_type()
    }
}

fn compile_ir(ir: &Ir, set_flag: bool) -> Box<dyn InterpretFunc> {
    match ir {
        Ir::Add(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);

            unsafe {
                match t {
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
                        (f32::from_bits(lhs.execute(ctx) as u32) + f32::from_bits(rhs.execute(ctx) as u32)).to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) + f64::from_bits(rhs.execute(ctx))).to_bits() as u64
                    }),
                }
            }
        }
        Ir::Sub(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);

            unsafe {
                match t {
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
                        (f32::from_bits(lhs.execute(ctx) as u32) - f32::from_bits(rhs.execute(ctx) as u32)).to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) - f64::from_bits(rhs.execute(ctx))).to_bits() as u64
                    }),
                }
            }
        }
        Ir::Mul(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
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
                        (f32::from_bits(lhs.execute(ctx) as u32) * f32::from_bits(rhs.execute(ctx) as u32)).to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) * f64::from_bits(rhs.execute(ctx))).to_bits() as u64
                    }),
                }
            }
        }
        Ir::Div(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
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
                        (f32::from_bits(lhs.execute(ctx) as u32) / f32::from_bits(rhs.execute(ctx) as u32)).to_bits() as u64
                    }),
                    Type::F64 => Box::new(move |ctx| {
                        (f64::from_bits(lhs.execute(ctx)) / f64::from_bits(rhs.execute(ctx))).to_bits() as u64
                    }),
                }
            }
        }
        Ir::LShl(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
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
                }
            }
        }
        Ir::LShr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
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
                }
            }
        }
        Ir::AShr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
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
                }
            }
        }
        Ir::Rotr(t, op1, op2) => {
            let lhs = compile_op(op1, set_flag);
            let rhs = compile_op(op2, set_flag);
            unsafe {
                match t {
                    Type::U8 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u8).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U16 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u16).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U32 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u32).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
                    Type::U64 => Box::new(move |ctx| {
                        ((lhs.execute(ctx) as u64).rotate_right(rhs.execute(ctx) as u32)) as u64
                    }),
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
                }
            }
        }
        Ir::Load(t, op) => {
            let op = compile_op(op, set_flag);
            unsafe {
                match t {
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
                        ctx.mem(var).read_u64().unwrap() as u64
                    }),
                }
            }
        }
        Ir::ZextCast(t, op) => {
            let op = compile_op(op, set_flag);
            unsafe {
                match t {
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::I8
                    | Type::I16
                    | Type::I32
                    | Type::I64 => Box::new(move |ctx| op.execute(ctx)),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for zext! {:?}", t),
                }
            }
        }
        Ir::SextCast(t, op) => {
            let op = compile_op(op, set_flag);
            unsafe {
                match t {
                    Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                        Box::new(move |ctx| op.execute(ctx))
                    }
                    Type::I8 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i8).into();
                        v as u64
                    }),
                    Type::I16 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i16).into();
                        v as u64
                    }),
                    Type::I32 => Box::new(move |ctx| {
                        let v: i64 = (op.execute(ctx) as i32).into();
                        v as u64
                    }),
                    Type::I64 => Box::new(move |ctx| op.execute(ctx)),
                    Type::F32 | Type::F64 => unimplemented!("invalid type for zext! {:?}", t),
                }
            }
        }
        Ir::BitCast(t, op) => todo!(),
    }
}

fn compile_op(op: &Operand, set_flag: bool) -> Box<dyn InterpretFunc> {
    match op {
        Operand::Ir(ir) => compile_ir(&ir, set_flag),
        Operand::Register(id) => {
            let id = id.clone();
            Box::new(move |vm: &mut VmState| vm.gpr(id).get())
        }
        Operand::Immediate(imm) => {
            let imm = *imm;
            Box::new(move |_| imm)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::image::Image;
    use crate::vm_builder::VmBuilder;

    #[test]
    fn test_compile_simple_imm_ir() {
        let image = Image::from_image(vec![]);
        let mut vm = VmBuilder::new(&image).build(0);

        //Test UADD
        let ir = Ir::Add(Type::U64, Operand::Immediate(30), Operand::Immediate(50));
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result, 80);

        //Test IADD
        let ir = Ir::Add(
            Type::I64,
            Operand::Immediate((-10i64) as u64),
            Operand::Immediate(10),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result, 0);

        //Test USUB
        let ir = Ir::Sub(Type::U8, Operand::Immediate(10), Operand::Immediate(9));
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result, 1 as u64);

        //Test subtract minus value ISUB
        let ir = Ir::Sub(
            Type::I16,
            Operand::Immediate(10),
            Operand::Immediate((-10i16) as u64),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result, 20);

        //Test IMUL
        let ir = Ir::Mul(
            Type::I32,
            Operand::Immediate(10),
            Operand::Immediate((-10i64) as u64),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result as i64, -100);

        //Test UMUL-F64
        let ir = Ir::Mul(
            Type::F64,
            Operand::Immediate(10),
            Operand::Immediate((4.5f64).to_bits()),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert!(result as f64 > 44.9 && (result as f64) < 45.1);

        //Test IDIV
        let ir = Ir::Div(
            Type::I64,
            Operand::Immediate((10) as u64),
            Operand::Immediate((-5i64) as u64),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        assert_eq!(result as i64, -2);

        //Test UDIV-F64
        let ir = Ir::Div(
            Type::F64,
            Operand::Immediate((10) as u64),
            Operand::Immediate((3.3f64) as u64),
        );
        let result = unsafe { compile_ir(&ir, false).execute(&mut vm) };
        println!("{}", result as f64);
        assert!(result as f64 > 3.0 && (result as f64) < 3.1);
    }
}
