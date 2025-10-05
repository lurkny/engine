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

pub struct UniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Uniform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::new(
                Mat4::IDENTITY,
                Mat4::IDENTITY,
            )]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    pub fn update(&self, queue: &wgpu::Queue, matrix: Mat4, projection: Mat4) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[TransformUniform::new(matrix, projection)]),
        )
    }
}
