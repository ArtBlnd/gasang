use crate::image::Image;

pub trait Engine {
    fn init(image: Image) -> Self;
    fn run(&mut self) -> u64;
}
