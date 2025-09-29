use crate::graphics::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: [f32; 3], color: Color) -> Self {
        Self {
            position,
            color: color.to_array(),
        }
    }
}

pub struct Geometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

pub struct GeometryBuilder;

impl GeometryBuilder {
    pub fn triangle(size: f32, color: Color) -> Geometry {
        let height = size * (3.0_f32.sqrt() / 2.0);
        let vertices = vec![
            Vertex::new([0.0, height / 2.0, 0.0], color),      // Top
            Vertex::new([-size / 2.0, -height / 2.0, 0.0], color),
            Vertex::new([size / 2.0, -height / 2.0, 0.0], color),
        ];
        let indices = vec![0, 1, 2];
        Geometry::new(vertices, indices)
    }

    pub fn rectangle(width: f32, height: f32, color: Color) -> Geometry {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let vertices = vec![
            Vertex::new([-half_width, -half_height, 0.0], color),
            Vertex::new([half_width, -half_height, 0.0], color),
            Vertex::new([half_width, half_height, 0.0], color),
            Vertex::new([-half_width, half_height, 0.0], color),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Geometry::new(vertices, indices)
    }

    pub fn circle(radius: f32, segments: u32, color: Color) -> Geometry {
        let mut vertices = Vec::with_capacity((segments + 1) as usize);
        let mut indices = Vec::with_capacity((segments * 3) as usize);

        // Center vertex
        vertices.push(Vertex::new([0.0, 0.0, 0.0], color));

        // Generate vertices around the circle
        for i in 0..segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            vertices.push(Vertex::new([x, y, 0.0], color));
        }

        // Generate triangles from center to each edge
        for i in 0..segments {
            let next = if i + 1 == segments { 1 } else { i + 2 };
            indices.extend_from_slice(&[0, i + 1, next]);
        }

        Geometry::new(vertices, indices)
    }

    pub fn quad(size: f32, color: Color) -> Geometry {
        Self::rectangle(size, size, color)
    }
}