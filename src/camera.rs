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
        let target = self.position + self.ahead();
        Isometry3::look_at_rh(
            &self.position,
            &target,
            &self.up()
        ).to_homogeneous()
    }

    fn compute_projection(&self, aspect_ratio: f32) -> Perspective3<f32> {
        Perspective3::new(aspect_ratio, self.fov, self.near, self.far)
    }
 
    pub fn ahead(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(0., 0., 1.)
    }

    pub fn up(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(0., 1., 0.)
    }
    
    pub fn right(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(1., 0., 0.)
    }
   
    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }
    
    pub fn rotate(&mut self, r: UnitQuaternion<f32>) {
        self.orientation = r * self.orientation;
    }
    
    pub fn roll(&mut self, angle: f32) {
        let axis = self.ahead();
        self.rotate(UnitQuaternion::new(axis * angle));
    }
    
    pub fn pitch(&mut self, angle: f32) {
        let axis = self.right();
        self.rotate(UnitQuaternion::new(axis * angle));
    }
    
    pub fn yaw(&mut self, angle: f32) {
        let axis = self.up();
        self.rotate(UnitQuaternion::new(axis * angle));
    }
    
    pub fn move_right(&mut self, d: f32) {
        self.position += self.right() * d;
    }

    pub fn move_up(&mut self, d: f32) {
        self.position += self.up() * d;
    }

    pub fn move_ahead(&mut self, d: f32) {
        self.position += self.ahead() * d;
    }
}
