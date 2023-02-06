use std::convert::Infallible;

use crate::codegen::{Codegen, Executable};
use crate::compiler::Compiler;
use crate::error::{CompileError, Error};
use crate::ir::{BlockDestination, IrBlock};
use crate::mmu::MemoryFrame;
use crate::vm_state::VmState;

use machineinstr::{MachineInstParser, MachineInstrParserRule};

use utility::BitReader;

pub struct Engine<C, R, G> {
    compiler: C,
    parse_rule: R,

    codegen: G,
}

impl<C, R, G> Engine<C, R, G> {
    pub fn new(compiler: C, parse_rule: R, codegen: G) -> Self {
        Self {
            compiler,
            parse_rule,
            codegen,
        }
    }
}

impl<C, R, G> Engine<C, R, G>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    pub unsafe fn run(&mut self, vm_state: &mut VmState) -> Result<Infallible, Error> {
        // get entrypoint memory frame and compile it.
        let ep_frame = vm_state.mem(vm_state.ip());
        let ep_block = self.compile_until_branch_or_eof(ep_frame)?;

        let mut code = self.codegen.compile(ep_block);

        loop {
            for code_block in &code {
                code_block.execute(vm_state);
            }

            vm_state.dump();

            let next_frame = vm_state.mem(vm_state.ip());
            let next_block = self.compile_until_branch_or_eof(next_frame)?;

            code = self.codegen.compile(next_block);
        }
    }

    fn compile_until_branch_or_eof(
        &mut self,
        frame: MemoryFrame,
    ) -> Result<Vec<IrBlock>, CompileError> {
        compile_until_branch_or_eof(frame, &self.parse_rule, &self.compiler)
    }
}

fn compile_until_branch_or_eof<R, C>(
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
        let block = compiler.compile(instr.op)?;
        let last_dest = block.items().last().unwrap().dest().clone();
        results.push(block);

        if let BlockDestination::Eip = last_dest {
            break;
        }

        if let BlockDestination::Exit = last_dest {
            break;
        }
    }

    Ok(results)
}
