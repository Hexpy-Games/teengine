use gl::types::*;
use rusttype::{point, Font, Scale};
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

use crate::sprite::sprite_shaders;

use super::{TextAlignment, TextStyle, VerticalAlignment};

static DEFAULT_FONT_DATA: &[u8] =
    include_bytes!("../assets/fonts/Pretendard-Medium.ttf");

// 동적 텍스처 아틀라스 관리를 위한 구조체
#[derive(Debug)]
struct AtlasRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug)]
pub struct GlyphInfo {
    pub tex_coords: [f32; 4], // UV 좌표
    pub size: [f32; 2],       // 픽셀 크기
    pub offset: [f32; 2],     // 베이스라인으로부터의 오프셋
    pub advance: f32,         // 다음 글리프까지의 거리
}

#[derive(Debug, Clone)]
pub struct TextBounds {
    pub width: f32,
    pub height: f32,
    pub num_lines: usize,
}

pub struct TextRendererBuilder {
    atlas_size: (u32, u32),
    font_data: &'static [u8],
}

impl Default for TextRendererBuilder {
    fn default() -> Self {
        Self {
            atlas_size: (2048, 2048),
            font_data: DEFAULT_FONT_DATA,
        }
    }
}

impl TextRendererBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_atlas_size(
        mut self,
        width: u32,
        height: u32,
    ) -> Self {
        self.atlas_size = (width, height);
        self
    }

    pub fn with_font(
        mut self,
        font_data: &'static [u8],
    ) -> Self {
        self.font_data = font_data;
        self
    }

    pub fn build(self) -> Result<TextRenderer, String> {
        TextRenderer::new_with_font(self.atlas_size, self.font_data)
    }
}

pub struct TextRenderer {
    shader_program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    texture: GLuint,
    atlas_size: (u32, u32),
    glyph_cache: HashMap<char, GlyphInfo>,
    font: Font<'static>,
    next_region: AtlasRegion, // 다음 빈 영역 추적
}

impl TextRenderer {
    pub fn builder() -> TextRendererBuilder {
        TextRendererBuilder::new()
    }

    fn new_with_font(
        atlas_size: (u32, u32),
        font_data: &'static [u8],
    ) -> Result<Self, String> {
        Self::validate_atlas_size(atlas_size.0, atlas_size.1)?;

        let font =
            Font::try_from_bytes(font_data).ok_or("Failed to load font")?;

        // 셰이더 프로그램 생성
        let vertex_shader = sprite_shaders::compile_shader(
            gl::VERTEX_SHADER,
            TEXT_VERTEX_SHADER,
        );
        let fragment_shader = sprite_shaders::compile_shader(
            gl::FRAGMENT_SHADER,
            TEXT_FRAGMENT_SHADER,
        );
        let shader_program =
            sprite_shaders::link_program(vertex_shader, fragment_shader);

        // OpenGL 리소스 생성
        let mut texture = 0;
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // 텍스처 생성 및 설정
            gl::GenTextures(1, &mut texture);
            if texture == 0 {
                return Err("Failed to create texture".to_string());
            }

            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                atlas_size.0 as i32,
                atlas_size.1 as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            // VAO 생성
            gl::GenVertexArrays(1, &mut vao);
            if vao == 0 {
                gl::DeleteTextures(1, &texture);
                return Err("Failed to create VAO".to_string());
            }

            // VBO 생성
            gl::GenBuffers(1, &mut vbo);
            if vbo == 0 {
                gl::DeleteTextures(1, &texture);
                gl::DeleteVertexArrays(1, &vao);
                return Err("Failed to create VBO".to_string());
            }

            // 버퍼 설정
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (24 * std::mem::size_of::<f32>()) as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // 버텍스 속성 설정
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

            // 텍스처 유닛 설정
            gl::UseProgram(shader_program);
            let sampler_loc = gl::GetUniformLocation(
                shader_program,
                b"text\0".as_ptr() as *const _,
            );
            gl::Uniform1i(sampler_loc, 0);

            // 바인딩 해제
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::UseProgram(0);
        }

