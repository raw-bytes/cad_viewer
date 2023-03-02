use crate::{gl_call, gpu_data::GPUData};

use super::{
    shader::Shader,
    viewer::{ContextConfig, ViewerController},
};

use cad_import::structure::CADData;
use glow::HasContext;

use log::{debug, info, trace, warn};
use nalgebra_glm::{determinant, inverse, mat4_to_mat3, perspective, transpose, Mat3, Mat4};

pub struct Renderer<C: HasContext> {
    shader: Option<Shader<C>>,
    shader_version: String,
    cad_data: CADData,
    gpu_data: GPUData<C>,
    width: u32,
    height: u32,
}

impl<C: HasContext> Renderer<C> {
    pub fn new(cad_data: CADData) -> Self {
        let gpu_data = GPUData::new();

        Self {
            shader: None,
            shader_version: String::new(),
            cad_data: cad_data,
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

        let model_view_matrix = Mat4::identity();
        let aspect: f32 = (self.width as f32) / (self.height as f32);
        let projection_matrix = perspective(aspect, 1f32, 0.01f32, 100f32);

        let combined_mat = projection_matrix * model_view_matrix;

        for instance in self.gpu_data.get_instances() {
            let normal_mat = Self::compute_normal_matrix(&(model_view_matrix * instance.transform));
            let final_combined_mat = combined_mat * instance.transform;

            shader.set_matrices(context, &final_combined_mat, &normal_mat);

            let shape = &self.gpu_data.get_shapes()[instance.shape_index];

            for part in shape.parts.iter() {
                shader.set_material(context, &part.material);

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
}
