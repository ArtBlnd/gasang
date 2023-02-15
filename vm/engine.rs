use std::convert::Infallible;

use crate::codegen::{codegen_block_dest, Codegen, CompiledBlock, Executable};
use crate::compiler::Compiler;
use crate::cpu::Cpu;
use crate::error::{CompileError, Error};
use crate::interrupt::InterruptModel;
use crate::ir::{BlockDestination, IrBlock};
use crate::mmu::MemoryFrame;

use machineinstr::{MachineInstParser, MachineInstrParserRule};

use utility::BitReader;

pub struct Engine<C, R, M, G> {
    compiler: C,
    parse_rule: R,
    interrupt_model: M,

    codegen: G,
}

impl<C, R, M, G> Engine<C, R, M, G> {
    pub fn new(compiler: C, parse_rule: R, interrupt_model: M, codegen: G) -> Self {
        Self {
            compiler,
            parse_rule,
            interrupt_model,
            codegen,
        }
    }
}

impl<C, R, M, G> Engine<C, R, M, G>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
    M: InterruptModel,
    G: Codegen,

    G::Code: 'static,
{
    pub unsafe fn run(&mut self, vm_state: &mut Cpu) -> Result<Infallible, Error> {
        let this = std::panic::AssertUnwindSafe(|| self.run_inner(vm_state));
        match std::panic::catch_unwind(this) {
            Err(_) => {}
            Ok(Err(err)) => return Err(err),
            Ok(Ok(_)) => unreachable!(),
        }

        std::process::exit(-1);
    }

    pub unsafe fn run_inner(&mut self, vm_state: &mut Cpu) -> Result<Infallible, Error> {
        // get entrypoint memory frame and compile it.
        let ep_frame = vm_state.mem(vm_state.ip());
        let ep_block = self.compile_until_branch_or_eof(ep_frame)?;

        let mut compiled = codegen_ir_blocks(ep_block, &self.codegen);

        loop {
            for code in compiled {
                code.execute(vm_state, &self.interrupt_model);
            }

            vm_state.dump();

            let next_frame = vm_state.mem(vm_state.ip());
            let next_block = self.compile_until_branch_or_eof(next_frame)?;

            compiled = codegen_ir_blocks(next_block, &self.codegen);
        }
    }

    unsafe fn compile_until_branch_or_eof(
        &mut self,
        frame: MemoryFrame,
    ) -> Result<Vec<IrBlock>, CompileError> {
        compile_until_branch_or_eof(frame, &self.parse_rule, &self.compiler)
    }
}

unsafe fn codegen_ir_blocks<C>(blocks: Vec<IrBlock>, codegen: &C) -> Vec<CompiledBlock<C::Code>>
where
    C: Codegen,
    C::Code: 'static,
{
    let mut result = Vec::with_capacity(blocks.len());
    for block in blocks {
        let mut compiled_block = CompiledBlock::new(block.original_size() as u64);

        if block.restore_flag() {
            compiled_block.set_restore_flag();
        }

        for item in block.items() {
            let code = codegen.compile(item.root().clone());
            let dest = codegen_block_dest(codegen, item.dest().clone());

            compiled_block.push(code, dest);
        }

        result.push(compiled_block);
    }

    result
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

    let parser = MachineInstParser::new(BitReader::new(frame), rule.clone());
    for instr in parser {
        let block = compiler.compile(instr.op);
        let last_dest = block.items().last().unwrap().dest().clone();
        results.push(block);

        if let BlockDestination::Ip = last_dest {
            break;
        }

        if let BlockDestination::Exit = last_dest {
            break;
        }
    }

    Ok(results)
}
