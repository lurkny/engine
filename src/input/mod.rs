mod keyboard;
mod mouse;

use keyboard::Keyboard;
use mouse::Mouse;

pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }
    pub fn update(&mut self) {
        self.keyboard.update();
    }
}
