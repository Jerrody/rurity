#![deny(unsafe_op_in_unsafe_fn)]

use erupt::vk;
use raw_window_handle::HasRawWindowHandle;

mod context;
mod renderer;
mod scene;
mod utils;

pub struct Engine {
    renderer: renderer::Renderer,
    context: context::Context,

    #[cfg(all(not(feature = "no_log"), feature = "log"))]
    _logging_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl Engine {
    pub fn new(
        window: &impl HasRawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, vk::Result> {
        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        let logging_guard = utils::logging::init_logging();
        let context = context::Context::new(window, width, height)?;
        let renderer = renderer::Renderer::new();

        Ok(Self {
            context,
            renderer,
            #[cfg(all(not(feature = "no_log"), feature = "log"))]
            _logging_guard: logging_guard,
        })
    }

    #[inline(always)]
    pub fn draw_call(&mut self) -> Result<(), vk::Result> {
        self.renderer.draw(&self.context)?;

        Ok(())
    }
}
