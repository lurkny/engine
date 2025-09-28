use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use std::collections::HashSet;


#[derive(Default, Debug)]
pub struct Keyboard {
    curr_pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self::default()
    }
}
