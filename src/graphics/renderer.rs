use std::io::pipe;
use super::pipeline::RenderPipeline;
use super::{Color, Geometry, GeometryBuilder, GraphicsContext};
use std::iter;
use std::sync::Arc;
use wgpu::{
    CommandEncoder, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceTexture, TextureView,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::graphics::transform::Transform;
use crate::graphics::uniform::{TransformUniform, UniformBuffer};
use glam::Mat4;

pub struct Renderer {
    context: GraphicsContext,
    pipeline: RenderPipeline,
    uniform_buffer: UniformBuffer,
    projection: Mat4
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let context = GraphicsContext::new(window).await;
        let pipeline = RenderPipeline::new(&context.device, &context.config);
        let uniform_buffer = UniformBuffer::new(
            &context.device,
            pipeline.get_bind_group_layout()
        );

        let width = context.config.width as f32;
        let height = context.config.height as f32;
        let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);

        Self { context, pipeline, uniform_buffer, projection }
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
            uniform_buffer: &self.uniform_buffer,
            projection: self.projection
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.context.resize(new_size);
        let width = new_size.width as f32;
        let height = new_size.height as f32;
        self.projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);
    }
}

pub struct Frame<'a> {
    surface_texture: SurfaceTexture,
    view: TextureView,
    encoder: CommandEncoder,
    context: &'a GraphicsContext,
    pipeline: &'a RenderPipeline,
    uniform_buffer: &'a UniformBuffer,
    projection: Mat4
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
                    load: LoadOp::Clear(color.into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    pub fn draw_geometry(&mut self, geometry: &Geometry, transform: Transform) {
        self.context.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[TransformUniform::new(transform.to_matrix(), self.projection)])
        );
        let (vertex_buffer, index_buffer) =
            self.pipeline.create_buffers(&self.context.device, geometry);

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
        render_pass.set_bind_group(0, &self.uniform_buffer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..geometry.indices.len() as u32, 0, 0..1);
    }

    pub fn draw_triangle(&mut self, size: f32, color: Color, transform: Transform) {
        let geometry = GeometryBuilder::triangle(size, color);
        self.draw_geometry(&geometry, transform);
    }

    pub fn draw_rectangle(&mut self, width: f32, height: f32, color: Color, transform: Transform) {
        let geometry = GeometryBuilder::rectangle(width, height, color);
        self.draw_geometry(&geometry, transform);
    }

    pub fn draw_circle(&mut self, radius: f32, segments: u32, color: Color, transform: Transform) {
        let geometry = GeometryBuilder::circle(radius, segments, color);
        self.draw_geometry(&geometry, transform);
    }

    pub fn draw_quad(&mut self, size: f32, color: Color, transform: Transform) {
        let geometry = GeometryBuilder::quad(size, color);
        self.draw_geometry(&geometry, transform);
    }

    pub fn present(self) {
        self.context.queue.submit(iter::once(self.encoder.finish()));
        self.surface_texture.present();
    }
}
