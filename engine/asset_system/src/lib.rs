#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unstable_features)]

use std::io::{Read, Write};

#[repr(u8)]
pub enum AssetType {
    Mesh = 0,
    Texture = 1,
}

pub struct AssetFile {
    asset_type: AssetType,
    version: u32,
    metadata: String,
    raw_data: Vec<u8>,
}

impl AssetFile {
    fn save_asset_file(&self, path: &impl AsRef<std::path::Path>) -> Result<(), String> {
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

        // TODO: Make it more pretty.
        if let Err(e) = asset_file.write(&self.version.to_le_bytes()) {
            return Err(e.to_string());
        }
        if let Err(e) = asset_file.write(&self.metadata.as_bytes()) {
            return Err(e.to_string());
        }
        if let Err(e) = asset_file.write(&self.raw_data) {
            return Err(e.to_string());
        }

        Ok(())
    }

    fn load_asset_file(&mut self, path: &str) -> Result<(), String> {
        use bytes::Buf;
        use std::fs::File;
        use std::mem::{size_of, size_of_val};

        let mut asset_file = File::options()
            .write(false)
            .read(true)
            .open(path)
            .or_else(|e| {
                let e = format!("Error: Failed to save an asset file: {:?}", e.kind());
                Err(e)
            })?;
        let mut data = vec![];
        if let Err(e) = asset_file.read_to_end(&mut data) {
            return Err(e.to_string());
        }

        let mut bytes_representation = bytes::Bytes::from(data);
        let file_type = bytes_representation.copy_to_bytes(size_of_val(&self.version));

        Ok(())
    }
}
