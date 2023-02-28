use crate::gl_call;

use super::{
    shader::Shader,
    viewer::{ContextConfig, ViewerController},
};

use cad_import::structure::CADData;
use glow::HasContext;

use log::{debug, info, trace};

pub struct Renderer<C: HasContext> {
    shader: Option<Shader<C>>,
    shader_version: String,
    width: u32,
    height: u32,
}

impl<C: HasContext> Renderer<C> {
    pub fn new(cad_data: CADData) -> Self {
        Self {
            shader: None,
            shader_version: String::new(),
            width: 0,
            height: 0,
        }
    }
}

impl<C: HasContext> ViewerController<C> for Renderer<C> {
    fn initialize(&mut self, context: &C, context_config: ContextConfig) -> anyhow::Result<()> {
        info!("Initialize Renderer...");

        self.shader_version = context_config.shader_version;
        self.width = context_config.width;
        self.height = context_config.height;

        info!("Shader Version: {}", self.shader_version);
        self.shader = Some(Shader::new(context, &self.shader_version)?);

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

        match &self.shader {
            Some(shader) => {
                shader.bind(context);
            }
            None => {}
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
