use crate::codegen::{Codegen, Executable, ExecutionContext};
use crate::compiler::Compiler;
use crate::cpu::Cpu;
use crate::debug::{DebugEvent, Event, ExecutionMode};
use crate::error::{CompileError, DebugError, Error};
use crate::ir::{BlockDestination, IrBlock};
use crate::softmmu::{Mmu, MmuData};

use gdbstub::arch::Arch;
use gdbstub::target::Target;
use machineinstr::{MachineInstParser, MachineInstrParserRule};
use utility::ByteReader;

use thread_local::ThreadLocal;

use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

pub struct Board<C, R, G, A> {
    ir_comp: C, // IR compiler, which compiles machine instructions into IR blocks.
    ir_cgen: G, // IR codegen, which generates executable code from IR blocks.
    mci_parser: R,

    mmu: Mmu,
    cpu_init: Cpu,
    cpu_core: ThreadLocal<Mutex<Cpu>>,
    exec_mode: ExecutionMode,

    debug_arch: A,
    breakpoints: HashSet<u64>,
}

impl<C, R, G, A> Board<C, R, G, A> {
    pub fn new(
        ir_comp: C,
        ir_cgen: G,
        mci_parser: R,
        debug_arch: A,
        mmu: Mmu,
        cpu_init: Cpu,
    ) -> Self {
        Self {
            ir_comp,
            ir_cgen,
            mci_parser,
            mmu,
            debug_arch,
            cpu_init,
            cpu_core: ThreadLocal::new(),
            exec_mode: ExecutionMode::Step,
            breakpoints: HashSet::new(),
        }
    }

    pub fn cpu(&self) -> &ThreadLocal<Mutex<Cpu>> {
        &self.cpu_core
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn add_breakpoint(&mut self, addr: u64) -> Result<(), DebugError> {
        if self.breakpoints.insert(addr) {
            Ok(())
        } else {
            Err(DebugError::BreakpointAlreadyExist(addr))
        }
    }

    pub fn remove_breakpoint(&mut self, addr: u64) -> Result<(), DebugError> {
        if self.breakpoints.remove(&addr) {
            Ok(())
        } else {
            Err(DebugError::BreakpointNotExist(addr))
        }
    }
}

impl<C, R, G, A> Board<C, R, G, A>
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
        loop {
            let block = self.compile_until_branch_or_eof(ctx.mmu.clone(), ctx.cpu().pc())?;

            let compiled = codegen_ir_blocks(block, &self.ir_cgen);

            debug_assert!(!compiled.is_empty());
            for code in compiled {
                code.execute(ctx);
            }
        }
    }

    unsafe fn compile_until_branch_or_eof(
        &self,
        mmu: Mmu,
        pc: u64,
    ) -> Result<Vec<IrBlock>, CompileError> {
        compile_until_branch_or_eof(mmu, pc, &self.mci_parser, &self.ir_comp)
    }
}

unsafe fn codegen_ir_blocks<C>(blocks: Vec<IrBlock>, codegen: &C) -> Vec<C::ExecBlock>
where
    C: Codegen,
{
    blocks.iter().map(|b| codegen.compile_ir_block(b)).collect()
}

unsafe fn compile_until_branch_or_eof<R, C>(
    mmu: Mmu,
    pc: u64,
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

    let parser = MachineInstParser::new(ByteReader::new(mmu.iter(pc)), rule.clone());
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

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> Board<C, R, G, A>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    pub unsafe fn debug(
        &self,
        mut poll_incoming_data: impl FnMut() -> bool,
    ) -> Result<DebugEvent, Error> {
        let mut mmu = self.mmu().clone();
        let mut cpu = self
            .cpu_core
            .get_or(|| Mutex::new(self.cpu_init.clone()))
            .lock()
            .unwrap();

        let mut ctx = ExecutionContext {
            cpu: cpu.borrow_mut(),
            mmu: &mut mmu,
        };

        match self.exec_mode {
            ExecutionMode::Continue => {
                let mut cycles = 0;
                loop {
                    if cycles % 1024 == 0 {
                        // poll for incoming data
                        if poll_incoming_data() {
                            break Ok(DebugEvent::IncomingData);
                        }
                    }
                    cycles += 1;

                    if let Some(event) = self.step(&mut ctx) {
                        break Ok(DebugEvent::Event(event));
                    };
                }
            }
            ExecutionMode::Step => Ok(DebugEvent::Event(
                self.step(&mut ctx).unwrap_or(Event::DoneStep),
            )),
        }
    }

    pub unsafe fn step(&self, ctx: &mut ExecutionContext) -> Option<Event> {
        let mut rule = self.mci_parser.clone();
        let instr = rule
            .parse(&mut ByteReader::new(ctx.mmu.iter(ctx.cpu().pc())))
            .unwrap();
        let block = self.ir_comp.compile(instr.op);
        let compiled = self.ir_cgen.compile_ir_block(&block);
        self.mmu().clear_events();

        compiled.execute(ctx);

        if let Some(wp) = self.mmu().check_watchpoint_hit() {
            return Some(Event::Watch(wp.0, wp.1));
        }

        if self.breakpoints.contains(&ctx.cpu().pc()) {
            return Some(Event::SwBreak);
        }

        None
    }
}
