use bincode::{deserialize, serialize};
use image::RgbaImage;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use flate2::Compression;
use flate2::{read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CharInfo, FontAtlas, FontError, FontType};

#[derive(Serialize, Deserialize)]
pub struct FontAtlasFile {
    version: u32,        // File format version
    checksum: Vec<u8>,   // SHA-256 checksum of the image data
    image_data: Vec<u8>, // Compressed image data
    chars: HashMap<char, CharInfo>,
    padding: u32,
    width: u32,
    height: u32,
    line_height: f32,
    font_type: FontType,
    font_size: f32,
}

impl FontAtlasFile {
    pub fn save_to_file(
        font_atlas: &FontAtlas,
        path: impl AsRef<Path>,
    ) -> Result<(), FontError> {
        // Compress image data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let raw_image: &[u8] = font_atlas.image.as_raw();
        encoder
            .write_all(raw_image)
            .map_err(|e| FontError::SaveError(e.to_string()))?;
        let compressed_data = encoder
            .finish()
            .map_err(|e| FontError::SaveError(e.to_string()))?;

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(&font_atlas.image.as_raw());
        let checksum = hasher.finalize().to_vec();

        let atlas_file = FontAtlasFile {
            version: FontAtlas::CURRENT_VERSION,
            checksum,
            image_data: compressed_data,
            chars: font_atlas.chars.clone(),
            padding: font_atlas.padding,
            width: font_atlas.width,
            height: font_atlas.height,
            line_height: font_atlas.line_height,
            font_type: font_atlas.font_type.clone(),
            font_size: font_atlas.font_size,
        };

        let encoded = serialize(&atlas_file)
            .map_err(|e| FontError::SaveError(e.to_string()))?;

        let mut file = File::create(path)
            .map_err(|e| FontError::SaveError(e.to_string()))?;

        file.write_all(&encoded)
            .map_err(|e| FontError::SaveError(e.to_string()))?;

        Ok(())
    }

    pub fn load_from_file(
        path: impl AsRef<Path>
    ) -> Result<FontAtlas, FontError> {
        let mut file = File::open(path)
            .map_err(|e| FontError::LoadError(e.to_string()))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| FontError::LoadError(e.to_string()))?;

        Self::load_from_bytes(&buffer)
    }

    pub fn load_from_bytes(data: &[u8]) -> Result<FontAtlas, FontError> {
        let atlas_file: FontAtlasFile = deserialize(data)
            .map_err(|e| FontError::LoadError(e.to_string()))?;

        // Version check
        if atlas_file.version != FontAtlas::CURRENT_VERSION {
            return Err(FontError::LoadError(format!(
                "Unsupported file version: {}",
                atlas_file.version
            )));
        }

        // Decompress image data
        let mut decoder = GzDecoder::new(&atlas_file.image_data[..]);
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .map_err(|e| FontError::LoadError(e.to_string()))?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&decompressed_data);
        let checksum = hasher.finalize().to_vec();
        if checksum != atlas_file.checksum {
            return Err(FontError::LoadError(
                "Checksum verification failed".to_string(),
            ));
        }

        let image = RgbaImage::from_raw(
            atlas_file.width,
            atlas_file.height,
            decompressed_data,
        )
        .ok_or_else(|| {
            FontError::LoadError(
                "Failed to create image from raw data".to_string(),
            )
        })?;

        Ok(FontAtlas {
            image,
            chars: atlas_file.chars,
            padding: atlas_file.padding,
            width: atlas_file.width,
            height: atlas_file.height,
            line_height: atlas_file.line_height,
            font_type: atlas_file.font_type,
            font_size: atlas_file.font_size,
        })
    }
}
