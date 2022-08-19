
use serde::Deserialize;
use std::path::Path;
use std::fs::File;
use std::io::Read;

/// Config struct
/// Configuration for how Collada data is translated to model instances. The only currently-
/// supported feature is merging models together under a new name.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub merges: Vec<Merge>
}

impl Config {

    /// Parse configuration options from a TOML file
    pub fn from_toml_file(path: &Path) -> Config {
        let mut collada_file = File::open(path)
            .expect("Failed to open a config file");
        let file_metadata = std::fs::metadata(path)
            .expect("Failed to read config file metadata");
        let mut file_bytes = vec![0; file_metadata.len() as usize];
        collada_file.read(&mut file_bytes)
            .expect("Buffer overflow reading from config file");
        toml::from_slice(file_bytes.as_slice()).unwrap()
    }
}

/// Merge struct
/// Defines a merge operation, taking geometries from Collada data by name and merging their
/// vertices together under a new model name.
#[derive(Debug, Deserialize)]
pub struct Merge {
    pub name: String,
    pub geometries: Vec<String>
}
