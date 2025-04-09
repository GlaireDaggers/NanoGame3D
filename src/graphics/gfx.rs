use std::{mem::transmute, ptr::null_mut};
use gamemath::Mat4;
use crate::misc::{Vector2, Vector3, Vector4};

pub const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: u32 = 0x83F0;
pub const GL_COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
pub const GL_COMPRESSED_RGBA_S3TC_DXT3_EXT: u32 = 0x83F2;

pub fn create_shader(shader_type: u32, shader_src: &str) -> u32 {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        if shader == 0 {
            panic!("Failed to create shader");
        }

        let str_ptr = shader_src.as_ptr() as *const i8;
        let str_len = shader_src.len() as i32;
        gl::ShaderSource(shader, 1, &str_ptr, &str_len);
        gl::CompileShader(shader);

        let mut compile_status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);

        if compile_status == 0 {
            let mut log_length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log_str = vec![0; log_length as usize];
            gl::GetShaderInfoLog(shader, log_length, null_mut(), log_str.as_mut_ptr());

            panic!("=== SHADER COMPILE FAILED ===\n{}\n{}", shader_src, String::from_utf8(transmute(log_str)).unwrap());
        }

        shader
    }
}

pub fn create_program(vtx_shader_src: &str, frag_shader_src: &str) -> u32 {
    unsafe {
        let program = gl::CreateProgram();
        if program == 0 {
            panic!("Failed to create shader program");
        }

        let vtx_shader = create_shader(gl::VERTEX_SHADER, vtx_shader_src);
        let frag_shader = create_shader(gl::FRAGMENT_SHADER, frag_shader_src);

        gl::AttachShader(program, vtx_shader);
        gl::AttachShader(program, frag_shader);

        gl::LinkProgram(program);

        let mut link_status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);

        if link_status == 0 {
            let mut log_length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log_str = vec![0; log_length as usize];
            gl::GetProgramInfoLog(program, log_length, null_mut(), log_str.as_mut_ptr());

            panic!("=== SHADER LINK FAILED ===\n{}\n{}\n{}", vtx_shader_src, frag_shader_src, String::from_utf8(transmute(log_str)).unwrap());
        }

        gl::DetachShader(program, vtx_shader);
        gl::DetachShader(program, frag_shader);
        gl::DeleteShader(vtx_shader);
        gl::DeleteShader(frag_shader);

        program
    }
}

pub fn set_uniform_float(program: u32, name: &str, value: f32) {
    unsafe {
        let uniform_location = gl::GetUniformLocation(program, name.as_ptr() as *const i8);
        gl::Uniform1f(uniform_location, value);
    }
}

pub fn set_uniform_vec2(program: u32, name: &str, value: Vector2) {
    unsafe {
        let uniform_location = gl::GetUniformLocation(program, name.as_ptr() as *const i8);
        gl::Uniform2f(uniform_location, value.x, value.y);
    }
}

pub fn set_uniform_vec3(program: u32, name: &str, value: Vector3) {
    unsafe {
        let uniform_location = gl::GetUniformLocation(program, name.as_ptr() as *const i8);
        gl::Uniform3f(uniform_location, value.x, value.y, value.z);
    }
}

pub fn set_uniform_vec4(program: u32, name: &str, value: Vector4) {
    unsafe {
        let uniform_location = gl::GetUniformLocation(program, name.as_ptr() as *const i8);
        gl::Uniform4f(uniform_location, value.x, value.y, value.z, value.w);
    }
}

pub fn set_uniform_mat4(program: u32, name: &str, value: Mat4) {
    unsafe {
        let uniform_location = gl::GetUniformLocation(program, name.as_ptr() as *const i8);
        gl::UniformMatrix4fv(uniform_location, 1, 0, value.rows.as_ptr() as *const f32);
    }
}