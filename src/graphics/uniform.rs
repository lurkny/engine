use std::num::NonZeroU64;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct TransformUniform {
    matrix: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new(matrix: Mat4, projection: Mat4) -> Self {
        Self {
            matrix: matrix.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
        }
    }
}

#[derive(Debug)]
pub struct UniformPool {
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub current_offset: usize,
    pub alignment: usize,
    pub buffer_size: usize,
    pub current_buffer: usize,
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
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Uniform Buffer {i}")),
                size: buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bind Group {i}")),
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: Some(NonZeroU64::new(aligned_size as u64).unwrap()),
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

        let offset = self.current_offset;
        queue.write_buffer(
            &self.buffers[self.current_buffer],
            offset as u64,
            bytemuck::cast_slice(&[*uniform]),
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
