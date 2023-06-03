mod variable_liveness;
pub use variable_liveness::*;

pub trait Analysis {
    type Output;

    fn analyze(&self) -> Self::Output;
}
