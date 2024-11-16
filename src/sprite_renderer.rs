use crate::shaders;
use crate::sprite::Sprite;
use gl::types::*;
use glam::{Mat4, Vec3};

pub struct SpriteRenderer {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl SpriteRenderer {
    pub fn new() -> Self {
        let vertex_shader =
            shaders::compile_shader(gl::VERTEX_SHADER, shaders::SPRITE_VERTEX_SHADER);
        let fragment_shader =
            shaders::compile_shader(gl::FRAGMENT_SHADER, shaders::SPRITE_FRAGMENT_SHADER);
        let program = shaders::link_program(vertex_shader, fragment_shader);

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // 버텍스 데이터 구조 변경 (위치와 텍스처 좌표)
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (24 * std::mem::size_of::<f32>()) as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // 위치 속성
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // 텍스처 좌표 속성
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(1);
        }

        Self { program, vao, vbo }
    }

    pub fn draw_sprite(&self, sprite: &Sprite, projection: &Mat4) {
        unsafe {
            gl::UseProgram(self.program);

            let frame_dimensions = sprite.get_frame_dimensions();

            let model = Mat4::from_scale_rotation_translation(
                Vec3::new(frame_dimensions.x * sprite.get_pixel_scale(), frame_dimensions.y * sprite.get_pixel_scale(), 1.0),
                glam::Quat::from_rotation_z(sprite.rotation),
                sprite.position.extend(0.0),
            );

            let model_loc = gl::GetUniformLocation(self.program, b"model\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.to_cols_array().as_ptr());

            let projection_loc =
                gl::GetUniformLocation(self.program, b"projection\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(
                projection_loc,
                1,
                gl::FALSE,
                projection.to_cols_array().as_ptr(),
            );

            sprite.texture.bind();

            // 버텍스 데이터 업데이트
            let vertices: [f32; 24] = [
                // 위치      // 텍스처 좌표
                0.0, 1.0, sprite.tex_coords[0].x, sprite.tex_coords[0].y,
                1.0, 0.0, sprite.tex_coords[2].x, sprite.tex_coords[2].y,
                0.0, 0.0, sprite.tex_coords[3].x, sprite.tex_coords[3].y,
                0.0, 1.0, sprite.tex_coords[0].x, sprite.tex_coords[0].y,
                1.0, 1.0, sprite.tex_coords[1].x, sprite.tex_coords[1].y,
                1.0, 0.0, sprite.tex_coords[2].x, sprite.tex_coords[2].y,
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
