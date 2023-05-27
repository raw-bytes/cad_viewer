use crate::{gl_call, gpu_data::GPUData};

use super::{
    bbox::BBox,
    camera::Camera,
    shader::Shader,
    viewer::{ContextConfig, ViewerController},
};

use cad_import::structure::{CADData, Node};
use glow::HasContext;

use glutin::event::{MouseButton, VirtualKeyCode};
use log::{debug, error, info, trace, warn};
use nalgebra_glm::{determinant, inverse, mat4_to_mat3, transpose, vec4_to_vec3, Mat3, Mat4, Vec4};

pub struct Renderer<C: HasContext> {
    shader: Option<Shader<C>>,
    shader_version: String,
    cad_data: CADData,
    scene_volume: BBox,
    camera: Camera,
    gpu_data: GPUData<C>,
    width: u32,
    height: u32,
}

impl<C: HasContext> Renderer<C> {
    pub fn new(cad_data: CADData) -> Self {
        let gpu_data = GPUData::new();
        let mut scene_volume = BBox::new();
        Self::compute_bbox(
            cad_data.get_root_node(),
            Mat4::identity(),
            &mut scene_volume,
        );

        let mut camera = Camera::new();
        camera.focus(&scene_volume).unwrap();

        Self {
            shader: None,
            shader_version: String::new(),
            cad_data,
            scene_volume,
            camera,
            gpu_data,
            width: 0,
            height: 0,
        }
    }

    fn compute_normal_matrix(m: &Mat4) -> Mat3 {
        let m = mat4_to_mat3(m);

        let d: f32 = determinant(&m);
        if d.abs() <= 1e-9 {
            m
        } else {
            transpose(&inverse(&m))
        }
    }

    /// Computes the bounding volume for the given node and all its children recursively.
    ///
    /// # Arguments
    /// * `node` - The node which defines the subtree for which the bounding volume will be computed.
    /// * `transform` - The transformation to be applied
    /// * `bbox` - Mutable reference for the bounding volume to be updated.
    fn compute_bbox(node: &Node, transform: Mat4, bbox: &mut BBox) {
        // update transformation
        let transform = match node.get_transform() {
            Some(t) => transform * t,
            None => transform,
        };

        // compute bounding volume for all parts
        for shape in node.get_shapes() {
            for part in shape.get_parts() {
                let mesh = part.get_mesh();
                let positions = mesh.get_vertices().get_positions();

                for p in positions.iter() {
                    let p = vec4_to_vec3(&(transform * Vec4::new(p.0.x, p.0.y, p.0.z, 1f32)));
                    bbox.extend_pos(&p);
                }
            }
        }

        // iterate over all children and update the global
        for child in node.get_children() {
            Self::compute_bbox(child, transform, bbox);
        }
    }
}

impl<C: HasContext> ViewerController<C> for Renderer<C> {
    fn initialize(&mut self, context: &C, context_config: ContextConfig) -> anyhow::Result<()> {
        info!("Initialize Renderer...");

        gl_call!(context, enable, glow::DEPTH_TEST);

        self.shader_version = context_config.shader_version;
        self.width = context_config.width;
        self.height = context_config.height;

        info!("Shader Version: {}", self.shader_version);
        self.shader = Some(Shader::new(context, &self.shader_version)?);

        info!("Transfer CPU data to GPU...");
        self.gpu_data.add_cad_data(context, &self.cad_data)?;

        Ok(())
    }

    fn draw(&mut self, context: &C) {
        trace!("Draw");
        gl_call!(
            context,
            viewport,
            0,
            0,
            self.width as i32,
            self.height as i32
        );
        gl_call!(context, clear_color, 0.2, 0.2, 1.0, 1.0);
        gl_call!(
            context,
            clear,
            glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT
        );

        let shader = match &self.shader {
            Some(shader) => {
                shader.bind(context);
                shader
            }
            None => {
                warn!("Draw aborted -> Shader not ready");
                return;
            }
        };

        self.camera.update_window_size(self.width, self.height);
        let model_view_matrix = self.camera.get_data().get_model_matrix();
        let projection_matrix = self.camera.get_data().get_projection_matrix();

        let combined_mat = projection_matrix * model_view_matrix;

        for instance in self.gpu_data.get_instances() {
            let normal_mat = Self::compute_normal_matrix(&(model_view_matrix * instance.transform));
            let final_combined_mat = combined_mat * instance.transform;

            shader.set_matrices(
                context,
                &model_view_matrix,
                &final_combined_mat,
                &normal_mat,
            );

            let shape = &self.gpu_data.get_shapes()[instance.shape_index];

            for part in shape.parts.iter() {
                shader.set_material(context, &part.material);

                let normals_enabled = part.mesh.has_normals();
                shader.set_attributes(context, normals_enabled);

                part.mesh.draw(context);
            }
        }

        gl_call!(context, use_program, None);
    }

    fn cleanup(&mut self, context: &C) {
        info!("Clean up...");
        match &mut self.shader {
            Some(s) => s.cleanup(context),
            _ => {}
        }
    }

    fn resize(&mut self, _context: &C, width: u32, height: u32) {
        debug!("resize ({}, {})", width, height);

        self.width = width;
        self.height = height;
    }

    fn cursor_move(&mut self, x: f64, y: f64) {
        self.camera.update_mouse_motion(x, y);
    }

    fn keyboard_event(&mut self, virtual_key: VirtualKeyCode, pressed: bool) {
        match (virtual_key, pressed) {
            (VirtualKeyCode::A, true) => {
                info!("Show all");
                match self.camera.focus(&self.scene_volume) {
                    Err(err) => {
                        error!("Failed to focus on scene due to {}", err);
                    }
                    _ => {}
                }
            }
            (VirtualKeyCode::C, true) => {
                info!("Export Camera...");

                let cam_data = self.camera.get_data();
                println!("{}", cam_data.to_string());

                let model_matrix = cam_data.get_model_matrix();
                let proj_matrix = cam_data.get_projection_matrix();

                println!("\"model_view_matrix\":\n{:?}", model_matrix.as_slice());
                println!("\"projection_matrix\":\n{:?}", proj_matrix.as_slice());
            }
            _ => {}
        }
    }

    fn mouse_button(&mut self, x: f64, y: f64, button: MouseButton, pressed: bool) {
        self.camera.update_mouse_button(x, y, button, pressed);
    }
}
