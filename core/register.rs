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
