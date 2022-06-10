#![deny(unsafe_code)]
#![deny(unstable_features)]

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

mod texture;

const CURRENT_ASSET_SYSTEM_VERSION: &str = "0.1.0";

// TODO: Rename in the future, name of the trait looks not so good.
pub trait Packaging {
    fn pack(
        &self,
        name: &str,
        path: &str,
        raw_data: Vec<u8>,
        compression_mode: CompressionMode,
    ) -> Result<AssetFile, String>;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CompressionMode {
    Default = 4,
    Fast = 2,
    HighCompression = 7,
    VeryHighCompression = 9,
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize)]
pub enum AssetType {
    Mesh = 0,
    Texture = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetFile {
    name: String,
    path: String,
    asset_type: AssetType,
    compression_mode: CompressionMode,
    version: String,
    metadata: String,
    raw_data: Vec<u8>,
}

impl AssetFile {
    pub fn new<T: Packaging>(
        asset: T,
        name: &str,
        path: &str,
        raw_data: Vec<u8>,
        compression_mode: CompressionMode,
    ) -> Result<AssetFile, String> {
        asset.pack(name, path, raw_data, compression_mode)
    }

    fn save_content(&self, asset_file: File) -> Result<(), String> {
        let serialized = ron::to_string(self).map_err(|e| e.to_string())?;
        let mut serialized_data_encoder = lz4::EncoderBuilder::new()
            .level(self.compression_mode as u32)
            .build(asset_file)
            .map_err(|e| e.to_string())?;
        std::io::copy(&mut serialized.as_bytes(), &mut serialized_data_encoder)
            .map_err(|e| e.to_string())?;
        let (_output, result) = serialized_data_encoder.finish();

        result.map_err(|e| e.to_string())
    }

    pub fn save_asset_file(&self) -> Result<(), String> {
        match File::options().write(true).truncate(false).open(&self.path) {
            Ok(asset_file) => self.save_content(asset_file),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let asset_file = File::create(&self.path)
                        .map_err(|e| format!("Error: Failed to create an asset file {e:?}"))?;

                    self.save_content(asset_file)
                }
                e => {
                    let e = format!("Error: Failed to save an asset file: {e:?}");
                    Err(e)
                }
            },
        }
    }

    pub fn load_asset_file<T: AsRef<std::path::Path> + ?Sized>(
        path: &T,
    ) -> Result<AssetFile, String> {
        let asset_file = File::options()
            .write(false)
            .read(true)
            .open(path)
            .map_err(|e| format!("Error: Failed to load an asset file: {:?}", e.kind()))?;

        let mut decompressed_raw_data = vec![];
        lz4::Decoder::new(asset_file)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut decompressed_raw_data)
            .map_err(|e| e.to_string())?;

        let decompressed_data =
            std::str::from_utf8(&decompressed_raw_data).map_err(|e| e.to_string())?;

        let asset_file =
            ron::de::from_bytes(decompressed_data.as_bytes()).map_err(|e| e.to_string())?;

        Ok(asset_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREPARED_ASSET_FILE_PATH: &str = "src/test_asset_files/asset_file.bin";
    const CONTENT_OF_ASSET_FILE: &str =
        "(name:\"asset_file\",path:\"src/test_asset_files/asset_file.bin\",asset_type:Mesh,compression_mode:VeryHighCompression,version:\"0.1.0\",metadata:\"HI\",raw_data:[1,2,3])";

    #[test]
    #[cfg_attr(miri, ignore)]
    fn save_asset_file() {
        let asset_file = AssetFile {
            name: "asset_file".to_string(),
            path: PREPARED_ASSET_FILE_PATH.to_string(),
            asset_type: AssetType::Mesh,
            compression_mode: CompressionMode::VeryHighCompression,
            version: CURRENT_ASSET_SYSTEM_VERSION.to_string(),
            metadata: "HI".to_string(),
            raw_data: vec![1, 2, 3],
        };

        asset_file.save_asset_file().unwrap();
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn laod_asset_file() {
        let asset_file = AssetFile::load_asset_file(PREPARED_ASSET_FILE_PATH).unwrap();

        assert_eq!(
            &ron::to_string(&asset_file).unwrap(),
            CONTENT_OF_ASSET_FILE,
            "Content of the loaded asset file doesn't correspond with expected content.",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn create_new_asset_and_read_it() {
        const NEW_ASSET_FILE_PATH: &str = "src/test_asset_files/new_asset_file.bin";
        const NEW_ASSET_NAME: &str = "new_asset_file";
        const CONTENT_NEW_ASSET_FILE: &str =
        "(name:\"new_asset_file\",path:\"src/test_asset_files/new_asset_file.bin\",asset_type:Mesh,compression_mode:VeryHighCompression,version:\"0.1.0\",metadata:\"HI\",raw_data:[1,2,3])";

        let asset_file = AssetFile {
            name: NEW_ASSET_NAME.to_string(),
            path: NEW_ASSET_FILE_PATH.to_string(),
            asset_type: AssetType::Mesh,
            compression_mode: CompressionMode::VeryHighCompression,
            version: CURRENT_ASSET_SYSTEM_VERSION.to_string(),
            metadata: "HI".to_string(),
            raw_data: vec![1, 2, 3],
        };

        asset_file.save_asset_file().unwrap();
        let asset_file = AssetFile::load_asset_file(NEW_ASSET_FILE_PATH).unwrap();

        assert_eq!(
            &ron::to_string(&asset_file).unwrap(),
            CONTENT_NEW_ASSET_FILE,
            "Content of the loaded asset file doesn't correspond with expected content."
        );

        std::fs::remove_file(NEW_ASSET_FILE_PATH).unwrap();
    }
}
