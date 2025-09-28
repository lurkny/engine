use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, PhysicalKey};

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

    pub fn update(&mut self) {
        self.just_released.clear();
        self.just_released.clear();
    }

    pub fn process_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.curr_pressed.insert(keycode);
                    self.just_pressed.insert(keycode);
                }
                ElementState::Released => {
                    self.curr_pressed.remove(&keycode);
                    self.just_released.insert(keycode);
                }
            }
        }
    }

    pub fn is_pressed(&self, key: &KeyCode) -> bool {
        self.curr_pressed.contains(key)
    }

    pub fn is_just_pressed(&self, key: &KeyCode) -> bool {
        self.just_pressed.contains(key)
    }

    pub fn is_just_released(&self, key: &KeyCode) -> bool {
        self.just_released.contains(key)
    }
}
