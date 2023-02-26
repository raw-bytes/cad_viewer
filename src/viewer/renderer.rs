use std::path::{Path, PathBuf};

use super::viewer::ViewerController;

use glow::Context;

use log::info;

pub struct Renderer {
    input_file: PathBuf,
}

impl Renderer {
    pub fn new(input_file: &Path) -> Self {
        Self {
            input_file: input_file.to_owned(),
        }
    }
}

impl ViewerController for Renderer {
    fn initialize(&mut self, context: &Context, shader_version: &str) -> anyhow::Result<()> {
        info!("Initialize renderer...");
        info!("Shader Version: {}", shader_version);
        Ok(())
    }

    fn draw(&mut self, context: &Context) {}

    fn cleanup(&mut self, context: &Context) {}

    fn resize(&mut self, context: &Context, width: u32, height: u32) {}
}
