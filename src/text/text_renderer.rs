use gl::types::*;
use unicode_normalization::UnicodeNormalization as _;

use crate::{
    shader::{compile_shader, link_program},
    Texture,
};

use super::{
    text_error::TextError, CharInfo, FontAtlas, TEXT_FRAGMENT_SHADER,
    TEXT_VERTEX_SHADER,
};

pub struct TextRenderer {
    font_atlas: FontAtlas,
    shader_program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    texture: Texture,
}

pub struct TextRendererBuilder {
    font_atlas: Option<FontAtlas>,
}

impl TextRendererBuilder {
    pub fn new() -> Self {
        TextRendererBuilder { font_atlas: None }
    }

    pub fn with_font_atlas(
        mut self,
        font_atlas: FontAtlas,
    ) -> Self {
        self.font_atlas = Some(font_atlas);
        self
    }

    pub fn build(self) -> Result<TextRenderer, TextError> {
        let font_atlas = self
            .font_atlas
            .ok_or(TextError::FontLoadError("font load error".to_string()))?;

        let vertex_shader =
            compile_shader(gl::VERTEX_SHADER, TEXT_VERTEX_SHADER)
                .expect("Failed to compile vertex shader");
        let fragment_shader =
            compile_shader(gl::FRAGMENT_SHADER, TEXT_FRAGMENT_SHADER)
                .expect("Failed to compile fragment shader");

        // Create and compile shaders
        let shader_program = link_program(vertex_shader, fragment_shader)
            .expect("Failed to link program");

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::Enable(gl::MULTISAMPLE);
            let texture = TextRenderer::create_texture_from_atlas(&font_atlas)
                .expect("Failed to create texture from font atlas");

            // Generate VAO and VBO
            gl::GenVertexArrays(1, &mut vao);
            if vao == 0 {
                drop(texture);
                return Err(TextError::FontLoadError(
                    "Failed to create VAO".to_string(),
                ));
            }
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            if vbo == 0 {
                drop(texture);
                gl::DeleteVertexArrays(1, &vao);
                return Err(TextError::FontLoadError(
                    "Failed to create VBO".to_string(),
                ));
            }
            // Setup buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let buffer_size = 24 * std::mem::size_of::<f32>();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                buffer_size as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Vertex attributes
            let stride = 4 * std::mem::size_of::<f32>() as GLsizei;

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            // Add error checking
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("GL Error after buffer setup: {}", error);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            Ok(TextRenderer {
                font_atlas,
                shader_program,
                vao,
                vbo,
                texture,
            })
        }
    }
}

impl TextRenderer {
    pub fn builder() -> TextRendererBuilder {
        TextRendererBuilder::new()
    }

    pub fn new() -> Result<Self, TextError> {
        TextRenderer::builder().build()
    }

    fn create_texture_from_atlas(atlas: &FontAtlas) -> Result<Texture, String> {
        Texture::new_from_data(atlas.image.as_raw(), atlas.width, atlas.height)
    }

    fn render_glyph(
        &self,
        char_info: &CharInfo,
        x: f32,
        y: f32,
        scale: f32,
    ) {
        let x_pos = x + (char_info.xoffset * scale);
        let y_pos = y - (char_info.yoffset * scale);

        let w = char_info.width * scale * self.font_atlas.width as f32;
        let h = char_info.height * scale * self.font_atlas.height as f32;

        let c_x = char_info.x;
        let c_y = char_info.y;

        let c_w = char_info.width;
        let c_h = char_info.height;

        #[rustfmt::skip]
        let vertices: [f32; 24] = [
        //  pos                   tex coords
            x_pos,     y_pos,     c_x,       c_y,       // Left bottom
            x_pos,     y_pos + h, c_x,       c_y + c_h, // Left top
            x_pos + w, y_pos + h, c_x + c_w, c_y + c_h, // Right top
            x_pos + w, y_pos + h, c_x + c_w, c_y + c_h, // Right top
            x_pos + w, y_pos,     c_x + c_w, c_y,       // Right bottom
            x_pos,     y_pos,     c_x,       c_y,       // Left bottom
        ];

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
            );
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn render_text(
        &self,
        text: &str,
        mut x: f32,
        y: f32,
        scale: f32,
        color: [f32; 4],
        projection: &glam::Mat4,
    ) {
        unsafe {
            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vao);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id());

            gl::UniformMatrix4fv(
                gl::GetUniformLocation(
                    self.shader_program,
                    b"projection\0".as_ptr() as *const _,
                ),
                1,
                gl::FALSE,
                projection.to_cols_array().as_ptr(),
            );
            gl::Uniform4fv(
                gl::GetUniformLocation(
                    self.shader_program,
                    b"textColor\0".as_ptr() as *const _,
                ),
                1,
                color.as_ptr(),
            );
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.shader_program,
                    b"text\0".as_ptr() as *const _,
                ),
                0,
            );

            gl::BindVertexArray(self.vao);

            for c in text.nfc() {
                if let Some(char_info) = self.font_atlas.chars.get(&c) {
                    self.render_glyph(char_info, x, y, scale);
                    x += char_info.xadvance * scale;
                }
            }

            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::UseProgram(0);
        }
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            // Unbind everything
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);

            // Clean up resources
            self.texture.delete();
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.shader_program);
        }
    }
}
