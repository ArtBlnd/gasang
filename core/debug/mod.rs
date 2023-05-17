pub mod aarch64;
use aarch64::*;
use gdbstub::common::Signal;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::run_blocking::BlockingEventLoop;
use gdbstub::stub::SingleThreadStopReason;
use machineinstr::{MachineInstParser, MachineInstrParserRule};
use utility::ByteReader;

use crate::board::Board;
use crate::codegen::{Codegen, ExecutionContext};
use crate::compiler::Compiler;
use crate::error::{DebugError, Error, MmuError};
use crate::{cpu::Architecture, Cpu};

use gdbstub::arch::{Arch, Registers};
use gdbstub::target::ext::base::singlethread::SingleThreadBase;
use gdbstub::target::ext::breakpoints::{
    Breakpoints, HwBreakpoint, HwWatchpoint, SwBreakpoint, WatchKind as GdbWatchKind,
};
use gdbstub::target::{Target, TargetError};

use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::ops::Range;

pub enum ExecutionMode {
    Continue,
    Step,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WatchKind {
    Read,
    Write,
    ReadWrite,
}

impl From<WatchKind> for GdbWatchKind {
    fn from(value: WatchKind) -> Self {
        match value {
            WatchKind::Read => GdbWatchKind::Read,
            WatchKind::Write => GdbWatchKind::Write,
            WatchKind::ReadWrite => GdbWatchKind::ReadWrite,
        }
    }
}

impl From<GdbWatchKind> for WatchKind {
    fn from(value: GdbWatchKind) -> Self {
        match value {
            GdbWatchKind::Write => WatchKind::Write,
            GdbWatchKind::Read => WatchKind::Read,
            GdbWatchKind::ReadWrite => WatchKind::ReadWrite,
        }
    }
}

pub enum Event {
    DoneStep,
    Halted,
    Exit,
    SwBreak,
    HwBreak,
    Watch(u64, WatchKind),
}

pub enum DebugEvent {
    IncomingData,
    Event(Event),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatchPoint {
    pub addr: u64,
    pub len: u64,
    pub kind: WatchKind,
}

impl WatchPoint {
    pub fn is_hit(&self, range: &Range<u64>, watch_kind: WatchKind) -> bool {
        let in_range = range.contains(&self.addr)
            || range.contains(&(self.addr + self.len - 1))
            || (self.addr < range.start && self.addr + self.len >= range.end);
        let same_kind = self.kind == WatchKind::ReadWrite || self.kind == watch_kind;

        in_range && same_kind
    }
}

pub struct GdbEventLoop<C, R, G, A> {
    c: PhantomData<C>,
    r: PhantomData<R>,
    g: PhantomData<G>,
    a: PhantomData<A>,
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> BlockingEventLoop
    for GdbEventLoop<C, R, G, A>
where
    C: Compiler,
    R: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    type Target = Board<C, R, G, A>;
    type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;
    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        gdbstub::stub::run_blocking::Event<Self::StopReason>,
        gdbstub::stub::run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as gdbstub::conn::Connection>::Error,
        >,
    > {
        let poll_incoming_data = || conn.peek().map(|b| b.is_some()).unwrap_or(true);

        let debug_event = unsafe { target.debug(poll_incoming_data).unwrap() };

        let stop_reason = match debug_event {
            DebugEvent::IncomingData => todo!(),
            DebugEvent::Event(event) => match event {
                Event::DoneStep => SingleThreadStopReason::DoneStep,
                Event::Halted => SingleThreadStopReason::Terminated(Signal::SIGSTOP),
                Event::SwBreak => SingleThreadStopReason::SwBreak(()),
                Event::HwBreak => SingleThreadStopReason::HwBreak(()),
                Event::Watch(addr, kind) => SingleThreadStopReason::Watch {
                    tid: (),
                    kind: kind.into(),
                    addr,
                },
                Event::Exit => SingleThreadStopReason::Exited(0),
            },
        };

        Ok(gdbstub::stub::run_blocking::Event::TargetStopped(
            stop_reason,
        ))
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        todo!()
    }
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> SingleThreadBase for Board<C, R, G, A> {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> gdbstub::target::TargetResult<(), Self> {
        let cpu = self.cpu().get().unwrap().lock().unwrap();

        for name in cpu.arch().gprs() {
            let src = cpu.reg_by_name(&name).unwrap();
            let src = cpu.gpr(src);
            let dst = regs.reg_by_name(&name).unwrap();
            let dst = regs.gpr_mut(dst);

            *dst = src.clone();
        }

        regs.set_pc(cpu.pc());
        regs.set_flag(cpu.flag());

        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
    ) -> gdbstub::target::TargetResult<(), Self> {
        let mut cpu = self.cpu().get().unwrap().lock().unwrap();
        let cpu = cpu.borrow_mut();

        for name in Architecture::AArch64Bin.gprs() {
            let dst = cpu
                .reg_by_name(&name)
                .ok_or(TargetError::Fatal(DebugError::InvalidRegName(name.clone())))?;
            let dst = cpu.gpr_mut(dst);
            let src = regs
                .reg_by_name(&name)
                .ok_or(TargetError::Fatal(DebugError::InvalidRegName(name.clone())))?;
            let src = regs.gpr(src);

            *dst = src.clone();
        }

        cpu.set_pc(regs.pc());
        cpu.set_flag(regs.flag());

        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
    ) -> gdbstub::target::TargetResult<(), Self> {
        let mmu = self.mmu();

        unsafe {
            mmu.read(start_addr, data)
                .map_err(|e| TargetError::Fatal(DebugError::MMU(e)))?;
        }

        Ok(())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &[u8],
    ) -> gdbstub::target::TargetResult<(), Self> {
        let mmu = self.mmu();

        unsafe {
            mmu.write(start_addr, data)
                .map_err(|e| TargetError::Fatal(DebugError::MMU(e)))?;
        }

        Ok(())
    }
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> Target for Board<C, R, G, A> {
    type Arch = A;
    type Error = DebugError;

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        gdbstub::target::ext::base::BaseOps::SingleThread(self)
    }
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> Breakpoints for Board<C, R, G, A> {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        None
    }

    fn support_hw_breakpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::HwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_watchpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::HwWatchpointOps<'_, Self>> {
        Some(self)
    }
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> HwBreakpoint for Board<C, R, G, A> {
    fn add_hw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        _kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        Ok(self.add_breakpoint(addr).is_ok())
    }

