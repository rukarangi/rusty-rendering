use cgmath::SquareMatrix;


/*
Shoudl separate out the matrix that implies the size of the screen,
so that it does not have an offset on that for drawing 
and one that transforms it with position for stuff that does move.
*/

pub struct Camera {
    //pub left: f32,
    //pub right: f32,
    //pub bottom: f32,
    //pub top: f32,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new(/*left: f32, right: f32, bottom: f32, top: f32*/width: f32, height: f32, x: f32, y: f32) -> Self {
        Camera {
            //left,
            //right,
            //bottom,
            //top,
            width,
            height,
            x,
            y,
        }
    }

    pub fn position(&self) -> [f32; 2] {
        return [self.x, self.y];
    }

    pub fn build_matrix(&self) -> cgmath::Matrix4<f32> {
        let ortho: cgmath::Matrix4<f32> = cgmath::Ortho {
            left:   0.0,
            right:  self.width,
            bottom: 0.0,
            top:    self.height,
            near:   -1.0,
            far:    1.0,
        }.into();

        return OPENGL_TO_WGPU_MATRIX * ortho;
    }

    pub fn modify_position(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
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
    pub position: [f32; 2],
    buffer: [f32; 2], // for memory layout so that shader accepts
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            position: [0.0, 0.0],
            buffer: [0.0, 0.0]
        }
    }

    pub fn update_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_matrix().into();
        self.position = camera.position();

        println!("{:?}", self);
    }
}