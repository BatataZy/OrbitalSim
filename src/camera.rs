use cgmath::{Point3, Rad, Matrix4, Vector3, InnerSpace, perspective, SquareMatrix, Quaternion, Deg};
use instant::Duration;
use winit::{event::{ElementState, MouseScrollDelta, MouseButton}, dpi::PhysicalPosition};

#[rustfmt::skip] //Translation from OpenGL viewing matrix format to WGPU's format
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    target: Point3<f32>,
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, T: Into<Point3<f32>>, F: Into<Rad<f32>>>(
        position: V,
        target: T,
        aspect: f32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            position: position.into(),
            target: target.into(),
            aspect,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

//Creates the viewing matrix so everything is rendered properly – magically works
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        
        let view = Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y());
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.calc_matrix().into();
    }
}

#[derive(Debug)]
pub struct CameraController {
    is_right_pressed: f32,
    is_middle_pressed: f32,
    horizontal: f32,
    vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    scroll_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32, scroll_sensitivity: f32) -> Self {
        Self{
            is_right_pressed: 0.0,
            is_middle_pressed: 0.0,
            horizontal: 0.0,
            vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            scroll_sensitivity,
        }
    }

//Tells if right or middle mouse click are, well... clicked
    pub fn process_buttons(&mut self, button: MouseButton, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {1.0} else {0.0};

        match button {
            MouseButton::Right => {self.is_right_pressed = amount; true}
            MouseButton::Middle => {self.is_middle_pressed = amount; true}
            _ => false
        }
    }

//Gets the x and y components of the mouse's speed
    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.horizontal = (mouse_dx) as f32;
        self.vertical = (mouse_dy) as f32;
    }

//Gets the scroll speed
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 50.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32
        }
    }

//Get Camera'd!!!
    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        
        let _dt = dt.as_secs_f32();
        let fwd = camera.position - camera.target;
        let fwd_nor =fwd.normalize();

    //YAW
    
        let current_mag = fwd.magnitude();

        //Rotates a unit vector around the y axis
        let next_pos_h = Quaternion::new(0.707 * self.sensitivity * self.horizontal, 0.0, 1.0, 0.0) * fwd_nor;

        //Rotates a unit vector around the axis perpendicular to the view
        //The ifs make sure you stay from 5 to 175 degrees – things get whacky at the poles and we don't want that
        let next_pos_v = if fwd.angle(Vector3::unit_y()) >= Deg(5.0).into() && self.vertical >= 0.0 {
            Quaternion::new(-0.707 * self.vertical * self.sensitivity, fwd_nor.cross(Vector3::unit_y()).normalize().x, 0.0, fwd_nor.cross(Vector3::unit_y()).normalize().z) * fwd_nor
        } else if fwd.angle(Vector3::unit_y()) <= Deg(175.0).into() && self.vertical <= 0.0 {
            Quaternion::new(-0.707 * self.vertical * self.sensitivity, fwd_nor.cross(Vector3::unit_y()).normalize().x, 0.0, fwd_nor.cross(Vector3::unit_y()).normalize().z) * fwd_nor
        } else {-fwd_nor};
        
        //This combines vertical and horizontal rotation. Horizontal's y component is flipped so it doesn't break
        let next_pos = (next_pos_v + Vector3::new(next_pos_h.x, -next_pos_h.y, next_pos_h.z)).normalize();
        
        //Updates the camera position to be at a fixed a distance from the "target"
        if self.is_right_pressed != 0.0 {camera.position = camera.target - next_pos * current_mag};

    //TRANSLATION

        //Moves both the camera and the target in an axis perpendicular to the view
        camera.position += self.horizontal * self.speed * self.is_middle_pressed * (fwd_nor.cross(Vector3::unit_y())).normalize() * fwd.magnitude();
        camera.target += self.horizontal * self.speed * self.is_middle_pressed * (fwd_nor.cross(Vector3::unit_y())).normalize() * fwd.magnitude();

        //Moves both the camera and the target in the y axis
        camera.position.y += self.vertical * self.speed * self.is_middle_pressed * fwd.magnitude();
        camera.target.y += self.vertical * self.speed * self.is_middle_pressed * fwd.magnitude();

    //ZOOM

        //The ifs make sure you don't go too far –20 units max– or too close –1 unit min–
        if fwd.magnitude() < 20.0 && self.scroll > 0.0 {camera.position += fwd_nor * self.scroll * self.scroll_sensitivity} else if fwd.magnitude() > 1.0  && self.scroll < 0.0 {camera.position += fwd_nor * self.scroll * self.scroll_sensitivity} else {};
        
    //RESETTING VARIABLES
        self.scroll = 0.0;    
        self.horizontal = 0.0;
        self.vertical = 0.0;
        
    }
}