use super::pipeline::RenderPipeline;
use super::{Color, Geometry, GraphicsContext, Vertex};
use crate::graphics::transform::Transform;
use crate::graphics::uniform::{TransformUniform, UniformPool};
use glam::Mat4;
use std::f32::consts::PI;
use std::iter;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{
    CommandEncoder, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceTexture, TextureView,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    context: GraphicsContext,
    pipeline: RenderPipeline,
    uniform_pool: UniformPool,
    projection: Mat4,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let context = GraphicsContext::new(window).await;
        let pipeline = RenderPipeline::new(&context.device, &context.config);
        let uniform_pool = UniformPool::new(&context.device, pipeline.get_bind_group_layout());

        let width = context.config.width as f32;
        let height = context.config.height as f32;
        let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);

        Self {
            context,
            pipeline,
            uniform_pool,
            projection,
        }
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
            uniform_pool: &mut self.uniform_pool,
            projection: self.projection,
            vertex_stream: Vec::with_capacity(10000),
            index_stream: Vec::with_capacity(30000),
            draw_commands: Vec::with_capacity(100),
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.context.resize(new_size);
        let width = new_size.width as f32;
        let height = new_size.height as f32;
        self.projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);
    }
}

#[derive(Debug, Clone)]
pub struct DrawCommand<'a> {
    pub geometry: &'a Geometry,
    pub transform: Transform,
}

struct StreamedDraw {
    vertex_offset: u32,
    index_offset: u32,
    index_count: u32,
    transform: Transform,
}

#[derive(Debug)]
struct DrawCall {
    bind_group_index: usize,
    uniform_offset: u32,
    index_start: u32,
    index_count: u32,
}

pub struct Frame<'a> {
    surface_texture: SurfaceTexture,
    view: TextureView,
    encoder: CommandEncoder,
    context: &'a GraphicsContext,
    pipeline: &'a RenderPipeline,
    uniform_pool: &'a mut UniformPool,
    projection: Mat4,
    vertex_stream: Vec<Vertex>,
    index_stream: Vec<u32>,
    draw_commands: Vec<StreamedDraw>,
}

impl<'a> Frame<'a> {
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
        let vertex_offset = self.vertex_stream.len() as u32;
        let index_offset = self.index_stream.len() as u32;

        self.vertex_stream.extend_from_slice(&geometry.vertices);

        for &idx in &geometry.indices {
            self.index_stream.push(idx + vertex_offset);
        }

        self.draw_commands.push(StreamedDraw {
            vertex_offset,
            index_offset,
            index_count: geometry.indices.len() as u32,
            transform,
        });
    }

    pub fn present(mut self) {
        if self.vertex_stream.is_empty() {
            self.context.queue.submit(iter::once(self.encoder.finish()));
            self.surface_texture.present();
            return;
        }

        let vertex_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Frame Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.vertex_stream),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Frame Index Buffer"),
                    contents: bytemuck::cast_slice(&self.index_stream),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let mut draw_calls = Vec::with_capacity(self.draw_commands.len());

        for cmd in &self.draw_commands {
            let uniform = TransformUniform::new(cmd.transform.to_matrix(), self.projection);
            let (bind_group_idx, offset) =
                self.uniform_pool.allocate(&self.context.queue, &uniform);

            draw_calls.push(DrawCall {
                bind_group_index: bind_group_idx,
                uniform_offset: offset,
                index_start: cmd.index_offset,
                index_count: cmd.index_count,
            });
        }

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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

        for draw in &draw_calls {
            render_pass.set_bind_group(
                0,
                &self.uniform_pool.bind_groups[draw.bind_group_index],
                &[draw.uniform_offset],
            );
            render_pass.draw_indexed(
                draw.index_start..(draw.index_start + draw.index_count),
                0,
                0..1,
            );
        }

        drop(render_pass);

        self.context.queue.submit(iter::once(self.encoder.finish()));
        self.surface_texture.present();

        self.uniform_pool.next_frame();
    }

    pub fn draw_triangle(&mut self, size: f32, color: Color, transform: Transform) {
        let vertex_offset = self.vertex_stream.len() as u32;
        let index_offset = self.index_stream.len() as u32;

        let height = size * (3.0_f32.sqrt() / 2.0);

        self.vertex_stream
            .push(Vertex::new([0.0, height / 2.0, 0.0], color));
        self.vertex_stream
            .push(Vertex::new([-size / 2.0, -height / 2.0, 0.0], color));
        self.vertex_stream
            .push(Vertex::new([size / 2.0, -height / 2.0, 0.0], color));

        self.index_stream
            .extend_from_slice(&[vertex_offset, vertex_offset + 1, vertex_offset + 2]);

        self.draw_commands.push(StreamedDraw {
            vertex_offset,
            index_offset,
            index_count: 3,
            transform,
        });
    }

    pub fn draw_rectangle(&mut self, width: f32, height: f32, color: Color, transform: Transform) {
        let vertex_offset = self.vertex_stream.len() as u32;
        let index_offset = self.index_stream.len() as u32;

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        self.vertex_stream
            .push(Vertex::new([-half_width, -half_height, 0.0], color));
        self.vertex_stream
            .push(Vertex::new([half_width, -half_height, 0.0], color));
        self.vertex_stream
            .push(Vertex::new([half_width, half_height, 0.0], color));
        self.vertex_stream
            .push(Vertex::new([-half_width, half_height, 0.0], color));

        self.index_stream.extend_from_slice(&[
            vertex_offset,
            vertex_offset + 1,
            vertex_offset + 2,
            vertex_offset,
            vertex_offset + 2,
            vertex_offset + 3,
        ]);

        self.draw_commands.push(StreamedDraw {
            vertex_offset,
            index_offset,
            index_count: 6,
            transform,
        });
    }

    pub fn draw_circle(&mut self, radius: f32, segments: u32, color: Color, transform: Transform) {
        let vertex_offset = self.vertex_stream.len() as u32;
        let index_offset = self.index_stream.len() as u32;

        self.vertex_stream.push(Vertex::new([0.0, 0.0, 0.0], color));
        for i in 0..segments {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            self.vertex_stream.push(Vertex::new(
                [radius * angle.cos(), radius * angle.sin(), 0.0],
                color,
            ));
        }

        for i in 0..segments {
            let next = if i + 1 == segments { 0 } else { i + 1 };
            self.index_stream.push(vertex_offset);
            self.index_stream.push(vertex_offset + i + 1);
            self.index_stream.push(vertex_offset + next + 1);
        }

        self.draw_commands.push(StreamedDraw {
            vertex_offset,
            index_offset,
            index_count: segments * 3,
            transform,
        });
    }

    pub fn draw_quad(&mut self, size: f32, color: Color, transform: Transform) {
        self.draw_rectangle(size, size, color, transform);
    }
}
