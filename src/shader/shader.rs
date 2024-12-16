use gl::types::{GLenum, GLint, GLuint};

pub fn compile_shader(
    shader_type: GLenum,
    source: &str,
) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = std::ffi::CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut _,
            );
            return Err(String::from_utf8(buffer).unwrap());
        }

        println!("Shader compiled successfully");

        Ok(shader)
    }
}

// shader program linking helper function
pub fn link_program(
    vertex_shader: GLuint,
    fragment_shader: GLuint,
) -> Result<GLuint, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        if success != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut _,
            );
            return Err(String::from_utf8(buffer).unwrap());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        Ok(program)
    }
}
