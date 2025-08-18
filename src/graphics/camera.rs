use cgmath::*;
use winit::keyboard::KeyCode;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    //pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let view = cgmath::Matrix4::look_to_rh(
            self.eye,
            cgmath::Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        );
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn update_aspect(&mut self, newaspect: f32) {
        self.aspect = newaspect;
    }
}
// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // MVP Matrix

    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    eye: [f32; 3],
    aspect: f32,
    view_proj: [[f32; 4]; 4],
    //empty: [f32; 1], // for byte alignment
}

impl CameraUniform {
    pub fn new() -> Self {
        //use cgmath::SquareMatrix;
        // default is identity
        Self {
            eye: cgmath::vec3(0., 0., 0.).into(),
            aspect: 0.,
            view_proj: cgmath::Matrix4::identity().into(),
            //empty: [0.; 16],
        }
    }

    // replace view_proj with new projection
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
        self.aspect = camera.aspect.into();
        self.eye = camera.eye.into();
        //self.dir = (camera.target - camera.eye).into();
    }
}

pub struct CameraController {
    speed: f32,
    forward: f32,
    backward: f32,
    right: f32,
    left: f32,
    up: f32,
    down: f32,
    t_h: f32,
    t_v: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed: speed,
            forward: 0.,
            backward: 0.,
            right: 0.,
            left: 0.,
            up: 0.,
            down: 0.,
            t_h: 0.,
            t_v: 0.,
        }
    }

    pub fn process_events(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        let amount = if is_pressed { 1.0 } else { 0.0 };
        match code {
            KeyCode::KeyW => {
                self.forward = amount;
                true
            }
            KeyCode::KeyA => {
                self.left = amount;
                true
            }
            KeyCode::KeyS => {
                self.backward = amount;
                true
            }
            KeyCode::KeyD => {
                self.right = amount;
                true
            }
            KeyCode::KeyE => {
                self.up = amount;
                true
            }
            KeyCode::KeyQ => {
                self.down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn reset(&self, cam: &mut Camera) {
        cam.eye.x = 0.;
        cam.eye.y = 0.;
        cam.eye.z = -1.;
        cam.yaw = cgmath::Deg(90.0).into();
        cam.pitch = cgmath::Deg(0.0).into();
    }

    pub fn handle_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.t_h = mouse_dx as f32;
        self.t_v = mouse_dy as f32;
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        camera.eye += forward * (self.forward - self.backward) * self.speed;
        camera.eye += right * (self.right - self.left) * self.speed;
        camera.eye += camera.up * (self.up - self.down) * self.speed;

        //// Redo radius calc in case the forward/backward is pressed.
        //let forward = camera.target - camera.eye;
        //let forward_norm = forward.normalize();
        //let right = forward_norm.cross(camera.up);

        //if self.right {
        //    camera.eye += right * self.speed;
        //    camera.target += right * self.speed;
        //}
        //if self.left {
        //    camera.eye -= right * self.speed;
        //    camera.target -= right * self.speed;
        //}

        //let up = camera.up;
        //if self.up {
        //    camera.eye += up * self.speed;
        //    camera.target += up * self.speed;
        //}
        //if self.down {
        //    camera.eye -= up * self.speed;
        //    camera.target -= up * self.speed;
        //}
    }
}
