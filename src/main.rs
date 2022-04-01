mod shader;
mod texture;

use cocoa::appkit::{NSColorSpace, NSWindow};
use cocoa::base::{id, nil};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::macos::WindowExtMacOS;
use glutin::window::{Window, WindowBuilder};
use glutin::{Api, ContextBuilder, GlRequest};
use shader::Shader;
use std::ffi::c_void;
use std::{mem, ptr};
use texture::{Channels, Format, Texture};

const VERTEX_SHADER_SOURCE: &str = include_str!("shader.vs");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("shader.fs");
const CONTAINER: &[u8] = include_bytes!("container.jpeg");
const AWESOME_FACE: &[u8] = include_bytes!("awesome_face.png");

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

    let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    let size = context.window().inner_size();
    unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32) };

    let vertices: &[f32] = &[
        // positions   // colors      // texture coords
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
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
            8 * mem::size_of::<f32>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<f32>() as i32,
            (3 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<f32>() as i32,
            (6 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    let texture1 = Texture::new(CONTAINER, Format::Jpeg, Channels::Rgb);
    let texture2 = Texture::new(AWESOME_FACE, Format::Png, Channels::Rgba);

    shader.use_shader();
    shader.set_int(uniform!("texture1"), 0);
    shader.set_int(uniform!("texture2"), 1);

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

                    gl::ActiveTexture(gl::TEXTURE0);
                    texture1.bind();
                    gl::ActiveTexture(gl::TEXTURE1);
                    texture2.bind();

                    shader.use_shader();
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

fn set_window_color_space_to_srgb(window: &Window) {
    unsafe {
        let color_space = NSColorSpace::sRGBColorSpace(nil);
        NSWindow::setColorSpace_(window.ns_window() as id, color_space);
    }
}
