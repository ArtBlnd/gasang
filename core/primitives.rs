use num_traits::PrimInt;

pub trait Primitive: PrimInt {}
impl<T> Primitive for T where T: PrimInt {}
