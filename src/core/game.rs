use crate::graphics::Frame;
use crate::input::Input;

pub trait Game {
    fn update(&mut self, input: &Input, dt: f64);
    fn render(&self, frame: &mut Frame);
}
