use core::ir::{Flag, IrConstant, IrValue};
use std::{
    cell::{Cell, RefCell},
    mem,
};

use crate::codegen::{Context, ValueView};

use super::RegisterFile;

pub struct RustjitContext {
    pub(super) registers: RegisterFile,
    pub(super) variables: RefCell<Option<Box<[Cell<u128>]>>>,
    pub(super) flag: Box<[Cell<bool>]>,
}

impl Context for RustjitContext {
    #[inline(always)]
    fn get<T: ValueView>(&self, value: IrValue) -> T {
        match value {
            IrValue::Constant(IrConstant::U8(value)) => T::from_bytes(&[value]),
            IrValue::Constant(IrConstant::U16(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Constant(IrConstant::U32(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Constant(IrConstant::U64(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Constant(IrConstant::I8(value)) => T::from_bytes(&[value as u8]),
            IrValue::Constant(IrConstant::I16(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Constant(IrConstant::I32(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Constant(IrConstant::I64(value)) => T::from_bytes(&value.into_bytes()),
            IrValue::Register(_, id) => self.registers.get(id),
            IrValue::Variable(_, id) => {
                let variable = self.variables.borrow().as_ref().unwrap()[id].get();
                T::from_bytes(&variable.into_bytes()[..mem::size_of::<T>()])
            }
        }
    }

    #[inline(always)]
    fn set<T: ValueView>(&self, value: IrValue, new_value: T) {
        match value {
            IrValue::Constant(_) => panic!("Cannot set constant value"),
            IrValue::Register(_, id) => *self.registers.get_mut(id) = new_value,
            IrValue::Variable(_, id) => {
                let variable = self.variables.borrow().as_ref().unwrap()[id].get();
                let mut dst = variable.into_bytes();
                let src = new_value.into_bytes();
                let src = src.as_ref();

                dst[..src.len()].copy_from_slice(src.as_ref());
                self.variables.borrow().as_ref().unwrap()[id].set(u128::from_bytes(&dst));
            }
        }
    }

    #[inline(always)]
    fn get_flag(&self, flag: Flag) -> bool {
        self.flag[flag.into_index()].get()
    }

    #[inline(always)]
    fn set_flag(&self, flag: Flag, value: bool) {
        self.flag[flag.into_index()].set(value)
    }
}
