use super::{Color, GraphicsContext};
use std::iter;
use std::sync::Arc;
use wgpu::{
    CommandEncoder, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceTexture, TextureView,
};
use winit::dpi::{PhysicalSize, PhysicalUnit};
use winit::window::Window;

pub struct Renderer {
    context: GraphicsContext,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let context = GraphicsContext::new(window).await;
        Self { context }
    }

    pub fn begin_frame(&mut self) -> Option<Frame<'_>> {
        let surface_texture = self.context.surface.get_current_texture().ok()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Some(Frame {
            surface_texture,
            view,
            encoder,
            context: &self.context,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.context.resize(new_size);
    }
}

pub struct Frame<'a> {
    surface_texture: SurfaceTexture,
    view: TextureView,
    encoder: CommandEncoder,
    context: &'a GraphicsContext,
}

impl<'a> Frame<'_> {
    pub fn clear(&mut self, color: Color) {
        self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: color.r as f64,
                        g: color.g as f64,
                        b: color.b as f64,
                        a: color.a as f64,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    pub fn present(self) {
        self.context.queue.submit(iter::once(self.encoder.finish()));
        self.surface_texture.present();
    }
}
