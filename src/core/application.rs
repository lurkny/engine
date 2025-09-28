use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event_loop::ControlFlow;

pub struct Application {
    window: Option<Arc<Window>>,
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
            title: title.into(),
            size: (width, height),
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
            fps: Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    pub fn update(&mut self, dt: f64) {

    }

    pub fn render(&mut self) {

    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.size.0,
                    self.size.1
                ));

            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => window,
                Err(e) => panic!("Failed to create window: {}", e),
            };

            self.window = Some(Arc::new(window));
            println!("Window created");
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
                println!("Close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                //TODO Rendering
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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