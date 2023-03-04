use anyhow::bail;
use nalgebra_glm::{
    column, determinant, dot, inverse_transpose, mat3_to_mat4, mat4_to_mat3, normalize,
    perspective, translation, transpose, Mat3, Mat4, Vec3, Vec4,
};

#[derive(Clone, Copy)]
pub struct CameraData {
    center: Vec3,
    cam_axis: Mat3,
    radius: f32,
    window_size: (u32, u32),

    scene_center: Vec3,
    scene_radius: f32,
}

impl CameraData {
    /// Returns a new empty camera data object
    pub fn new() -> CameraData {
        let identity_matrix = Mat3::identity();

        CameraData {
            center: Vec3::new(0.0, 0.0, 0.0),
            cam_axis: identity_matrix,
            radius: 0.0,
            window_size: (100, 100),

            scene_center: Vec3::new(0f32, 0f32, 0f32),
            scene_radius: 10f32,
        }
    }

    /// Returns the model view matrix for the camera.
    pub fn get_model_matrix(&self) -> Mat4 {
        let dir: Vec3 = column(&self.cam_axis, 2);

        // compute position of the camera
        let factor = self.radius.exp();
        let cam_pos = self.center + dir * factor;

        // create rotation matrix
        let rot_mat = transpose(&self.cam_axis);
        let rot_mat = mat3_to_mat4(&rot_mat);

        let tmat: Mat4 = translation(&(-cam_pos));
        let res = rot_mat * tmat;

        res
    }

    /// Returns the projection matrix for the camera
    pub fn get_projection_matrix(&self) -> Mat4 {
        let aspect = (self.window_size.0 as f32) / (self.window_size.1 as f32);

        let mmat = self.get_model_matrix();

        // transform the scene center
        let z = -(mmat.row(2)
            * Vec4::new(
                self.scene_center[0],
                self.scene_center[1],
                self.scene_center[2],
                1.0,
            ))[0];

        // determine far plane
        let far = z + self.scene_radius * 1.5;
        let near = (z - self.scene_radius).max(far * 1e-6f32);

        perspective(aspect, 1.0, near, far)
    }

    /// Returns the combined matrix, i.e. the combination of the projection and model view matrix
    pub fn get_combined_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_model_matrix()
    }

    /// Returns the normal matrix
    pub fn get_normal_matrix(&self) -> Mat3 {
        let mat = mat4_to_mat3(&self.get_model_matrix());

        let d: f32 = determinant(&mat);
        if d.abs() <= 1e-9 {
            return mat;
        } else {
            return inverse_transpose(mat);
        }
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        self.window_size
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_axis(&self) -> &Mat3 {
        &self.cam_axis
    }

    pub fn get_center(&self) -> &Vec3 {
        &self.center
    }

    /// Sets the range of the camera data.
    ///
    ///* `center` - The center of the scene.
    ///* `radius` - The radius around the scene center.
    pub fn set_scene(&mut self, center: Vec3, radius: f32) -> anyhow::Result<()> {
        if radius <= 0.0 {
            bail!("Scene radius must be positive!!!");
        }

        self.scene_center = center;
        self.scene_radius = radius;

        Ok(())
    }

    pub fn set_window_size(&mut self, w: u32, h: u32) {
        self.window_size = (w, h);
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn set_center(&mut self, center: &Vec3) {
        self.center = center.clone();
    }

    pub fn set_rotated_cam_axis(&mut self, axis: &Mat3, rot_mat: &Mat3) {
        // rotate x axis
        let c0: Vec3 = normalize(&((*rot_mat) * column(axis, 0)));

        // rotate y axis
        let mut c1 = (*rot_mat) * column(axis, 1);
        c1 = c1 - c0 * dot(&c1, &c0);
        c1 = normalize(&c1);

        // rotate z axis
        let mut c2 = (*rot_mat) * column(axis, 2);

        c2 = c2 - c0 * dot(&c2, &c0);
        c2 = c2 - c1 * dot(&c2, &c1);

        c2 = normalize(&c2);

        self.cam_axis = Mat3::from_columns(&[c0, c1, c2]);
    }
}
