use crate::shader::link_program;
use crate::sprite::sprite_shaders;
use crate::{shader::compile_shader, sprite::sprite::Sprite};
use gl::types::*;
use glam::{Mat4, Vec3};

pub struct SpriteRenderer {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl SpriteRenderer {
    pub fn new() -> Self {
        let vertex_shader = compile_shader(
            gl::VERTEX_SHADER,
            sprite_shaders::SPRITE_VERTEX_SHADER,
        )
        .expect("Failed to compile vertex shader");
        let fragment_shader = compile_shader(
            gl::FRAGMENT_SHADER,
            sprite_shaders::SPRITE_FRAGMENT_SHADER,
        )
        .expect("Failed to compile fragment shader");
        let program = link_program(vertex_shader, fragment_shader)
            .expect("Failed to link program");

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);

            // Create and bind VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Change vertex data structure (position and texture coordinates)
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (24 * std::mem::size_of::<f32>()) as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Position attribute
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self { program, vao, vbo }
    }

    pub fn draw_sprite(
        &self,
        sprite: &Sprite,
        projection: &Mat4,
    ) {
        unsafe {
            gl::UseProgram(self.program);

            let scale = Vec3::new(
                sprite.sprite_size.width * sprite.get_pixel_scale(),
                sprite.sprite_size.height * sprite.get_pixel_scale(),
                1.0,
            );

            let model = Mat4::from_scale_rotation_translation(
                scale,
                glam::Quat::from_rotation_z(sprite.rotation),
                sprite.position.extend(0.0),
            );

            let model_loc = gl::GetUniformLocation(
                self.program,
                b"model\0".as_ptr() as *const _,
            );
            gl::UniformMatrix4fv(
                model_loc,
                1,
                gl::FALSE,
                model.to_cols_array().as_ptr(),
            );

            let projection_loc = gl::GetUniformLocation(
                self.program,
                b"projection\0".as_ptr() as *const _,
            );
            gl::UniformMatrix4fv(
                projection_loc,
                1,
                gl::FALSE,
                projection.to_cols_array().as_ptr(),
            );

            let use_color_key_loc = gl::GetUniformLocation(
                self.program,
                b"useColorKey\0".as_ptr() as *const _,
            );

            if let Some(color_key) = &sprite.color_key {
                let color_key_loc = gl::GetUniformLocation(
                    self.program,
                    b"colorKey\0".as_ptr() as *const _,
                );
                let threshold_loc = gl::GetUniformLocation(
                    self.program,
                    b"threshold\0".as_ptr() as *const _,
                );

                gl::Uniform1i(use_color_key_loc, 1);
                gl::Uniform3f(
                    color_key_loc,
                    color_key.color.x,
                    color_key.color.y,
                    color_key.color.z,
                );
                gl::Uniform1f(threshold_loc, color_key.threshold);
            } else {
                gl::Uniform1i(use_color_key_loc, 0);
            }

            gl::ActiveTexture(gl::TEXTURE0);
            sprite.texture.bind();

            // Update vertex data
            let vertices: [f32; 24] = [
                // Pos
                // Texture coordinate
                0.0,
                1.0,
                sprite.tex_coords[0].x,
                sprite.tex_coords[0].y,
                1.0,
                0.0,
                sprite.tex_coords[2].x,
                sprite.tex_coords[2].y,
                0.0,
                0.0,
                sprite.tex_coords[3].x,
                sprite.tex_coords[3].y,
                0.0,
                1.0,
                sprite.tex_coords[0].x,
                sprite.tex_coords[0].y,
                1.0,
                1.0,
                sprite.tex_coords[1].x,
                sprite.tex_coords[1].y,
                1.0,
                0.0,
                sprite.tex_coords[2].x,
                sprite.tex_coords[2].y,
            ];

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
            );

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}