    fn remove_hw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        _kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        Ok(self.remove_breakpoint(addr).is_ok())
    }
}

impl<C, R, G, A: Arch<Usize = u64, Registers = Cpu>> HwWatchpoint for Board<C, R, G, A> {
    fn add_hw_watchpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        len: <Self::Arch as Arch>::Usize,
        kind: GdbWatchKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let result = self.mmu().add_watchpoint(WatchPoint {
            addr,
            len,
            kind: kind.into(),
        });

        Ok(result.is_ok())
    }

    fn remove_hw_watchpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        len: <Self::Arch as Arch>::Usize,
        kind: GdbWatchKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let result = self.mmu().remove_watchpoint(WatchPoint {
            addr,
            len,
            kind: kind.into(),
        });

        Ok(result.is_ok())
    }
}

impl Registers for Cpu {
    type ProgramCounter = u64;

    fn pc(&self) -> Self::ProgramCounter {
        self.pc()
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        let vec = match self.arch() {
            Architecture::AArch64Bin => serialize_aarch64(self).unwrap(),
            _ => unreachable!(),
        };

        for byte in vec {
            write_byte(Some(byte));
        }

        write_byte(None);
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        match self.arch() {
            Architecture::AArch64Bin => deserialize_aarch64(bytes, self),
            _ => unreachable!(),
        };

        Ok(())
    }
}