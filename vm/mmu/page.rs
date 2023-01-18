#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Unmapped,
    Memory {
        offs: usize,
    }
}