        Ok(Self {
            shader_program,
            vao,
            vbo,
            texture,
            atlas_size,
            glyph_cache: HashMap::new(),
            font,
            next_region: AtlasRegion {
                x: 0,
                y: 0,
                width: atlas_size.0,
                height: 32,
            },
        })
    }

    fn get_or_cache_glyph(
        &mut self,
        c: char,
        font_size: f32,
    ) -> Option<GlyphInfo> {
        if let Some(info) = self.glyph_cache.get(&c) {
            Some(info.clone())
        } else {
            let scale = Scale::uniform(font_size);
            let info = self.cache_glyph(c, scale)?;
            self.glyph_cache.insert(c, info.clone());
            Some(info)
        }
    }

    fn draw_glyph(
        &self,
        glyph: &GlyphInfo,
        x: f32,
        y: f32,
        font_size: f32,
    ) {
        self.render_glyph(glyph, x, y, font_size / glyph.size[1])
    }

    pub fn calculate_bounds(
        &mut self,
        text: &str,
        style: &TextStyle,
    ) -> TextBounds {
        let scale = Scale::uniform(style.font_size);
        let v_metrics = self.font.v_metrics(scale);

        let mut max_width: f32 = 0.0;
        let mut current_width: f32 = 0.0;
        let mut num_lines = 1;
        let mut current_line_chars = 0;

        for c in text.nfc() {
            if c == '\n' {
                max_width = max_width.max(current_width);
                current_width = 0.0;
                current_line_chars = 0;
                num_lines += 1;
                continue;
            }

            if let Some(info) = self.glyph_cache.get(&c) {
                current_width += info.advance + style.letter_spacing;
                current_line_chars += 1;
            } else if let Some(info) = self.cache_glyph(c, scale) {
                current_width += info.advance + style.letter_spacing;
                current_line_chars += 1;
                self.glyph_cache.insert(c, info.clone());
            }

            // 최대 너비 제한이 있는 경우 줄바꿈 처리
            if let Some(max_width_limit) = style.max_width {
                if current_width > max_width_limit && current_line_chars > 0 {
                    max_width = max_width.max(current_width);
                    current_width = 0.0;
                    current_line_chars = 0;
                    num_lines += 1;
                }
            }
        }

        // 마지막 줄 처리
        max_width = max_width.max(current_width);
        if current_line_chars > 0 && style.letter_spacing > 0.0 {
            max_width -= style.letter_spacing; // 마지막 문자 뒤의 자간 제거
        }

        // 패딩 적용
        max_width += style.padding.left + style.padding.right;
        let height = (num_lines as f32) * (style.font_size * style.line_height)
            + style.padding.top
            + style.padding.bottom;

        TextBounds {
            width: max_width,
            height,
            num_lines,
        }
    }

    // 텍스트 배경 그리기
    fn draw_text_background(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        style: &TextStyle,
        bg_color: [f32; 4],
    ) {
        unsafe {
            // 배경을 위한 임시 셰이더 프로그램 사용
            gl::UseProgram(self.shader_program);

            // 배경색 설정
            let color_loc = gl::GetUniformLocation(
                self.shader_program,
                b"textColor\0".as_ptr() as *const _,
            );
            gl::Uniform4fv(color_loc, 1, bg_color.as_ptr());

            // 텍스트 영역 계산
            let bounds = self.calculate_bounds(text, style);

            let (x_pos, width) = match style.alignment {
                TextAlignment::Left => (x, bounds.width),
                TextAlignment::Center => (x - bounds.width / 2.0, bounds.width),
                TextAlignment::Right => (x - bounds.width, bounds.width),
            };

            let y_pos = match style.vertical_alignment {
                VerticalAlignment::Top => y,
                VerticalAlignment::Middle => y - bounds.height / 2.0,
                VerticalAlignment::Bottom => y - bounds.height,
            };

            // 배경 사각형을 위한 버텍스 데이터
            let vertices: [f32; 24] = [
                // 위치          // 텍스처 좌표 (사용되지 않음)
                x_pos,
                y_pos + bounds.height,
                0.0,
                0.0,
                x_pos + width,
                y_pos,
                0.0,
                0.0,
                x_pos,
                y_pos,
                0.0,
                0.0,
                x_pos,
                y_pos + bounds.height,
                0.0,
                0.0,
                x_pos + width,
                y_pos + bounds.height,
                0.0,
                0.0,
                x_pos + width,
                y_pos,
                0.0,
                0.0,
            ];

            // 배경 사각형 그리기
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
            );

            // 텍스처 바인딩 해제 (배경은 단색)
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // 텍스처 다시 바인딩 (텍스트 렌더링을 위해)
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    fn validate_atlas_size(
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        if !width.is_power_of_two() || !height.is_power_of_two() {
            return Err("Atlas dimensions must be power of 2".to_string());
        }
        if width < 256 || height < 256 {
            return Err("Atlas size too small".to_string());
        }
        if width > 8192 || height > 8192 {
            return Err("Atlas size too large".to_string());
        }
        Ok(())
    }

    fn find_space_for_glyph(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<(u32, u32)> {
        const PADDING: u32 = 1; // 글리프 간 패딩
        let width = width + PADDING;
        let height = height + PADDING;

        if width > self.atlas_size.0 || height > self.atlas_size.1 {
            return None;
        }

        // 현재 줄에 맞는지 확인
        if self.next_region.x + width <= self.atlas_size.0 {
            let pos = (self.next_region.x, self.next_region.y);
            self.next_region.x += width;
            self.next_region.height = self.next_region.height.max(height);
            return Some(pos);
        }

        // 새 줄로 이동
        self.next_region.y += self.next_region.height;
        self.next_region.x = 0;
        self.next_region.height = height;

        // 새 줄이 아틀라스를 벗어나는지 확인
        if self.next_region.y + height <= self.atlas_size.1 {
            let pos = (self.next_region.x, self.next_region.y);
            self.next_region.x = width;
            Some(pos)
        } else {
            None // 아틀라스가 가득 참
        }
    }

    // 한 줄의 너비 계산
    fn calculate_line_width(
        &mut self,
        line: &str,
        style: &TextStyle,
    ) -> f32 {
        let scale = Scale::uniform(style.font_size);
        let mut width = 0.0;

        for c in line.nfc() {
            if let Some(info) = self.glyph_cache.get(&c) {
                width += info.advance + style.letter_spacing;
            } else if let Some(info) = self.cache_glyph(c, scale) {
                width += info.advance + style.letter_spacing;
                self.glyph_cache.insert(c, info);
            }
        }

        if !line.is_empty() && style.letter_spacing > 0.0 {
            width -= style.letter_spacing; // 마지막 문자 뒤의 자간 제거
        }

        width
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        style: &TextStyle,
        projection: &glam::Mat4,
    ) {
        // 배경이 있는 경우 먼저 그리기
        if let Some(bg_color) = style.background_color {
            self.draw_text_background(text, x, y, style, bg_color);
        }

        let bounds = self.calculate_bounds(text, style);
        let mut base_y = match style.vertical_alignment {
            VerticalAlignment::Top => y + style.padding.top,
            VerticalAlignment::Middle => {
                y + (style.padding.top + style.padding.bottom) / 2.0
                    - bounds.height / 2.0
            }
            VerticalAlignment::Bottom => {
                y + style.padding.bottom + bounds.height
            }
        };

        unsafe {
            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

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

            let color_loc = gl::GetUniformLocation(
                self.shader_program,
                b"textColor\0".as_ptr() as *const _,
            );
            gl::Uniform4fv(color_loc, 1, style.color.as_ptr());
        }

        let mut current_line = String::new();
        let mut current_width = 0.0;
        let base_x = x + style.padding.left;

        for c in text.nfc() {
            if c == '\n' {
                self.draw_line(&current_line, base_x, base_y, style);
                current_line.clear();
                current_width = 0.0;
                base_y += style.font_size * style.line_height;
                continue;
            }

            if let Some(info) = self.get_or_cache_glyph(c, style.font_size) {
                let next_width =
                    current_width + info.advance + style.letter_spacing;
                if let Some(max_width) = style.max_width {
                    if next_width > max_width && !current_line.is_empty() {
                        self.draw_line(&current_line, base_x, base_y, style);
                        current_line.clear();
                        current_width = 0.0;
                        base_y += style.font_size * style.line_height;
                    }
                }
                current_line.push(c);
                current_width = next_width;
            }
        }

        if !current_line.is_empty() {
            self.draw_line(&current_line, base_x, base_y, style);
        }
    }

    fn draw_line(
        &mut self,
        line: &str,
        x: f32,
        y: f32,
        style: &TextStyle,
    ) {
        let line_width = self.calculate_line_width(line, style);
        let x_offset = match style.alignment {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => -line_width / 2.0,
            TextAlignment::Right => -line_width,
        };

        let mut cursor_x = x + x_offset;
        for c in line.nfc() {
            if let Some(glyph_info) = self.glyph_cache.get(&c) {
                self.draw_glyph(glyph_info, cursor_x, y, style.font_size);
                cursor_x += glyph_info.advance + style.letter_spacing;
            }
        }
    }

    // 글리프를 텍스처 아틀라스에 추가
    fn cache_glyph(
        &mut self,
        c: char,
        scale: Scale,
    ) -> Option<GlyphInfo> {
        let glyph = self.font.glyph(c).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let v_metrics = self.font.v_metrics(scale);

        // 글리프를 원점에 배치
        let glyph = glyph.positioned(point(0.0, v_metrics.ascent));

        if let Some(bb) = glyph.pixel_bounding_box() {
            let width = (bb.max.x - bb.min.x) as u32;
            let height = (bb.max.y - bb.min.y) as u32;

            // 빈 글리프 처리 (예: 공백)
            if width == 0 || height == 0 {
                return Some(GlyphInfo {
                    tex_coords: [0.0, 0.0, 0.0, 0.0],
                    size: [0.0, 0.0],
                    offset: [0.0, 0.0],
                    advance: h_metrics.advance_width,
                });
            }

            // 아틀라스에서 공간 찾기
            let (atlas_x, atlas_y) =
                self.find_space_for_glyph(width, height)?;

            // 글리프 이미지 생성
            let mut buffer = vec![0u8; (width * height) as usize];
            glyph.draw(|x, y, v| {
                let idx = y as usize * width as usize + x as usize;
                buffer[idx] = (v * 255.0) as u8;
            });

            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.texture);
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    atlas_x as i32,
                    atlas_y as i32,
                    width as i32,
                    height as i32,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    buffer.as_ptr() as *const _,
                );
            }

            // UV 좌표 계산
            let tex_coords = [
                atlas_x as f32 / self.atlas_size.0 as f32,
                atlas_y as f32 / self.atlas_size.1 as f32,
                (atlas_x + width) as f32 / self.atlas_size.0 as f32,
                (atlas_y + height) as f32 / self.atlas_size.1 as f32,
            ];

            Some(GlyphInfo {
                tex_coords,
                size: [width as f32, height as f32],
                offset: [bb.min.x as f32, bb.min.y as f32],
                advance: h_metrics.advance_width,
            })
        } else {
            // 렌더링 할 수 없는 글리프 처리
            Some(GlyphInfo {
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                size: [0.0, 0.0],
                offset: [0.0, 0.0],
                advance: h_metrics.advance_width,
            })
        }
    }

    pub fn render_text(
        &mut self,
        text: &str,
        mut x: f32,
        y: f32,
        font_size: f32,
        color: [f32; 4],
        projection: &glam::Mat4,
    ) {
        let scale = Scale::uniform(font_size);

        unsafe {
            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            // 유니폼 설정
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

            let color_loc = gl::GetUniformLocation(
                self.shader_program,
                b"textColor\0".as_ptr() as *const _,
            );
            gl::Uniform4fv(color_loc, 1, color.as_ptr());
        }

        // 유니코드 정규화 적용
        for c in text.nfc() {
            let glyph_info = if let Some(info) = self.glyph_cache.get(&c) {
                info.clone()
            } else if let Some(info) = self.cache_glyph(c, scale) {
                self.glyph_cache.insert(c, info.clone());
                info
            } else {
                continue; // 캐시 실패
            };

            // 글리프 렌더링
            self.render_glyph(&glyph_info, x, y, 1.0);
            x += glyph_info.advance;
        }
    }

    pub fn render_text_multiline(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        style: &TextStyle,
        projection: &glam::Mat4,
    ) {
        let scale = Scale::uniform(style.font_size);
        let v_metrics = self.font.v_metrics(scale);
        let line_height = style.line_height * style.font_size;

        // 줄 바꿈 처리를 위한 임시 버퍼
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut lines = Vec::new();

        for c in text.nfc() {
            if c == '\n' {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                continue;
            }

            let glyph_info = if let Some(info) = self.glyph_cache.get(&c) {
                info.clone()
            } else if let Some(info) = self.cache_glyph(c, scale) {
                self.glyph_cache.insert(c, info.clone());
                info
            } else {
                continue;
            };

            let next_width =
                current_width + glyph_info.advance + style.letter_spacing;

            if let Some(max_width) = style.max_width {
                if next_width > max_width && !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                    current_width = 0.0;
                }
            }

            current_line.push(c);
            current_width = next_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // 각 줄 렌더링
        let mut current_y = y;
        for line in lines {
            let line_width = self.calculate_text_width(&line, style);
            let line_x = match style.alignment {
                TextAlignment::Left => x,
                TextAlignment::Center => x - line_width / 2.0,
                TextAlignment::Right => x - line_width,
            };

            self.render_text(
                &line,
                line_x,
                current_y,
                style.font_size,
                style.color,
                projection,
            );

            current_y += line_height;
        }
    }

    // 텍스트 너비 계산
    pub fn calculate_text_width(
        &mut self,
        text: &str,
        style: &TextStyle,
    ) -> f32 {
        let scale = Scale::uniform(style.font_size);
        let mut width = 0.0;

        for c in text.nfc() {
            if let Some(info) = self.glyph_cache.get(&c) {
                width += info.advance + style.letter_spacing;
            } else if let Some(info) = self.cache_glyph(c, scale) {
                width += info.advance + style.letter_spacing;
                self.glyph_cache.insert(c, info);
            }
        }

        if !text.is_empty() {
            width -= style.letter_spacing; // 마지막 문자 뒤의 자간 제거
        }

        width
    }

    // 텍스트 경계 상자 계산
    pub fn calculate_text_bounds(
        &mut self,
        text: &str,
        style: &TextStyle,
    ) -> (f32, f32) {
        let scale = Scale::uniform(style.font_size);
        let v_metrics = self.font.v_metrics(scale);

        let mut max_width: f32 = 0.0;
        let mut current_width: f32 = 0.0;
        let mut num_lines: usize = 1;

        for c in text.nfc() {
            if c == '\n' {
                max_width = max_width.max(current_width);
                current_width = 0.0;
                num_lines += 1;
                continue;
            }

            if let Some(info) = self.glyph_cache.get(&c) {
                current_width += info.advance + style.letter_spacing;
            } else if let Some(info) = self.cache_glyph(c, scale) {
                current_width += info.advance + style.letter_spacing;
                self.glyph_cache.insert(c, info);
            }
        }

        max_width = max_width.max(current_width);
        let total_height =
            num_lines as f32 * style.line_height * style.font_size;

        (max_width, total_height)
    }

    fn render_glyph(
        &self,
        glyph: &GlyphInfo,
        x: f32,
        y: f32,
        scale: f32,
    ) {
        if glyph.size[0] == 0.0 || glyph.size[1] == 0.0 {
            return; // 보이지 않는 글리프는 건너뜀
        }

        let x1 = x + glyph.offset[0] * scale;
        let y1 = y + glyph.offset[1] * scale;
        let x2 = x1 + glyph.size[0] * scale;
        let y2 = y1 + glyph.size[1] * scale;

        let vertices: [f32; 24] = [
            x1,
            y2,
            glyph.tex_coords[0],
            glyph.tex_coords[3],
            x2,
            y1,
            glyph.tex_coords[2],
            glyph.tex_coords[1],
            x1,
            y1,
            glyph.tex_coords[0],
            glyph.tex_coords[1],
            x1,
            y2,
            glyph.tex_coords[0],
            glyph.tex_coords[3],
            x2,
            y2,
            glyph.tex_coords[2],
            glyph.tex_coords[3],
            x2,
            y1,
            glyph.tex_coords[2],
            glyph.tex_coords[1],
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
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            // 먼저 바인딩된 상태 해제
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);

            // 리소스 정리
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.shader_program);
        }
    }
}

const TEXT_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;
    
    out vec2 TexCoords;
    
    uniform mat4 projection;
    
    void main()
    {
        gl_Position = projection * vec4(aPos, 0.0, 1.0);
        TexCoords = aTexCoords;
    }
"#;

const TEXT_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    in vec2 TexCoords;
    out vec4 FragColor;
    
    uniform sampler2D text;
    uniform vec4 textColor;
    
    void main()
    {
        vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text, TexCoords).r);
        FragColor = textColor * sampled;
    }
"#;
