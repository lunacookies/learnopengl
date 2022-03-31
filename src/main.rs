use cocoa::appkit::{NSColorSpace, NSWindow};
use cocoa::base::{id, nil};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::macos::WindowExtMacOS;
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};

fn main() {
    let el = EventLoop::new();

    let wb = WindowBuilder::new();

    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };
    let window = context.window();

    unsafe {
        let color_space = NSColorSpace::sRGBColorSpace(nil);
        NSWindow::setColorSpace_(window.ns_window() as id, color_space);
    }

    gl::load_with(|s| context.get_proc_address(s));

    let size = window.inner_size();
    unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32) };

    el.run(move |event, _, cf| {
        *cf = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *cf = ControlFlow::Exit
            }

            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Finish();
                }
                context.swap_buffers().unwrap();
            }

            _ => {}
        }
    });
}
