use crate::graphics::Renderer;
use crate::input::Input;
use pollster::block_on;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;
use winit::dpi::LogicalSize;
use winit::event_loop::ControlFlow;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub struct Application {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: Input,
    title: String,
    size: (u32, u32),
    last_update: Instant,
    accumulator: Duration,
    fps: Duration,
}

impl Application {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            window: None,
            renderer: None,
            title: title.into(),
            size: (width, height),
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
            fps: Duration::from_secs_f64(1.0 / 60.0),
            input: Input::new(),
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    pub fn update(&mut self, dt: f64) {}

    pub fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            if let Some(mut frame) = renderer.begin_frame() {
                frame.clear(crate::graphics::Color::rgb(0.2, 0.3, 0.8)); // Blue

                frame.draw_circle(0.15, 32, crate::graphics::Color::BLUE);

                frame.present();
            }
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(&self.title)
                .with_inner_size(LogicalSize::new(self.size.0, self.size.1));

            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => Arc::new(window),
                Err(e) => panic!("Failed to create window: {}", e),
            };

            let renderer = block_on(Renderer::new(window.clone()));

            self.window = Some(window);
            self.renderer = Some(renderer);
            info!("Window created");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input.keyboard.process_event(&event);
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(renderer) = &mut self.renderer {
                    let size = self.window.as_ref().unwrap().inner_size();
                    renderer.resize(size);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.input.update();

        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;
        self.accumulator += delta;

        while self.accumulator >= self.fps {
            self.update(self.fps.as_secs_f64());
            self.accumulator -= self.fps;
        }

        self.render();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
