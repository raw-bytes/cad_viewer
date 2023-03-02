use anyhow::bail;
use cad_import::structure::Material;
use glow::HasContext;
use log::debug;
use nalgebra_glm::{Mat3, Mat4};

use crate::gl_call;

pub struct Shader<C: HasContext> {
    program: Option<C::Program>,
    uniform_combined_mat: C::UniformLocation,
    uniform_normal_mat: C::UniformLocation,
    uniform_diffuse_color: C::UniformLocation,
}

impl<C: HasContext> Shader<C> {
    /// Creates a new instance of the shader.
    ///
    /// # Arguments
    /// * `context` - The OpenGL context used for creating and compiling the shader
    /// * `shader_version` - The version string for the shader code.
    pub fn new(context: &C, shader_version: &str) -> anyhow::Result<Self> {
        debug!("Create shader program...");
        let program: C::Program = match gl_call!(context, create_program) {
            Ok(program) => program,
            Err(err) => {
                bail!("Failed to create shader program due to {}", err);
            }
        };

        debug!("Compile shader source...");
        let shader_sources = [
            (glow::VERTEX_SHADER, include_str!("shaders/shader.vert")),
            (glow::FRAGMENT_SHADER, include_str!("shaders/shader.frag")),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());
        for (shader_type, shader_source) in shader_sources.iter() {
            let shader_id = match gl_call!(context, create_shader, *shader_type) {
                Ok(id) => id,
                Err(err) => {
                    bail!(
                        "Failed to compile shader {} due to {}",
                        Self::shader_name(*shader_type),
                        err
                    );
                }
            };

            gl_call!(
                context,
                shader_source,
                shader_id,
                &format!("{}\n{}", shader_version, shader_source)
            );

            debug!("Compile shader {} ...", Self::shader_name(*shader_type));
            gl_call!(context, compile_shader, shader_id);
            if !gl_call!(context, get_shader_compile_status, shader_id) {
                let error_string = gl_call!(context, get_shader_info_log, shader_id);
                bail!(
                    "Shader {} Error: {}",
                    Self::shader_name(*shader_type),
                    error_string
                );
            }

            gl_call!(context, attach_shader, program, shader_id);

            shaders.push(shader_id);
        }

        debug!("Link shader program ...");
        gl_call!(context, link_program, program);
        if !gl_call!(context, get_program_link_status, program) {
            let error_string = gl_call!(context, get_program_info_log, program);
            bail!("Failed linking shader program due to {}", error_string);
        }

        debug!("Shader compilation cleanup ...");
        for shader in shaders {
            gl_call!(context, detach_shader, program, shader);
            gl_call!(context, delete_shader, shader);
        }

        // find uniform shader variables
        let uniform_combined_mat = Self::get_uniform_location(context, program, "combinedMat")?;
        let uniform_normal_mat = Self::get_uniform_location(context, program, "normalMat")?;
        let uniform_diffuse_color = Self::get_uniform_location(context, program, "diffuseColor")?;

        Ok(Shader {
            program: Some(program),
            uniform_combined_mat,
            uniform_normal_mat,
            uniform_diffuse_color,
        })
    }

    /// Tries to find the specified uniform variable.
    fn get_uniform_location(
        context: &C,
        program: C::Program,
        name: &str,
    ) -> anyhow::Result<C::UniformLocation> {
        match gl_call!(context, get_uniform_location, program, name) {
            Some(l) => Ok(l),
            None => {
                bail!("Could not find uniform variable {}", name);
            }
        }
    }

    /// Sets the matrices for the shader uniform variables.
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `combined_mat` - The multiplied projection and model view matrix.
    /// * `normal_mat` - The normal matrix.
    pub fn set_matrices(&self, context: &C, combined_mat: &Mat4, normal_mat: &Mat3) {
        gl_call!(
            context,
            uniform_matrix_4_f32_slice,
            Some(&self.uniform_combined_mat),
            false,
            combined_mat.as_slice()
        );

        gl_call!(
            context,
            uniform_matrix_3_f32_slice,
            Some(&self.uniform_normal_mat),
            false,
            normal_mat.as_slice()
        );
    }

    /// Sets uniform variable for the given material.
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `material` - The material data to set.
    pub fn set_material(&self, context: &C, material: &Material) {
        match material {
            Material::PhongMaterial(p) => {
                gl_call!(
                    context,
                    uniform_3_f32_slice,
                    Some(&self.uniform_diffuse_color),
                    p.diffuse_color.0.as_slice()
                );
            }
            Material::None => {
                gl_call!(
                    context,
                    uniform_3_f32,
                    Some(&self.uniform_diffuse_color),
                    0f32,
                    0f32,
                    0f32
                );
            }
        }
    }

    /// Binds the shader program to the given context.
    pub fn bind(&self, context: &C) {
        gl_call!(context, use_program, self.program);
    }

    /// Deletes the program object.
    pub fn cleanup(&mut self, context: &C) {
        gl_call!(context, delete_program, self.program.unwrap());
        self.program = None;
    }

    /// returns the name for the given shader type.
    ///
    /// # Arguments
    /// * `shader_type` - The type of the shader
    fn shader_name(shader_type: u32) -> &'static str {
        match shader_type {
            glow::VERTEX_SHADER => "vertex shader",
            glow::FRAGMENT_SHADER => "fragment shader",
            _ => "unknown shader",
        }
    }
}
