use glam::{Mat4, Vec2, Vec3};
use winit::dpi::Position;

type Radians = f32;
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: Radians,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0));

        translation * rotation * scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
