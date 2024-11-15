use gl::types::*;
use std::ffi::CString;

pub fn compile_shader(shader_type: GLenum, source: &str) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = Vec::with_capacity(512);
            let mut len = 0;
            gl::GetShaderInfoLog(shader, 512, &mut len, info_log.as_mut_ptr() as *mut GLchar);
            info_log.set_len(len as usize);
            panic!(
                "Shader compilation failed: {}",
                String::from_utf8_lossy(&info_log)
            );
        }
        shader
    }
}

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = Vec::with_capacity(512);
            let mut len = 0;
            gl::GetProgramInfoLog(program, 512, &mut len, info_log.as_mut_ptr() as *mut GLchar);
            info_log.set_len(len as usize);
            panic!(
                "Program linking failed: {}",
                String::from_utf8_lossy(&info_log)
            );
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        program
    }
}

pub const SPRITE_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;
    out vec2 TexCoords;
    uniform mat4 model;
    uniform mat4 projection;

    void main()
    {
        TexCoords = aTexCoords;
        gl_Position = projection * model * vec4(aPos.x, aPos.y, 0.0, 1.0);
    }
"#;

pub const SPRITE_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    in vec2 TexCoords;
    out vec4 color;
    uniform sampler2D image;

    void main()
    {
        color = texture(image, TexCoords);
    }
"#;
