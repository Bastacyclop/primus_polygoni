use nalgebra::{self, Point3, Vector3, Isometry3, Perspective3, Matrix4,
               UnitQuaternion};

pub struct Camera {
    position: Point3<f32>,
    orientation: UnitQuaternion<f32>,

    near: f32,
    far: f32,
    fov: f32,

    transform: Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3::new(0., 0., -2.),
            orientation: nalgebra::one(),

            near: 0.1,
            far: 10.,
            fov: (70f32).to_radians(),

            transform: nalgebra::one(),
        }
    }

    pub fn update(&mut self, aspect_ratio: f32) {
        let view = self.compute_view();
        let projection = self.compute_projection(aspect_ratio);
        self.transform = projection.as_matrix() * view;
    }

    fn compute_view(&self) -> Matrix4<f32> {
        Isometry3::look_at_rh(
            &self.position,
            &(self.position + self.orientation * Vector3::z()),
            &(self.orientation * Vector3::y())
        ).to_homogeneous()
    }

    fn compute_projection(&self, aspect_ratio: f32) -> Perspective3<f32> {
        Perspective3::new(aspect_ratio, self.fov, self.near, self.far)
    }

    pub fn gpu_transform(&self) -> [[f32; 4]; 4] {
        use std::mem;
        // FIXME
        let ptr = self.transform.as_slice().as_ptr();
        unsafe { *mem::transmute::<_, &[[f32; 4]; 4]>(ptr) }
    }

    fn rotate(&mut self, r: UnitQuaternion<f32>) {
        self.orientation = r * self.orientation;
    }

    pub fn roll(&mut self, angle: f32) {
        let axis = self.orientation * -Vector3::z();
        self.rotate(UnitQuaternion::new(axis * angle));
    }

    pub fn pitch(&mut self, angle: f32) {
        let axis = self.orientation * -Vector3::x();
        self.rotate(UnitQuaternion::new(axis * angle));
    }

    pub fn yaw(&mut self, angle: f32) {
        let axis = self.orientation * Vector3::y();
        self.rotate(UnitQuaternion::new(axis * angle));
    }

    pub fn move_right(&mut self, d: f32) {
        self.position += self.orientation * Vector3::x() * -d;
    }

    pub fn move_up(&mut self, d: f32) {
        self.position += self.orientation * Vector3::y() * d;
    }

    pub fn move_ahead(&mut self, d: f32) {
        self.position += self.orientation * Vector3::z() * d;
    }
}
