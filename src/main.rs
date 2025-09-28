use crate::core::Application;

mod core;
mod input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new("My Game Engine", 800, 600);
    app.run()
}