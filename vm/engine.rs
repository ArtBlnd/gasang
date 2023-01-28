use crate::image::Image;

pub trait Engine {
    fn init(image: Image) -> Self;
    unsafe fn run(&mut self) -> u64;
}
