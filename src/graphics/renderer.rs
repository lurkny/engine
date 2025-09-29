use super::{Color, GraphicsContext, Geometry, GeometryBuilder};
use super::pipeline::RenderPipeline;
use std::iter;
use std::sync::Arc;
use wgpu::StoreOp::Store;
use wgpu::{
    CommandEncoder, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceTexture, TextureView,
};
use winit::dpi::{PhysicalSize, PhysicalUnit};
use winit::window::Window;

pub struct Renderer {
    context: GraphicsContext,
    pipeline: RenderPipeline,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let context = GraphicsContext::new(window).await;
        let pipeline = RenderPipeline::new(&context.device, &context.config);
        Self { context, pipeline }
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
            pipeline: &self.pipeline,
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
    pipeline: &'a RenderPipeline,
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

    pub fn draw_geometry(&mut self, geometry: &Geometry) {
        let (vertex_buffer, index_buffer) = self.pipeline.create_buffers(&self.context.device, geometry);

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shape Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(self.pipeline.get_pipeline());
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..geometry.indices.len() as u32, 0, 0..1);
    }

    pub fn draw_triangle(&mut self, size: f32, color: Color) {
        let geometry = GeometryBuilder::triangle(size, color);
        self.draw_geometry(&geometry);
    }

    pub fn draw_rectangle(&mut self, width: f32, height: f32, color: Color) {
        let geometry = GeometryBuilder::rectangle(width, height, color);
        self.draw_geometry(&geometry);
    }

    pub fn draw_circle(&mut self, radius: f32, segments: u32, color: Color) {
        let geometry = GeometryBuilder::circle(radius, segments, color);
        self.draw_geometry(&geometry);
    }

    pub fn draw_quad(&mut self, size: f32, color: Color) {
        let geometry = GeometryBuilder::quad(size, color);
        self.draw_geometry(&geometry);
    }

    pub fn present(self) {
        self.context.queue.submit(iter::once(self.encoder.finish()));
        self.surface_texture.present();
    }
}
