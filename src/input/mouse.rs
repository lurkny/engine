use std::collections::HashSet;
use winit::dpi::PhysicalPosition;
use winit::event::{ButtonId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};

#[derive(Default)]
pub struct Mouse {
    position: PhysicalPosition<f64>,
    curr_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    scroll_delta: (f32, f32),
}

impl Mouse {
    const SCROLL_LINE_HEIGHT: f32 = 16.0;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.scroll_delta = (0.0, 0.0);
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> Result<(), String> {
        match &event {
            WindowEvent::CursorMoved { position, .. } => Ok(self.position = *position),
            WindowEvent::MouseInput { state, button, .. } => {
                self.process_button_event(state, button)
            }
            WindowEvent::MouseWheel { delta, .. } => Ok(self.process_wheel_event(*delta)),
            _ => Err(String::from("Unsupported Event")),
        }
    }

    fn process_wheel_event(&mut self, delta: MouseScrollDelta) {
        self.scroll_delta = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                (x * Self::SCROLL_LINE_HEIGHT, y * Self::SCROLL_LINE_HEIGHT)
            }
            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
        };
    }

    fn process_button_event(
        &mut self,
        state: &ElementState,
        button: &MouseButton,
    ) -> Result<(), String> {
        match state {
            ElementState::Pressed => {
                self.curr_pressed.insert(*button);
                self.just_pressed.insert(*button);
            }
            ElementState::Released => {
                self.curr_pressed.remove(button);
                self.just_released.insert(*button);
            }
        }

        Ok(())
    }

    pub fn position(&self) -> (f64, f64) {
        (self.position.x, self.position.y)
    }

    pub fn is_pressed(&self, button: &MouseButton) -> bool {
        self.curr_pressed.contains(button)
    }

    pub fn is_just_pressed(&self, button: &MouseButton) -> bool {
        self.just_pressed.contains(button)
    }

    pub fn is_just_released(&self, button: &MouseButton) -> bool {
        self.just_released.contains(button)
    }
}
