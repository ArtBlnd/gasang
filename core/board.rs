use crate::codegen::{Codegen, Executable, ExecutionContext};
use crate::compiler::Compiler;
use crate::cpu::Cpu;
use crate::error::{CompileError, Error};
use crate::ir::{BlockDestination, IrBlock};
use crate::softmmu::{MemoryFrame, Mmu};

use machineinstr::{MachineInstParser, MachineInstrParserRule};
use utility::ByteReader;

use thread_local::ThreadLocal;

use std::borrow::BorrowMut;
use std::convert::Infallible;
use std::sync::Mutex;

pub struct Board<C, R, G> {
    ir_comp: C, // IR compiler, which compiles machine instructions into IR blocks.
    ir_cgen: G, // IR codegen, which generates executable code from IR blocks.

    mci_parser: R,

    mmu: Mmu,

    cpu_init: Cpu,
    cpu_core: ThreadLocal<Mutex<Cpu>>,
}

impl<C, R, G> Board<C, R, G> {
    pub fn new(ir_comp: C, ir_cgen: G, mci_parser: R, mmu: Mmu, cpu_init: Cpu) -> Self {
        Self {
            ir_comp,
            ir_cgen,
            mci_parser,
            mmu,
            cpu_init,
            cpu_core: ThreadLocal::new(),
        }
    }
}

impl<C, R, G> Board<C, R, G>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    pub unsafe fn run(&self) -> Result<Infallible, Error> {
        use std::panic;
        use std::process::exit;

        let mut mmu = self.mmu.clone();
        let mut cpu = self
            .cpu_core
            .get_or(|| Mutex::new(self.cpu_init.clone()))
            .lock()
            .unwrap();

        let mut ctx = ExecutionContext {
            cpu: cpu.borrow_mut(),
            mmu: &mut mmu,
        };

        let this = panic::AssertUnwindSafe(|| self.run_inner(&mut ctx));
        match panic::catch_unwind(this) {
            Err(_) => {}
            Ok(Err(err)) => return Err(err),
            Ok(Ok(_)) => unreachable!(),
        }
        cpu.dump();

        exit(-1);
    }

    pub unsafe fn run_inner(&self, ctx: &mut ExecutionContext) -> Result<Infallible, Error> {
        // get entrypoint memory frame and compile it.
        let ep_frame = ctx.mmu.frame(ctx.cpu.pc());
        let ep_block = self.compile_until_branch_or_eof(ep_frame)?;

        let mut compiled = codegen_ir_blocks(ep_block, &self.ir_cgen);

        loop {
            debug_assert!(!compiled.is_empty());
            for code in compiled {
                code.execute(ctx);
            }

            let next_frame = ctx.mmu.frame(ctx.cpu.pc());
            let next_block = self.compile_until_branch_or_eof(next_frame)?;

            compiled = codegen_ir_blocks(next_block, &self.ir_cgen);
        }
    }

    unsafe fn compile_until_branch_or_eof(
        &self,
        frame: MemoryFrame,
    ) -> Result<Vec<IrBlock>, CompileError> {
        compile_until_branch_or_eof(frame, &self.mci_parser, &self.ir_comp)
    }
}

unsafe fn codegen_ir_blocks<C>(blocks: Vec<IrBlock>, codegen: &C) -> Vec<C::ExecBlock>
where
    C: Codegen,
{
    blocks.iter().map(|b| codegen.compile_ir_block(b)).collect()
}

unsafe fn compile_until_branch_or_eof<R, C>(
    frame: MemoryFrame,
    rule: &R,
    compiler: &C,
) -> Result<Vec<IrBlock>, CompileError>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
{
    let mut results = Vec::new();

    // This should not be implemented globally.
    // reading MemoryFrame is unsafe.
    impl Iterator for MemoryFrame {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            let result = unsafe { self.read_u8().ok() };

            self.consume(1);
            result
        }
    }

    let parser = MachineInstParser::new(ByteReader::new(frame), rule.clone());
    for instr in parser {
        let block = compiler.compile(instr.op);
        let last_dest = block.items().last().unwrap().dest().clone();
        results.push(block);

        if let BlockDestination::Exit | BlockDestination::Pc = last_dest {
            break;
        }
    }

    Ok(results)
}
