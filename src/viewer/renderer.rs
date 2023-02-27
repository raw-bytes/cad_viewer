use std::path::{Path, PathBuf};

use crate::gl_call;

use super::viewer::{ContextConfig, ViewerController};

use glow::{Context, HasContext};

use log::{debug, info, trace};

pub struct Renderer {
    input_file: PathBuf,
    shader_version: String,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(input_file: &Path) -> Self {
        Self {
            input_file: input_file.to_owned(),
            shader_version: String::new(),
            width: 0,
            height: 0,
        }
    }
}

impl ViewerController for Renderer {
    fn initialize(
        &mut self,
        context: &Context,
        context_config: ContextConfig,
    ) -> anyhow::Result<()> {
        self.shader_version = context_config.shader_version;
        self.width = context_config.width;
        self.height = context_config.height;

        info!("Initialize renderer...");
        info!("Shader Version: {}", self.shader_version);
        Ok(())
    }

    fn draw(&mut self, context: &Context) {
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
        gl_call!(context, clear, glow::COLOR_BUFFER_BIT);
    }

    fn cleanup(&mut self, context: &Context) {}

    fn resize(&mut self, _context: &Context, width: u32, height: u32) {
        debug!("resize ({}, {})", width, height);

        self.width = width;
        self.height = height;
    }
}
