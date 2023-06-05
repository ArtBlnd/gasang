use core::{ir::BasicBlock, Architecture};
use std::collections::HashMap;

use cranelift::codegen::Context as CodegenContext;
use cranelift::frontend::FunctionBuilder;
use cranelift::prelude::*;
use cranelift_jit::JITModule;

use crate::{Codegen, Executable};

pub struct CraneliftCodegen {
    ctx_function_builder: FunctionBuilderContext,
    ctx_codegen: CodegenContext,

    module: JITModule,
}

impl Codegen for CraneliftCodegen {
    type Context = ();
    type Executable = CraneliftExecutable;

    fn new_context<A: Architecture>() -> Self::Context {
        todo!()
    }

    fn compile(&self, _bb: BasicBlock) -> Self::Executable {
        todo!()
    }
}

pub struct CraneliftExecutable;
impl Executable for CraneliftExecutable {
    type Context = ();

    unsafe fn execute(&self, _context: &mut Self::Context) {
        todo!()
    }
}

pub struct BasicBlockTranslator<'cg> {
    basic_block: BasicBlock,

    function_builder: FunctionBuilder<'cg>,
    function_variables: HashMap<usize, Variable>,

    module: &'cg JITModule,
}

impl<'cg> BasicBlockTranslator<'cg> {
    pub fn finalize(self) {
        self.function_builder.finalize();
    }

    pub fn translate(&self) {}
}
