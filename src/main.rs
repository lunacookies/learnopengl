use cocoa::appkit::{NSColorSpace, NSWindow};
use cocoa::base::{id, nil};
use gl::types::{GLboolean, GLenum};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::macos::WindowExtMacOS;
use glutin::window::{Window, WindowBuilder};
use glutin::{Api, ContextBuilder, GlRequest};
use std::ffi::{c_void, CString};
use std::{mem, ptr};

const VERTEX_SHADER_SOURCE: &str = include_str!("vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("fragment.glsl");

fn main() {
    let el = EventLoop::new();

    let wb = WindowBuilder::new();

    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(wb, &el)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    set_window_color_space_to_srgb(context.window());

    gl::load_with(|s| context.get_proc_address(s));

    let shader_program = compile_shaders(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    let size = context.window().inner_size();
    unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32) };

    let vertices: &[f32] = &[
        0.5, 0.5, 0.0, // top right
        0.5, -0.5, 0.0, // bottom right
        -0.5, -0.5, 0.0, // bottom left
        -0.5, 0.5, 0.0, // top left
    ];
    let indices: &[u32] = &[0, 1, 3, 1, 2, 3];

    let mut vbo = 0;
    let mut ebo = 0;
    let mut vao = 0;

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<f32>() * vertices.len()) as isize,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mem::size_of::<u32>() * indices.len()) as isize,
            indices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<f32>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    el.run(move |event, _, cf| {
        *cf = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                WindowEvent::Resized(size) => unsafe {
                    context.resize(size);
                    gl::Viewport(0, 0, size.width as i32, size.height as i32);
                },
                _ => {}
            },

            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);

                    gl::UseProgram(shader_program);
                    gl::BindVertexArray(vao);
                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as i32,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                    );
                    gl::BindVertexArray(0);

                    gl::Finish();
                }
                context.swap_buffers().unwrap();
            }

            _ => {}
        }
    });
}

fn compile_shaders(
    vertex_shader_source: &'static str,
    fragment_shader_source: &'static str,
) -> u32 {
    let vertex_shader = compile_shader(vertex_shader_source, "vertex", gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fragment_shader_source, "fragment", gl::FRAGMENT_SHADER);

    unsafe {
        let shader_program = gl::CreateProgram();

        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success as *mut _);

        if success as GLboolean == gl::FALSE {
            let mut info_log = [0; 512];
            gl::GetProgramInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr());
            let info_log = mem::transmute::<[i8; 512], [u8; 512]>(info_log);
            panic!("shader program linking failed:\n{}", std::str::from_utf8(&info_log).unwrap());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    }
}

fn compile_shader(source: &'static str, name: &str, type_: GLenum) -> u32 {
    let source = CString::new(source).unwrap();
    let source = &(source.as_ptr() as *const i8) as *const _;

    unsafe {
        let shader = gl::CreateShader(type_);
        gl::ShaderSource(shader, 1, source, ptr::null());
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success as *mut _);

        if success as GLboolean == gl::FALSE {
            let mut info_log = [0; 512];
            gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), info_log.as_mut_ptr());
            let info_log = mem::transmute::<[i8; 512], [u8; 512]>(info_log);
            panic!(
                "{name} shader compilation failed:\n{}",
                std::str::from_utf8(&info_log).unwrap()
            );
        }

        shader
    }
}

fn set_window_color_space_to_srgb(window: &Window) {
    unsafe {
        let color_space = NSColorSpace::sRGBColorSpace(nil);
        NSWindow::setColorSpace_(window.ns_window() as id, color_space);
    }
}
