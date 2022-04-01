use gl::types::{GLboolean, GLenum};
use std::ffi::CString;
use std::{mem, ptr};

#[derive(Clone, Copy)]
pub(crate) struct Shader {
    id: u32,
}

impl Shader {
    pub(crate) fn new(vertex_source: &str, fragment_source: &str) -> Self {
        let vertex = compile(vertex_source, "vertex", gl::VERTEX_SHADER);
        let fragment = compile(fragment_source, "fragment", gl::FRAGMENT_SHADER);

        unsafe {
            let program = gl::CreateProgram();

            gl::AttachShader(program, vertex);
            gl::AttachShader(program, fragment);
            gl::LinkProgram(program);

            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success as *mut _);

            if success as GLboolean == gl::FALSE {
                let mut info_log = [0; 512];
                gl::GetProgramInfoLog(program, 512, ptr::null_mut(), info_log.as_mut_ptr());
                let info_log = mem::transmute::<[i8; 512], [u8; 512]>(info_log);
                panic!(
                    "shader program linking failed:\n{}",
                    std::str::from_utf8(&info_log).unwrap()
                );
            }

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            Self { id: program }
        }
    }

    pub(crate) fn use_shader(self) {
        unsafe { gl::UseProgram(self.id) };
    }

    pub(crate) fn set_bool(self, uniform: Uniform, value: bool) {
        unsafe { gl::Uniform1i(self.uniform_location(uniform), value as i32) }
    }

    pub(crate) fn set_int(self, uniform: Uniform, value: i32) {
        unsafe { gl::Uniform1i(self.uniform_location(uniform), value) }
    }

    pub(crate) fn set_float(self, uniform: Uniform, value: f32) {
        unsafe { gl::Uniform1f(self.uniform_location(uniform), value) }
    }

    fn uniform_location(self, uniform: Uniform) -> i32 {
        unsafe { gl::GetUniformLocation(self.id, uniform.name) }
    }
}

#[macro_export]
macro_rules! uniform {
    ($name:literal) => {{
        assert!(!$name.contains('\0'));
        $crate::shader::Uniform { name: concat!($name, "\0").as_bytes().as_ptr() as *const i8 }
    }};
}

pub(crate) struct Uniform {
    pub(crate) name: *const i8,
}

fn compile(source: &str, name: &str, type_: GLenum) -> u32 {
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
