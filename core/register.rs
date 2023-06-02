use std::hash::Hash;

/// A register representation
pub trait Register: Copy + Clone + PartialEq + Eq + Sized {
    type Id: RegisterId;

    fn parent(&self) -> Self::Id;
    fn id(&self) -> Self::Id;
    fn size(&self) -> usize;

    /// Returns true if the register is a read-only register
    fn is_read_only(&self) -> bool;
}

pub trait RegisterId: Copy + Clone + PartialEq + Eq + Sized + Hash {
    fn raw(&self) -> RawRegisterId;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RawRegisterId(usize);

impl RawRegisterId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}
