use core::fmt;
use std::hash::Hash;

pub trait Register: Copy + Clone + PartialEq + Eq + Sized + Hash {
    fn raw(&self) -> RawRegisterId;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RawRegisterId(usize);

impl RawRegisterId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl fmt::Display for RawRegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
