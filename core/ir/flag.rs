#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Flag {
    /// Zero flag
    ZF,
    /// Carry flag
    CF,
    /// Overflow flag
    OF,
}

impl Flag {
    pub fn into_index(self) -> usize {
        match self {
            Self::ZF => 0,
            Self::CF => 1,
            Self::OF => 2,
        }
    }

    pub fn max() -> usize {
        64
    }
}
