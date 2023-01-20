mod translate;
pub use translate::*;

use crate::VmState;

pub struct AArch64VM {
    state: VmState,
}
