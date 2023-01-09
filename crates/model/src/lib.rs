mod types;
mod factory;
mod collada;
mod config;

pub use factory::StoresAsFile;
pub use types::{Model, StaticVertex};

pub use collada::COLLADA;
pub use config::Config;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

/// Parse the model files in a source directory. Call from a build script.
/// Currently only supports COLLADA files with a ".dae" extension.
/// Files will be written to the build script's OUT_DIR location with a ".mdl" extension.
///
/// See USAGE.md for more information.
pub fn parse_directory(source_dir: &Path) -> Result<(), String> {

    if !source_dir.is_dir() {
        return Err(
            format!("Supplied source path must be a directory: {:?}", source_dir));
    }

    let binary_models_dir = {

        // Temporary directory in this package's directory
        #[cfg(test)]
        let mut dir = {
            let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            buf.pop();
            buf.pop();
            buf.push("target");
            buf.push("tmp");
            buf
        };

        // Build script output
        #[cfg(not(test))]
        let mut dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

        dir.push("models");
        if !dir.is_dir() {
            std::fs::create_dir(&dir).unwrap();
        }
        dir
    };

    convert_collada_files_in_directory(source_dir, &binary_models_dir);

    Ok(())
}

/// Traverse contents of directory and process COLLADA files. Also processes any matching
/// config files found for them.
fn convert_collada_files_in_directory(collada_models_dir: &Path, binary_models_dir: &Path) {
    for entry in std::fs::read_dir(collada_models_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = match path.extension() {
            Some(e) => e,
            None => continue
        };
        match extension.to_str() {
            Some("dae") => {
                let mut config_path = path.clone();
                config_path.set_extension("toml");
                let config = match config_path.exists() {
                    true => Config::from_toml_file(&config_path),
                    false => Config::default()
                };
                convert_collada_file(&path, config, binary_models_dir);
            },
            _ => continue
        };
    }
}

/// Interpret a COLLADA file and process it according to a given Config, writing output file(s)
fn convert_collada_file(source_file: &Path, config: Config, binary_models_dir: &Path) {
    let mut collada_file = File::open(source_file)
        .expect("Failed to open a file");
    let file_metadata = std::fs::metadata(source_file)
        .expect("Failed to read file metadata");
    let mut file_bytes = vec![0; file_metadata.len() as usize];
    collada_file.read_exact(&mut file_bytes)
        .expect("Buffer overflow reading from file");
    let collada = COLLADA::new(file_bytes.as_slice());
    let models = collada.extract_models(config);
    for model in models.iter() {
        let mut file_path = PathBuf::from(binary_models_dir);
        file_path.push(model.name.as_str());
        file_path.set_extension("mdl");
        unsafe {
            model.write_to_binary_file(&file_path).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn models_are_processed() {
        let models_dir = {
            let mut dir = std::env::current_dir().unwrap();
            dir.pop();
            dir.pop();
            dir.push("resources");
            dir.push("test");
            dir.push("models");
            dir
        };
        crate::parse_directory(&models_dir).unwrap();
    }
}
