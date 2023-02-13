use crate::codegen::*;
use crate::interrupt::InterruptModel;
use crate::ir::{BlockDestination, Type};
use crate::register::RegId;
use crate::VmState;

pub trait CompiledBlockDestinationTrait {
    unsafe fn reflect(&self, val: Value, vm: &mut VmState, interrupt_mode: &dyn InterruptModel);
    fn is_dest_ip_or_exit(&self) -> bool {
        false
    }
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
        BlockDestination::MemoryRelU64(_ty, _, _) => todo!(),
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
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        vm.set_flag(*val.u64());
    }
}

struct SetIp;
impl CompiledBlockDestinationTrait for SetIp {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        vm.set_ip(*val.u64());
    }

    fn is_dest_ip_or_exit(&self) -> bool {
        true
    }
}
struct SetGpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetGpr {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        let val = *val.u64();

        let origin = vm.gpr(self.1).get();
        let val = origin & !self.0.gen_mask() | val & self.0.gen_mask();

        vm.gpr_mut(self.1).set(val);
    }
}

struct SetFpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetFpr {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        let val = *val.u64();

        let origin = vm.gpr(self.1).get();
        let val = origin & !self.0.gen_mask() | val & self.0.gen_mask();

        vm.gpr_mut(self.1).set(val);
    }
}

struct SetMemory(Type, u64);
impl CompiledBlockDestinationTrait for SetMemory {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        match self.0 {
            Type::U8 | Type::I8 => vm.mem(self.1).write_u8(*val.u8()),
            Type::U16 | Type::I16 => vm.mem(self.1).write_u16(*val.u16()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(self.1).write_u32(*val.u32()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(self.1).write_u64(*val.u64()),
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct SetMemoryI64(Type, RegId, i64);
impl CompiledBlockDestinationTrait for SetMemoryI64 {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        let (addr, of) = vm.gpr(self.1).get().overflowing_add_signed(self.2);
        println!("{addr:0x}");
        assert_eq!(of, false);

        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(*val.u8()),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(*val.u16()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(*val.u32()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(*val.u64()),
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct StoreMemoryIr(Type, Box<dyn CompiledCode>);
impl CompiledBlockDestinationTrait for StoreMemoryIr {
    unsafe fn reflect(&self, mut val: Value, vm: &mut VmState, _: &dyn InterruptModel) {
        let addr = unsafe { *self.1.execute(vm).u64() };
        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(*val.u8()),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(*val.u16()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(*val.u32()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(*val.u64()),
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct NoneDest;
impl CompiledBlockDestinationTrait for NoneDest {
    unsafe fn reflect(&self, _: Value, _: &mut VmState, _: &dyn InterruptModel) {}
}

struct SystemCall;
impl CompiledBlockDestinationTrait for SystemCall {
    unsafe fn reflect(
        &self,
        mut val: Value,
        vm: &mut VmState,
        interrupt_model: &dyn InterruptModel,
    ) {
        interrupt_model.syscall(*val.u64(), vm)
    }
}

struct Exit;
impl CompiledBlockDestinationTrait for Exit {
    unsafe fn reflect(&self, _: Value, _: &mut VmState, _: &dyn InterruptModel) {
        panic!("exit");
    }

    fn is_dest_ip_or_exit(&self) -> bool {
        true
    }
}
