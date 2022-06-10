use crate::{AssetFile, AssetType};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize)]
pub enum TextureFormat {
    RGBA8 = 43,
    RGB8 = 29,
    Unknown = 0,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureAsset {
    pub texture_format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl TextureAsset {
    #[allow(unused)]
    pub fn new(texture_format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            texture_format,
            width,
            height,
        }
    }
}

impl super::Packaging for TextureAsset {
    fn pack(
        &self,
        name: &str,
        path: &str,
        raw_data: Vec<u8>,
        compression_mode: super::CompressionMode,
    ) -> Result<AssetFile, String> {
        let serialized = ron::to_string(self).map_err(|e| e.to_string())?;

        Ok(AssetFile {
            name: name.to_string(),
            path: path.to_string(),
            asset_type: AssetType::Texture,
            compression_mode,
            version: super::CURRENT_ASSET_SYSTEM_VERSION.to_string(),
            metadata: serialized,
            raw_data,
        })
    }
}
