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
    pub fn new(scene_radius: f32) -> Camera {
        let p = Point3::new(0., scene_radius * 0.1, scene_radius * -1.25 - 1.5);
        let target = Point3::new(0., 0., scene_radius * -0.75);

        Camera {
            position: p,
            orientation: UnitQuaternion::rotation_between(
                &Vector3::z(), &(target - p)).unwrap(),

            near: 0.01,
            far: scene_radius * 2.5 + 3.,
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
            &(self.position + self.ahead()),
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

    pub fn rotate(&mut self, r: UnitQuaternion<f32>) {
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

    pub fn move_left(&mut self, angle: f32) {
        let r = UnitQuaternion::new(-Vector3::y() * angle);
        self.position = r * self.position;
        self.orientation = r * self.orientation;
    }

    pub fn move_right(&mut self, angle: f32) {
        let r = UnitQuaternion::new(Vector3::y() * angle);
        self.position = r * self.position;
        self.orientation = r * self.orientation;
    }

    pub fn ahead(&self) -> Vector3<f32> {
        self.orientation * Vector3::z()
    }
}
