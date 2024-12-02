use super::{TileLayer, TileMap};
use gl::types::*;
use glam::Mat4;

#[allow(unused)]
pub struct TileMapRenderer {
    shader_program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    instance_vbo: GLuint,
    max_instances: usize,
    vertex_data: Vec<f32>,
    instance_data: Vec<f32>,
}

impl TileMapRenderer {
    pub fn new(max_instances: usize) -> Result<Self, String> {
        let vertex_shader =
            compile_shader(gl::VERTEX_SHADER, TILEMAP_VERTEX_SHADER)?;
        let fragment_shader =
            compile_shader(gl::FRAGMENT_SHADER, TILEMAP_FRAGMENT_SHADER)?;
        let shader_program = link_program(vertex_shader, fragment_shader)?;

        let mut vao = 0;
        let mut vbo = 0;
        let mut instance_vbo = 0;

        // Default rectangle vertex data
        let vertex_data = vec![
            // positions    // texture coords
            0.0, 1.0, 0.0, 1.0, // top left
            1.0, 0.0, 1.0, 0.0, // bottom right
            0.0, 0.0, 0.0, 0.0, // bottom left
            0.0, 1.0, 0.0, 1.0, // top left
            1.0, 1.0, 1.0, 1.0, // top right
            1.0, 0.0, 1.0, 0.0, // bottom right
        ];

        unsafe {
            // Create and bind VAO
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Set vertex buffer
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Set vertex attributes
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            // Set instance buffer
            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (max_instances * 8 * std::mem::size_of::<f32>()) as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Set instance attributes
            // position & scale
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl::VertexAttribDivisor(2, 1);

            // UV coordinates
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                4,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as GLsizei,
                (4 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::VertexAttribDivisor(3, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(Self {
            shader_program,
            vao,
            vbo,
            instance_vbo,
            max_instances,
            vertex_data,
            instance_data: Vec::with_capacity(max_instances * 8),
        })
    }

    pub fn render(
        &mut self,
        tilemap: &TileMap,
        projection: &Mat4,
    ) {
        unsafe {
            gl::UseProgram(self.shader_program);

            // Set projection matrix
            let projection_loc = gl::GetUniformLocation(
                self.shader_program,
                b"projection\0".as_ptr() as *const _,
            );
            gl::UniformMatrix4fv(
                projection_loc,
                1,
                gl::FALSE,
                projection.to_cols_array().as_ptr(),
            );

            tilemap.tileset.bind_texture();

            // Render each layer
            for (_, layer) in &tilemap.layers {
                if !layer.visible {
                    continue;
                }

                self.render_layer(tilemap, layer);
            }
        }
    }

    fn render_layer(
        &mut self,
        tilemap: &TileMap,
        layer: &TileLayer,
    ) {
        self.instance_data.clear();
        let mut instance_count = 0;
        let scaled_tile_size = tilemap.tile_size as f32 * tilemap.scale;

        // Render only visible tiles (viewport culling)
        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                if let Some(tile) = layer.get_tile(x, y) {
                    if instance_count >= self.max_instances {
                        self.flush_batch();
                        instance_count = 0;
                    }

                    let world_pos = tilemap.tile_to_world(x, y);

                    if let Some(uvs) = tilemap.tileset.get_tile_uvs(tile.id) {
                        // Add instance data
                        self.instance_data.extend_from_slice(&[
                            world_pos.x,
                            world_pos.y, // position
                            scaled_tile_size,
                            scaled_tile_size, // scale
                            uvs[0].x,
                            uvs[0].y, // UV 좌표
                            uvs[2].x,
                            uvs[2].y,
                        ]);

                        instance_count += 1;
                    }
                }
            }
        }

        if instance_count > 0 {
            self.flush_batch();
        }
    }

    fn flush_batch(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.instance_data.len() * std::mem::size_of::<f32>())
                    as GLsizeiptr,
                self.instance_data.as_ptr() as *const _,
            );

            gl::DrawArraysInstanced(
                gl::TRIANGLES,
                0,
                6, // 6 vertices per tile
                (self.instance_data.len() / 8) as GLsizei, // instance count
            );
        }
    }
}

const TILEMAP_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;
    layout (location = 2) in vec4 aInstance;  // pos & scale
    layout (location = 3) in vec4 aInstanceTexCoords;  // UV coordinates

    out vec2 TexCoords;

    uniform mat4 projection;

    void main()
    {
        vec2 pos = aPos * aInstance.zw + aInstance.xy;
        gl_Position = projection * vec4(pos, 0.0, 1.0);
        
        vec2 texPos = aTexCoords;
        TexCoords = mix(aInstanceTexCoords.xy, aInstanceTexCoords.zw, texPos);
    }
"#;

const TILEMAP_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    in vec2 TexCoords;
    out vec4 FragColor;

    uniform sampler2D texture0;

    void main()
    {
        FragColor = texture(texture0, TexCoords);
    }
"#;

// shader compilation helper function
fn compile_shader(
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

        Ok(shader)
    }
}

// shader program linking helper function
fn link_program(
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
