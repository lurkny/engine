use crate::core::game::Game;
use crate::graphics::{Color, Renderer};
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
    game: Box<dyn Game>,
    input: Input,
    title: String,
    size: (u32, u32),
    last_update: Instant,
    accumulator: Duration,
    fps: Duration,
}

impl Application {
    pub fn new(
        title: impl Into<String>,
        width: u32,
        height: u32,
        game: impl Game + 'static,
    ) -> Self {
        Self {
            window: None,
            renderer: None,
            game: Box::new(game),
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

    pub fn update(&mut self, dt: f64) {
        self.game.update(&self.input, dt);
    }

    pub fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer
            && let Some(mut frame) = renderer.begin_frame()
        {
            frame.clear(Color::rgb(0.1, 0.1, 0.15));
            self.game.render(&mut frame);
            frame.present();
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
            WindowEvent::CursorMoved { position, .. } => self.input.mouse.move_cursor(position),
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.mouse.process_button_event(&state, &button)
            }
            WindowEvent::MouseWheel { delta, .. } => self.input.mouse.process_wheel_event(delta),

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;
        self.accumulator += delta;

        while self.accumulator >= self.fps {
            self.update(self.fps.as_secs_f64());
            self.accumulator -= self.fps;
        }

        self.render();

        // Clear per-frame input state (just_pressed / just_released / scroll)
        // after the game has had a chance to read it.
        self.input.update();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
