use crate::codegen::*;
use crate::interrupt::InterruptModel;
use crate::ir::{BlockDestination, Type};
use crate::register::RegId;
use crate::VmState;

pub trait CompiledBlockDestinationTrait {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, interrupt_mode: &dyn InterruptModel);
}

pub type CompiledBlockDestination = Box<dyn CompiledBlockDestinationTrait>;

pub fn codegen_block_dest<C>(codegen: &C, ds: BlockDestination) -> CompiledBlockDestination
where
    C: Codegen,
    C::Code: 'static,
{
    match ds {
        BlockDestination::Flags => Box::new(SetFlags),
        BlockDestination::Ip => Box::new(SetIp),
        BlockDestination::Gpr(ty, reg_id) => Box::new(SetGpr(ty, reg_id)),
        BlockDestination::Fpr(ty, reg_id) => Box::new(SetFpr(ty, reg_id)),
        BlockDestination::Memory(ty, addr) => Box::new(SetMemory(ty, addr)),
        BlockDestination::MemoryRelI64(ty, reg_id, offset) => {
            Box::new(SetMemoryI64(ty, reg_id, offset))
        }
        BlockDestination::MemoryRelU64(ty, _, _) => todo!(),
        BlockDestination::MemoryIr(ir) => {
            let ty = ir.get_type();
            let compiled_ir = Box::new(codegen.compile(ir));
            Box::new(StoreMemoryIr(ty, compiled_ir))
        }
        BlockDestination::None => Box::new(NoneDest),
        BlockDestination::SystemCall => Box::new(SystemCall),
        BlockDestination::Exit => Box::new(Exit),
    }
}

struct SetFlags;
impl CompiledBlockDestinationTrait for SetFlags {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        vm.set_flag(val);
    }
}

struct SetIp;
impl CompiledBlockDestinationTrait for SetIp {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        vm.set_ip(val);
    }
}
struct SetGpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetGpr {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        let origin = vm.gpr(self.1).get();
        let val = origin & !self.0.gen_mask() | val & self.0.gen_mask();

        vm.gpr_mut(self.1).set(val);
    }
}

struct SetFpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetFpr {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        let origin = vm.gpr(self.1).get();
        let val = origin & !self.0.gen_mask() | val & self.0.gen_mask();

        vm.gpr_mut(self.1).set(val);
    }
}

struct SetMemory(Type, u64);
impl CompiledBlockDestinationTrait for SetMemory {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        match self.0 {
            Type::U8 | Type::I8 => vm.mem(self.1).write_u8(val as u8),
            Type::U16 | Type::I16 => vm.mem(self.1).write_u16(val as u16),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(self.1).write_u32(val as u32),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(self.1).write_u64(val as u64),
            Type::Void | Type::Bool => unreachable!(),
        }.unwrap();
    }
}

struct SetMemoryI64(Type, RegId, i64);
impl CompiledBlockDestinationTrait for SetMemoryI64 {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        let (addr, of) = vm.gpr(self.1).get().overflowing_add_signed(self.2);
        assert_eq!(of, false);

        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(val as u8),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(val as u16),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(val as u32),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(val as u64),
            Type::Void | Type::Bool => unreachable!(),
        }.unwrap();
    }
}

struct StoreMemoryIr(Type, Box<dyn CompiledCode>);
impl CompiledBlockDestinationTrait for StoreMemoryIr {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, _: &dyn InterruptModel) {
        let addr = unsafe { self.1.execute(vm) };
        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(val as u8),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(val as u16),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(val as u32),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(val as u64),
            Type::Void | Type::Bool => unreachable!(),
        }.unwrap();
    }
}

struct NoneDest; 
impl CompiledBlockDestinationTrait for NoneDest {
    unsafe fn reflect(&self, _: u64, _: &mut VmState, _: &dyn InterruptModel) {}
}

struct SystemCall;
impl CompiledBlockDestinationTrait for SystemCall {
    unsafe fn reflect(&self, val: u64, vm: &mut VmState, interrupt_model: &dyn InterruptModel) {
        interrupt_model.syscall(val, vm)
    }
}

struct Exit;
impl CompiledBlockDestinationTrait for Exit {
    unsafe fn reflect(&self, _: u64, _: &mut VmState, _: &dyn InterruptModel) {
        panic!("exit");
    }
}