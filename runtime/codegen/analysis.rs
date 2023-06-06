mod variable_liveness;
pub use variable_liveness::*;
mod ir_cost;
pub use ir_cost::*;

pub trait Analysis {
    type Output;

    fn analyze(&self) -> Self::Output;
}
