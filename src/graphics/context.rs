use std::sync::Arc;
use wgpu::wgt::DeviceDescriptor;
use wgpu::{
    Backends, Device, Features, Limits, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GraphicsContext {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

impl GraphicsContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = match instance.create_surface(window) {
            Ok(surface) => surface,
            Err(e) => panic!("Failed to create surface: {:?}", e),
        };

        let adapter = match instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(e) => panic!("Failed to create adapter: {:?}", e),
        };

        let (device, queue) = match adapter
            .request_device(&DeviceDescriptor {
                label: Some("GPU Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                ..Default::default()
            })
            .await
        {
            Ok(device) => device,
            Err(e) => panic!("Failed to create device: {:?}", e),
        };

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
