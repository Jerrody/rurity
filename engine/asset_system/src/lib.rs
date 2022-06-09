#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unstable_features)]

use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum AssetType {
    Mesh = 0,
    Texture = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetFile {
    asset_type: AssetType,
    version: String,
    metadata: String,
    raw_data: Vec<u8>,
}

impl AssetFile {
    pub fn save_asset_file(&self, path: &impl AsRef<std::path::Path>) -> Result<(), String> {
        use std::fs::File;

        let mut asset_file =
            File::options()
                .write(true)
                .open(path)
                .or_else(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => File::create(path).or_else(|e| {
                        let e = format!("Error: Failed to create an asset file {e:?}");
                        Err(e)
                    }),
                    e => {
                        let e = format!("Error: Failed to save an asset file: {e:?}");
                        Err(e)
                    }
                })?;

        Ok(ron::ser::to_writer(&mut asset_file, &self).map_err(|err| err.to_string())?)
    }

    pub fn load_asset_file(path: &str) -> Result<AssetFile, String> {
        use std::fs::File;

        let mut asset_file = File::options()
            .write(false)
            .read(true)
            .open(path)
            .or_else(|e| {
                let e = format!("Error: Failed to save an asset file: {:?}", e.kind());
                Err(e)
            })?;

        let mut raw_data = String::new();
        asset_file
            .read_to_string(&mut raw_data)
            .map_err(|e| e.to_string())?;

        let asset_file = ron::de::from_bytes(&raw_data.as_bytes()).map_err(|e| e.to_string())?;

        Ok(asset_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_PATH: &str = r"C:\rust\rurity\engine\asset_system\src\asset_file.bin";
    const CONTENT_OF_ASSET_FILE: &str =
        "(asset_type:Mesh,version:1,metadata:\"HI\",raw_data:[1,2,3])";

    #[test]
    fn save_asset_file() {
        let asset_file = AssetFile {
            asset_type: AssetType::Mesh,
            version: "0.1.0".to_string(),
            metadata: "HI".to_string(),
            raw_data: vec![1, 2, 3],
        };

        asset_file.save_asset_file(&FILE_PATH).unwrap();
    }

    #[test]
    fn laod_asset_file() {
        let asset_file = AssetFile::load_asset_file(&FILE_PATH).unwrap();

        assert_eq!(
            &ron::to_string(&asset_file).unwrap(),
            CONTENT_OF_ASSET_FILE,
            "Content of the loaded asset file doesn't correspond with expected content."
        );
    }
}
