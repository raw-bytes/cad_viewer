use anyhow::bail;
use nalgebra_glm::{
    column, determinant, dot, inverse_transpose, mat3_to_mat4, mat4_to_mat3, normalize,
    perspective, translation, transpose, Mat3, Mat4, Vec3, Vec4,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub struct CameraData {
    center: Vec3,
    cam_axis: Mat3,
    radius: f32,
    window_size: (u32, u32),

    scene_center: Vec3,
    scene_radius: f32,
}

impl ToString for CameraData {
    fn to_string(&self) -> String {
        let center = [self.center[0], self.center[1], self.center[2]];

        let cam_axis: [f32; 9] = self.cam_axis.as_slice().try_into().unwrap();

        let s = SerializedCameraData {
            radius: self.radius,
            center,
            cam_axis,
        };

        let result = serde_json::to_string(&s).unwrap();

        result
    }
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

    /// Sets the camera data by parsing the given string that is in a JSON format.
    pub fn set_from_string(&mut self, s: &str) -> anyhow::Result<()> {
        let s: SerializedCameraData = serde_json::from_str(s)?;

        self.radius = s.radius;
        self.center.copy_from_slice(&s.center);
        self.cam_axis.copy_from_slice(&s.cam_axis);

        Ok(())
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

    pub fn set_cam_axis(&mut self, cam_axis: Mat3) {
        self.cam_axis = cam_axis;
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

/// Struct for serializing the camera data
#[derive(Serialize, Deserialize)]
struct SerializedCameraData {
    pub center: [f32; 3],
    pub cam_axis: [f32; 9],
    pub radius: f32,
}

#[cfg(test)]
mod test {
    use nalgebra_glm::{Mat3, Vec3};

    use super::CameraData;

    #[test]
    fn test_serialization() {
        let mut cam_data = CameraData::new();
        cam_data.set_center(&Vec3::new(1f32, 2f32, 3f32));

        let r = Mat3::from_columns(&[
            Vec3::new(0f32, 0f32, 1f32),
            Vec3::new(1f32, 0f32, 0f32),
            Vec3::new(1f32, 0f32, 0f32),
        ]);

        cam_data.set_cam_axis(r);
        cam_data.set_radius(42f32);

        let s: String = cam_data.to_string();

        let mut cam_data2 = CameraData::new();
        cam_data2.set_from_string(&s).unwrap();

        assert_eq!(cam_data2.get_radius(), 42f32);
        assert_eq!(*cam_data2.get_center(), Vec3::new(1f32, 2f32, 3f32));
        assert_eq!(*cam_data2.get_axis(), r);
    }
}
