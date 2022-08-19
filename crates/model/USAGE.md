
Translate a directory's model files into a custom format
(for consumption by this engine). Currently only supports
COLLADA format.

### Basic Usage

Run from within a build script. It will output to the
package's `OUT_DIR` location, in a subdirectory called
`models`.

From within a build script:

```rust
use model::parse_directory;

let models_dir = {
    let mut dir = std::env::current_dir().unwrap();
    dir.pop();
    dir.push("resources");
    dir.push("models");
    dir
};
let models = parse_directory(&models_dir);
```

Then consume in your app. The generated files will have a
".mdl" extension, and the file names take from the names
of the geometries in the source file.

```rust
use model::{Model, StaticVertex, StoresAsFile};

const MODEL_BYTES: &[u8] = include_bytes!(
    concat!(env!("OUT_DIR"), "/models/SceneTerrain.mdl"));
let model: Model<StaticVertex> = unsafe {
    Model::new_from_bytes(MODEL_BYTES).unwrap()
};
```

### Advanced Configuration

The parser processes TOML configuration files for each
model file, if they are present. The configuration file
must have the same file name, but with a ".toml"
extension.

Currently, the only supported usage is to merge several
models from within the file into a new one with a new
name.

```toml
# Given a file Cubes.dae exists, contents of Cubes.toml:

[[merges]]
name = "Cubes"
geometries = ["Cube1", "Cube2", "Cube3"]
```
