use cgmath::SquareMatrix;


pub struct Camera {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Camera {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        Camera {
            left,
            right,
            bottom,
            top,
        }
    }

    pub fn build_matrix(&self) -> cgmath::Matrix4<f32> {
        let ortho: cgmath::Matrix4<f32> = cgmath::Ortho {
            left:   self.left,
            right:  self.right,
            bottom: self.bottom,
            top:    self.top,
            near:   -1.0,
            far:    1.0,
        }.into();

        return OPENGL_TO_WGPU_MATRIX * ortho;
    }

    pub fn modify_position(&mut self, x: f32, y: f32) {
        self.left += x;
        self.right += x;
        self.bottom += y;
        self.top += y;
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into()
        }
    }

    pub fn update_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_matrix().into();
    }
}