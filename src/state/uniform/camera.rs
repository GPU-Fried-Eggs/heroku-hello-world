use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use crate::state::camera::{Camera, Projection};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(in crate::state) struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(in crate::state) fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Matrix4::identity().into(),
        }
    }

    pub(in crate::state) fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}
