use anyhow::Result;
use glow::{Context, HasContext};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    ContextBuilder, ContextWrapper, PossiblyCurrent,
};

/// The configuration of the context.
pub struct ContextConfig {
    /// The shader version
    pub shader_version: String,

    /// The initial width of the context
    pub width: u32,

    /// The initial height of the context
    pub height: u32,
}

/// The trait for the viewer controller
pub trait ViewerController<C: HasContext> {
    /// Initialize call to allocate all OpenGL resource
    fn initialize(&mut self, context: &C, config: ContextConfig) -> Result<()>;

    /// Draws a single frame
    fn draw(&mut self, context: &C);

    /// Resize update of the frame
    fn resize(&mut self, context: &C, width: u32, height: u32);

    /// Final cleanup call to remove all GL resources.
    fn cleanup(&mut self, context: &C);
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
    context_config: ContextConfig,
}

impl<C: ViewerController<Context>> Viewer<C> {
    /// Creates and returns a new viewer with the given title.
    pub fn new(title: &str, controller: C) -> Result<Self> {
        let width: u32 = 1024;
        let height: u32 = 768;

        let (gl, shader_version, window, event_loop) = unsafe {
            let event_loop = EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title(title)
                .with_inner_size(glutin::dpi::LogicalSize::new(width as f32, height as f32));
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
            context_config: ContextConfig {
                shader_version: shader_version.to_owned(),
                width,
                height,
            },
        };

        Ok(viewer)
    }

    /// Runs the internal viewer main loop. The function blocks until the viewer has been closed.
    pub fn run(self) -> Result<()> {
        let viewer = self;

        let event_loop = viewer.event_loop;
        let window = viewer.window;
        let gl = viewer.gl;
        let context_config = viewer.context_config;
        let mut controller = viewer.controller;

        controller.initialize(&gl, context_config)?;

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
