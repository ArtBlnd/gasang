pub trait FixedBytesArray: Sized {
    const SIZE: usize;

    fn from_bytes(from: &[u8]) -> Self;
}

impl<const LEN: usize> FixedBytesArray for [u8; LEN] {
    const SIZE: usize = LEN;

    fn from_bytes(from: &[u8]) -> Self {
        from[..LEN].try_into().unwrap()
    }
}
