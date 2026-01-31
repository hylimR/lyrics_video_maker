use anyhow::{Context, Result};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, GlProfile, NotCurrentGlContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use skia_safe::{
    gpu::{gl::Interface, DirectContext, Protected, SurfaceOrigin},
    ColorType, Surface,
};
use std::num::NonZeroU32;
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

mod app;
// mod window; // Removed for now

use app::PreviewApp;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let window_builder = WindowBuilder::new()
        .with_title("KLyric Preview (Native)")
        .with_inner_size(LogicalSize::new(1280.0, 720.0));

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(false);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let window = window.expect("Failed to create window");
    let raw_window_handle = window.raw_window_handle();
    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let not_current_gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
    };

    let attrs = window.inner_size();
    let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
        .build(
            raw_window_handle,
            NonZeroU32::new(attrs.width).unwrap(),
            NonZeroU32::new(attrs.height).unwrap(),
        );

    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &surface_attributes)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
    };

    let gl_context = not_current_gl_context
        .make_current(&gl_surface)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Load GL
    gl::load_with(|symbol| {
        let symbol = std::ffi::CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
        let symbol = std::ffi::CString::new(name).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    })
    .expect("Failed to create Interface");

    let mut gr_context =
        DirectContext::new_gl(interface, None).expect("Failed to create Skia Context");

    let mut surface = create_skia_surface(&mut gr_context, attrs.width, attrs.height);

    let mut app = PreviewApp::new();

    // Initial resize command
    app.poll_stdin(); // consume initial inputs?

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll); // Poll for high perf? Or Wait?

            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            target.exit();
                        }
                        WindowEvent::Resized(size) => {
                            if size.width > 0 && size.height > 0 {
                                gl_surface.resize(
                                    &gl_context,
                                    NonZeroU32::new(size.width).unwrap(),
                                    NonZeroU32::new(size.height).unwrap(),
                                );

                                // Recreate Skia Surface
                                surface =
                                    create_skia_surface(&mut gr_context, size.width, size.height);

                                window.request_redraw();
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            app.render(surface.canvas());

                            gr_context.flush_and_submit();
                            let _ = gl_surface.swap_buffers(&gl_context);
                        }
                        _ => (),
                    }
                }
                Event::AboutToWait => {
                    if app.poll_stdin() {
                        window.request_redraw();
                    }
                    // If playing, we might need continuous redraw.
                    // For now, rely on SetTime commands flooding us or just explicit redraws.
                    // If we want smooth playback, we might need valid VSync.
                    // Glutin swap_buffers usually waits for VSync if SwapInterval is set.
                }
                _ => (),
            }
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

fn create_skia_surface(ctx: &mut DirectContext, width: u32, height: u32) -> Surface {
    let fb_info = skia_safe::gpu::gl::FramebufferInfo {
        fboid: 0,
        format: skia_safe::gpu::gl::Format::RGBA8.into(),
        protected: Protected::No,
    };

    let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
        (width as i32, height as i32),
        None,
        8,
        fb_info,
    );

    skia_safe::gpu::surfaces::wrap_backend_render_target(
        ctx,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("Failed to create Skia Surface")
}
