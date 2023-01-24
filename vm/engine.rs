use crate::image::Image;

pub trait Engine {
    fn init(&mut self, info: Image);
    fn run(&mut self) -> u64;
}
