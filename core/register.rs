use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
};

use crate::value::Value;

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
    value: Value,
}

impl GprRegister {
    pub fn new(name: impl AsRef<str>, size: u8) -> Self {
        Self {
            name: name.as_ref().to_string(),
            size,
            value: Value::new(size as usize),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u8 {
        self.size
    }
}

impl Deref for GprRegister {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for GprRegister {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug, Clone)]
pub struct FprRegister {
    name: String,
    size: u8,
    value: Value,
}

impl FprRegister {
    pub fn new(name: impl AsRef<str>, size: u8) -> Self {
        Self {
            name: name.as_ref().to_string(),
            size,
            value: Value::new(size as usize),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u8 {
        self.size
    }
}

impl Deref for FprRegister {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for FprRegister {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct SysRegister {
    name: String,
    size: u8,
    value: Value,
}

impl SysRegister {
    pub fn new(name: impl AsRef<str>, size: u8) -> Self {
        Self {
            name: name.as_ref().to_string(),
            size,
            value: Value::new(size as usize),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u8 {
        self.size
    }
}

impl Deref for SysRegister {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for SysRegister {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
