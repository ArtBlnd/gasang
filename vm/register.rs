use crate::interrupt::Interrupt;

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegId(pub u8);

impl Display for RegId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "reg:{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct GprRegister {
    name: String,
    size: u8,
    value: u64,
}

impl GprRegister {
    pub fn new(name: impl AsRef<str>, size: u8) -> Self {
        Self {
            name: name.as_ref().to_string(),
            size,
            value: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn set(&mut self, value: u64) {
        self.value = value;
    }

    pub fn get(&self) -> u64 {
        self.value
    }

    pub fn add(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value += value;
        Ok(())
    }

    pub fn sub(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value -= value;
        Ok(())
    }

    pub fn mul(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value *= value;
        Ok(())
    }

    pub fn div(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value /= value;
        Ok(())
    }

    pub fn shr(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value >>= value;
        Ok(())
    }

    pub fn shl(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value <<= value;
        Ok(())
    }

    pub fn or(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value |= value;
        Ok(())
    }

    pub fn and(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value &= value;
        Ok(())
    }

    pub fn xor(&mut self, value: u64) -> Result<(), Interrupt> {
        self.value ^= value;
        Ok(())
    }

    pub fn not(&mut self) -> Result<(), Interrupt> {
        self.value = !self.value;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FprRegister {
    pub name: String,
    pub size: u8,
    pub value: f64,
}

impl FprRegister {
    pub fn new(name: impl AsRef<str>, size: u8) -> Self {
        Self {
            name: name.as_ref().to_string(),
            size,
            value: 0.0,
        }
    }
}
