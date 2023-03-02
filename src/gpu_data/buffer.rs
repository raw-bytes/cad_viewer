use anyhow::Result;
use glow::HasContext;

use crate::{gl_call, viewer::gl_call::handle_glow_error};

/// The usage for the buffer data.
#[repr(u32)]
pub enum Usage {
    Static = glow::STATIC_DRAW,
    Dynamic = glow::DYNAMIC_DRAW,
    Stream = glow::STREAM_DRAW,
}

/// A single GPU Buffer.
pub struct Buffer<C: HasContext, const TARGET: u32> {
    buffer: C::Buffer,
}

impl<C: HasContext, const TARGET: u32> Buffer<C, TARGET> {
    /// Creates new empty gpu buffer.
    pub fn new(context: &C) -> Result<Self> {
        let buffer = handle_glow_error(gl_call!(context, create_buffer))?;

        Ok(Self { buffer })
    }

    /// Copies the given slice of data onto the GPU.
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `data` - The data to copied into the GPU buffer.
    pub fn set_data<T: Sized>(&self, context: &C, data: &[T], usage: Usage) {
        let num_bytes = data.len() * std::mem::size_of::<T>();
        let data = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, num_bytes) };

        gl_call!(context, bind_buffer, TARGET, Some(self.buffer));
        gl_call!(context, buffer_data_u8_slice, TARGET, data, usage as u32);
    }

    /// Binds the buffer
    pub fn bind(&self, context: &C) {
        gl_call!(context, bind_buffer, TARGET, Some(self.buffer));
    }
}
