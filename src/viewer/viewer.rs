use anyhow::Result;
use glow::Context;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    ContextBuilder, ContextWrapper, PossiblyCurrent,
};

pub trait ViewerController {
    fn initialize(&mut self, context: &Context, shader_version: &str) -> Result<()>;
    fn draw(&mut self, context: &Context);
    fn resize(&mut self, context: &Context, width: u32, height: u32);
    fn cleanup(&mut self, context: &Context);
}

/// The 3D viewer component
pub struct Viewer<C>
where
    C: 'static,
{
    event_loop: EventLoop<()>,
    window: ContextWrapper<PossiblyCurrent, Window>,
    gl: Context,
    controller: C,
    shader_version: String,
}

impl<C: ViewerController> Viewer<C> {
    /// Creates and returns a new viewer with the given title.
    pub fn new(title: &str, controller: C) -> Result<Self> {
        let (gl, shader_version, window, event_loop) = unsafe {
            let event_loop = EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title(title)
                .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
            let window = ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (gl, "#version 410", window, event_loop)
        };

        let viewer = Viewer {
            event_loop,
            window,
            gl,
            controller,
            shader_version: shader_version.to_owned(),
        };

        Ok(viewer)
    }

    /// Runs the internal viewer main loop. The function blocks until the viewer has been closed.
    pub fn run(self) -> Result<()> {
        let viewer = self;

        let event_loop = viewer.event_loop;
        let window = viewer.window;
        let gl = viewer.gl;
        let shader_version = viewer.shader_version;
        let mut controller = viewer.controller;

        controller.initialize(&gl, &shader_version)?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::LoopDestroyed => {
                    return;
                }
                Event::MainEventsCleared => {
                    window.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    controller.draw(&gl);
                    window.swap_buffers().unwrap();
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        controller.resize(
                            &gl,
                            physical_size.width as u32,
                            physical_size.height as u32,
                        );
                        window.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        controller.cleanup(&gl);
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
