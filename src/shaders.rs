pub static VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec3 aColor;
    out vec3 ourColor;
    uniform mat4 transform;
    void main() {
        gl_Position = transform * vec4(aPos.x, aPos.y, 0.0, 1.0);
        ourColor = aColor;
    }
"#;

pub static FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    in vec3 ourColor;
    void main() {
        FragColor = vec4(ourColor, 1.0f);
    }
"#;

pub fn compile_shader(shader_type: gl::types::GLenum, source: &str) -> gl::types::GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &(source.as_ptr() as *const _), &(source.len() as gl::types::GLint));
        gl::CompileShader(shader);
    }
    shader
}

pub fn link_program(vertex_shader: gl::types::GLuint, fragment_shader: gl::types::GLuint) -> gl::types::GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        program
    }
}