#[derive(Debug, Clone)]
pub struct NativeInstr<I> {
    pub op: I,
    pub size: u8,
}
