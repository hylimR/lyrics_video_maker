use anyhow::{Result, anyhow};
use glutin::{
    context::{ContextApi, ContextAttributesBuilder, Version, NotCurrentGlContext},
    display::{GetGlDisplay, GlDisplay},
    config::ConfigTemplateBuilder,
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub struct GlutinWindowContext {
    pub window: winit::window::Window,
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    // This is hard to abstract because EventLoop type is generic. 
    // We will keep the main logic in main.rs for now or simplify.
}
