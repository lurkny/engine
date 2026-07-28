use engine::core::{Application, Game};
use engine::graphics::{Color, Frame, Transform};
use engine::input::Input;
use glam::Vec2;

struct DemoGame;

impl Game for DemoGame {
    fn update(&mut self, _input: &Input, _dt: f64) {}

    fn render(&self, frame: &mut Frame) {
        let circle_transform = Transform::new(Vec2::new(200.0, 300.0));
        let triangle_transform = Transform::new(Vec2::new(500.0, 300.0));
        frame.draw_circle(50.0, 64, Color::WHITE, circle_transform);
        frame.draw_triangle(100.0, Color::RED, triangle_transform);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = Application::new("My Game Engine", 800, 600, DemoGame);
    app.run()
}
