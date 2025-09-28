mod keyboard;

use keyboard::Keyboard;

pub struct Input {
    pub keyboard: Keyboard,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keyboard: Keyboard::new(),
        }
    }
    pub fn update(&mut self) {
        self.keyboard.update();
    }
}
