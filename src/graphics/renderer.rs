use std::io::pipe;
use super::pipeline::RenderPipeline;
use super::{Color, Geometry, GeometryBuilder, GraphicsContext, Vertex};
use std::iter;
use std::num::NonZeroU64;
use std::sync::Arc;
use wgpu::{BindGroupLayout, CommandEncoder, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::graphics::transform::Transform;
use crate::graphics::uniform::{TransformUniform, UniformBuffer};
use glam::Mat4;
use wgpu::util::DeviceExt;
use wgpu::wgc::command::DrawKind::Draw;

pub struct Renderer {
    context: GraphicsContext,
    pipeline: RenderPipeline,
    uniform_pool: UniformPool,
    projection: Mat4
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let context = GraphicsContext::new(window).await;
        let pipeline = RenderPipeline::new(&context.device, &context.config);
        let uniform_pool = UniformPool::new(
            &context.device,
            pipeline.get_bind_group_layout()
        );

        let width = context.config.width as f32;
        let height = context.config.height as f32;
        let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);

        Self { context, pipeline, uniform_pool, projection }
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
            draw_queue: Vec::new()
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.context.resize(new_size);
        let width = new_size.width as f32;
        let height = new_size.height as f32;
        self.projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);
    }
}

#[derive(Debug)]
pub struct UniformPool {
    buffers: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    current_offset: usize,
    alignment: usize,
    buffer_size: usize,
    current_buffer: usize
}

impl UniformPool {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let alignment = device.limits().min_storage_buffer_offset_alignment as usize;
        let uniform_size = std::mem::size_of::<TransformUniform>();
        let aligned_size = ((uniform_size + alignment - 1) / alignment) * alignment;

        const NUM_BUFFERS: usize = 3;
        const UNIFORMS_PER_BUFFER: usize = 1000;

        let buffer_size = aligned_size * UNIFORMS_PER_BUFFER;
        let mut buffers = Vec::with_capacity(NUM_BUFFERS);
        let mut bind_groups = Vec::with_capacity(NUM_BUFFERS);

        for i in 0..NUM_BUFFERS {

            let buffer = device.create_buffer(&wgpu::BufferDescriptor{
                label: Some(&format!("Uniform Buffer {i}")),
                size: buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bind Group {i}")),
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: Some(NonZeroU64::new(aligned_size as u64).unwrap())
                    }),
                }],
            });
            buffers.push(buffer);
            bind_groups.push(bind_group);
        }

        Self {
            buffers,
            bind_groups,
            current_offset: 0,
            alignment: aligned_size,
            buffer_size,
            current_buffer: 0,
        }
    }

    pub fn allocate(&mut self, queue: &wgpu::Queue, uniform: &TransformUniform) -> (usize, u32) {
        if self.current_offset + self.alignment > self.buffer_size {
            panic!("Buffer Overflow");
        }

        let offset= self.current_offset;
        queue.write_buffer(
            &self.buffers[self.current_buffer],
            offset as u64,
            bytemuck::cast_slice(&[*uniform])
        );

        let result = (self.current_buffer, offset as u32);
        self.current_offset += self.alignment;
        result
    }

    pub fn next_frame(&mut self) {
        self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
        self.current_offset = 0;
    }
}

#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub transform: Transform
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
    draw_queue: Vec<DrawCommand>
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
        self.draw_queue.push(DrawCommand {
            vertices: geometry.vertices.clone(),
            indices: geometry.indices.clone(),
            transform
        });
    }

    pub fn present(mut self) {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut draw_calls = Vec::new();


        let mut vertex_offset = 0u32;
        let mut index_offset = 0u32;

        for cmd in &self.draw_queue {
            let uniform = TransformUniform::new(cmd.transform.to_matrix(), self.projection);
            let (bind_group_idx, offset) = self.uniform_pool.allocate(
                &self.context.queue,
                &uniform
            );

            all_vertices.extend_from_slice(&cmd.vertices);

            for &idx in &cmd.indices {
                all_indices.push(idx + vertex_offset);
            }

            draw_calls.push(DrawCall {
                bind_group_index: bind_group_idx,
                uniform_offset: offset,
                index_start: index_offset,
                index_count: cmd.indices.len() as u32,
            });

            vertex_offset += cmd.vertices.len() as u32;
            index_offset += cmd.indices.len() as u32;
        }

        let vertex_buffer = self.context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Frame Vertex Buffer"),
                contents: bytemuck::cast_slice(&all_vertices),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let index_buffer = self.context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Frame Index Buffer"),
                contents: bytemuck::cast_slice(&all_indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store
                },
                depth_slice: None
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
                &[draw.uniform_offset]
            );

            render_pass.draw_indexed(
                draw.index_start..(draw.index_start + draw.index_count),
                0,
                0..1
            );
        }

        drop(render_pass);

        self.context.queue.submit(iter::once(self.encoder.finish()));
        self.surface_texture.present();

        self.uniform_pool.next_frame();
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

}
