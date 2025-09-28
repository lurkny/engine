use crate::core::Application;

mod core;
mod graphics;
mod input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = Application::new("My Game Engine", 800, 600);
    app.run()
}
