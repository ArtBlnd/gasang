use crate::codegen::*;
use crate::ir::{BlockDestination, Type, VecType};
use crate::register::RegId;
use crate::value::Value;
use crate::Cpu;

pub trait CompiledBlockDestinationTrait {
    unsafe fn reflect(&self, val: Value, vm: &mut Cpu);
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
        BlockDestination::Sys(ty, reg_id) => Box::new(SetSys(ty, reg_id)),
        BlockDestination::FprSlot(ty, reg_id, slot) => Box::new(SetFprSlot(ty, reg_id, slot)),
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
        BlockDestination::Exit => Box::new(Exit),
    }
}

struct SetFlags;
impl CompiledBlockDestinationTrait for SetFlags {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        vm.set_flag(*val.u64_mut());
    }
}

struct SetIp;
impl CompiledBlockDestinationTrait for SetIp {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        vm.set_ip(*val.u64_mut());
    }

    fn is_dest_ip_or_exit(&self) -> bool {
        true
    }
}
struct SetGpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetGpr {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        let gpr = vm.gpr_mut(self.1);
        match self.0 {
            Type::U8 | Type::I8 => *gpr.u8_mut() = *val.u8_mut(),
            Type::U16 | Type::I16 => *gpr.u16_mut() = *val.u16_mut(),
            Type::U32 | Type::I32 => *gpr.u32_mut() = *val.u32_mut(),
            Type::U64 | Type::I64 => *gpr.u64_mut() = *val.u64_mut(),
            _ => unreachable!(),
        }
    }
}

struct SetFpr(Type, RegId);
impl CompiledBlockDestinationTrait for SetFpr {
    unsafe fn reflect(&self, val: Value, vm: &mut Cpu) {
        let fpr = vm.fpr_mut(self.1);
        match self.0 {
            Type::U8 | Type::I8 => *fpr.u8_mut() = val.u8(),
            Type::U16 | Type::I16 => *fpr.u16_mut() = val.u16(),
            Type::U32 | Type::I32 => *fpr.u32_mut() = val.u32(),
            Type::U64 | Type::I64 => *fpr.u64_mut() = val.u64(),
            Type::F32 => *fpr.f32_mut() = val.f32(),
            Type::F64 => *fpr.f64_mut() = val.f64(),
            Type::Vec(VecType::U64, 2) => *fpr.u64x2_mut() = val.u64x2(),
            _ => unreachable!(),
        }
    }
}

struct SetSys(Type, RegId);
impl CompiledBlockDestinationTrait for SetSys {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        let sys = vm.sys_mut(self.1);
        match self.0 {
            Type::U8 | Type::I8 => *sys.u8_mut() = *val.u8_mut(),
            Type::U16 | Type::I16 => *sys.u16_mut() = *val.u16_mut(),
            Type::U32 | Type::I32 => *sys.u32_mut() = *val.u32_mut(),
            Type::U64 | Type::I64 => *sys.u64_mut() = *val.u64_mut(),
            _ => unreachable!(),
        }
    }
}

struct SetMemory(Type, u64);
impl CompiledBlockDestinationTrait for SetMemory {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        match self.0 {
            Type::U8 | Type::I8 => vm.mem(self.1).write_u8(*val.u8_mut()),
            Type::U16 | Type::I16 => vm.mem(self.1).write_u16(*val.u16_mut()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(self.1).write_u32(*val.u32_mut()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(self.1).write_u64(*val.u64_mut()),
            Type::Vec(VecType::U64 | VecType::I64, 2) => {
                vm.mem(self.1).write(&val.u8_slice_ref()[..16])
            }
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct SetMemoryI64(Type, RegId, i64);
impl CompiledBlockDestinationTrait for SetMemoryI64 {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        let (addr, of) = vm.gpr(self.1).u64().overflowing_add_signed(self.2);
        assert_eq!(of, false);

        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(*val.u8_mut()),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(*val.u16_mut()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(*val.u32_mut()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(*val.u64_mut()),
            Type::Vec(VecType::U64 | VecType::I64, 2) => {
                vm.mem(addr).write(&val.u8_slice_ref()[..16])
            }
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct StoreMemoryIr(Type, Box<dyn CompiledCode>);
impl CompiledBlockDestinationTrait for StoreMemoryIr {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        let addr = unsafe { *self.1.execute(vm).u64_mut() };

        match self.0 {
            Type::U8 | Type::I8 => vm.mem(addr).write_u8(*val.u8_mut()),
            Type::U16 | Type::I16 => vm.mem(addr).write_u16(*val.u16_mut()),
            Type::U32 | Type::I32 | Type::F32 => vm.mem(addr).write_u32(*val.u32_mut()),
            Type::U64 | Type::I64 | Type::F64 => vm.mem(addr).write_u64(*val.u64_mut()),
            Type::Vec(VecType::U64 | VecType::I64, 2) => {
                vm.mem(addr).write(&val.u8_slice_ref()[..16])
            }
            _ => unreachable!(),
        }
        .unwrap();
    }
}

struct NoneDest;
impl CompiledBlockDestinationTrait for NoneDest {
    unsafe fn reflect(&self, _: Value, _: &mut Cpu) {}
}

struct Exit;
impl CompiledBlockDestinationTrait for Exit {
    unsafe fn reflect(&self, _: Value, _: &mut Cpu) {
        panic!("exit");
    }

    fn is_dest_ip_or_exit(&self) -> bool {
        true
    }
}

struct SetFprSlot(Type, RegId, u8);
impl CompiledBlockDestinationTrait for SetFprSlot {
    unsafe fn reflect(&self, mut val: Value, vm: &mut Cpu) {
        let fpr = vm.fpr_mut(self.1);

        match self.0 {
            Type::U8 | Type::I8 => fpr.u8_slice_mut()[self.2 as usize] = *val.u8_mut(),
            Type::U16 | Type::I16 => fpr.u16_slice_mut()[self.2 as usize] = *val.u16_mut(),
            Type::U32 | Type::I32 => fpr.u32_slice_mut()[self.2 as usize] = *val.u32_mut(),
            Type::U64 | Type::I64 => fpr.u64_slice_mut()[self.2 as usize] = *val.u64_mut(),
            _ => unreachable!(),
        }
    }
}
