use gl::types::*;
use image::GenericImageView;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Texture {
    id: GLuint,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn new(path: &Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let (width, height) = img.dimensions();
        let data = img.into_rgba8().into_raw();

        Texture::new_from_data(&data, width, height)
    }

    pub fn new_from_data(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Texture, String> {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
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
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );

            // Debug: Check for GL errors
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("GL Error after texture creation: {}", error);
                Err(format!("GL Error: {}", error))?;
            }

            println!("generated texture id: {}", id);
        }

        Ok(Self { id, width, height })
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }

    #[allow(unused)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[allow(unused)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
