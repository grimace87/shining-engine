
use serde::Deserialize;
use crate::types::StaticVertex;

/// Recognised values for the semantic attribute found in Collada XML
const SEMANTIC_VERTEX: &str = "VERTEX";
const SEMANTIC_POSITION: &str = "POSITION";
const SEMANTIC_NORMAL: &str = "NORMAL";
const SEMANTIC_TEX_COORD: &str = "TEXCOORD";

/// GeometryLibrary struct
/// Representation for a library_geometries XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GeometryLibrary {
    #[serde(rename = "geometry", default)]
    pub items: Vec<Geometry>
}

/// Geometry struct
/// Representation for items under a geometry XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Geometry {
    pub id: String,
    pub name: String,
    pub mesh: Mesh
}

/// Mesh struct
/// Representation for a mesh XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Mesh {
    vertices: Vertices,
    triangles: Triangles,

    #[serde(rename = "source", default)]
    sources: Vec<Source>
}

impl Mesh {

    /// Translate data within a mesh tag into a vector of StaticVertex instances
    pub fn get_vertex_data(&self) -> Vec<StaticVertex> {
        let interleaved_indices = self.decode_triangle_indices();
        let position_data = self.decode_position_data();
        let normal_data = self.decode_normal_data();
        let tex_coord_data = self.decode_tex_coord_data();

        let mut index = 0;
        let mut vertices = vec![];
        loop {
            if index >= interleaved_indices.len() {
                break;
            }
            let position_index = interleaved_indices[index];
            let normal_index = interleaved_indices[index + 1];
            let tex_coord_index = interleaved_indices[index + 2];
            vertices.push(StaticVertex::from_components(
                (
                    position_data[position_index * 3],
                    position_data[position_index * 3 + 1],
                    position_data[position_index * 3 + 2]),
                (
                    normal_data[normal_index * 3],
                    normal_data[normal_index * 3 + 1],
                    normal_data[normal_index * 3 + 2]),
                (
                    tex_coord_data[tex_coord_index * 2],
                    tex_coord_data[tex_coord_index * 2 + 1])
            ));
            index += 3;
        }
        vertices
    }

    /// Retrieve the index data from this mesh as a vector of unsigned integers
    fn decode_triangle_indices(&self) -> Vec<usize> {
        let value_string = &self.triangles.polygons.values;
        let numbers: Result<Vec<usize>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse integer array for triangles")
    }

    /// Retrieve the position data from this mesh as a vector of single-precision floating-point
    /// numbers
    fn decode_position_data(&self) -> Vec<f32> {
        let vertex_input = self.triangles.inputs.iter()
            .find(|input| input.semantic.as_str() == SEMANTIC_VERTEX)
            .expect("No VERTEX input found for triangles");
        if self.vertices.id.as_str() != &vertex_input.source[1..vertex_input.source.len()] {
            panic!("Mesh vertices id does not match triangles vertex input source");
        }
        if self.vertices.input.semantic.as_str() != SEMANTIC_POSITION {
            panic!("Mesh vertices input does not have POSITION semantic");
        }
        let position_source_id = &self.vertices.input.source;
        let position_source_id = &position_source_id[1..position_source_id.len()];
        let position_source = self.sources.iter()
            .find(|source| source.id.as_str() == position_source_id)
            .expect("Did not find position source for mesh");
        if position_source.technique_common.accessor.params.len() != 3 {
            panic!("Position source does not have 3 parameters");
        }
        let value_string = &position_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for position data")
    }

    /// Retrieve the normal data from this mesh as a vector of single-precision floating-point
    /// numbers
    fn decode_normal_data(&self) -> Vec<f32> {
        let normal_input = self.triangles.inputs.iter()
            .find(|input| input.semantic.as_str() == SEMANTIC_NORMAL)
            .expect("No NORMAL input found for triangles");
        let normal_source_id = &normal_input.source;
        let normal_source_id = &normal_source_id[1..normal_source_id.len()];
        let normal_source = self.sources.iter()
            .find(|source| source.id.as_str() == normal_source_id)
            .expect("Did not find normal source for mesh");
        if normal_source.technique_common.accessor.params.len() != 3 {
            panic!("Normal source does not have 3 parameters");
        }
        let value_string = &normal_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for normal data")
    }

    /// Retrieve the texture coordinate data from this mesh as a vector of single-precision
    /// floating-point numbers
    fn decode_tex_coord_data(&self) -> Vec<f32> {
        let tex_coord_input = self.triangles.inputs.iter()
            .find(|input| input.semantic.as_str() == SEMANTIC_TEX_COORD)
            .expect("No TEXCOORD input found for triangles");
        let tex_coord_source_id = &tex_coord_input.source;
        let tex_coord_source_id = &tex_coord_source_id[1..tex_coord_source_id.len()];
        let tex_coord_source = self.sources.iter()
            .find(|source| source.id.as_str() == tex_coord_source_id)
            .expect("Did not find tex coord source for mesh");
        if tex_coord_source.technique_common.accessor.params.len() != 2 {
            panic!("Tex coord source does not have 2 parameters");
        }
        let value_string = &tex_coord_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for tex coord data")
    }
}

/// Vertices struct
/// Representation for a vertices XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Vertices {
    id: String,
    input: Input
}

/// Input struct
/// Representation for an input XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Input {
    semantic: String,
    source: String,

    #[serde(default)]
    offset: i32
}

/// Triangles struct
/// Representation for a triangles XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Triangles {
    count: i32,

    #[serde(rename = "input", default)]
    inputs: Vec<Input>,

    #[serde(rename = "p", default)]
    polygons: IntegerArray
}

/// IntegerArray struct
/// Representation for a polygons XML tag
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct IntegerArray {

    #[serde(rename = "$value", default)]
    values: String
}

/// Source struct
/// Representation for items under a source XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Source {
    id: String,
    technique_common: TechniqueCommon,

    #[serde(rename = "float_array", default)]
    float_data: FloatArray
}

/// FloatArray struct
/// Representation for a float_data XML tag
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FloatArray {
    id: String,
    count: i32,

    #[serde(rename = "$value", default)]
    values: String
}

/// TechniqueCommon struct
/// Representation for a technique_common XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TechniqueCommon {
    accessor: Accessor
}

/// Accessor struct
/// Representation for a accessor XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Accessor {
    source: String,
    count: i32,
    stride: i32,

    #[serde(rename = "param", default)]
    params: Vec<Param>
}

/// Param struct
/// Representation for items under a param XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Param {
    name: String,

    #[serde(rename = "type", default)]
    param_type: String
}

/// VisualScenesLibrary struct
/// Representation for a library_visual_scenes XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VisualScenesLibrary {
    pub visual_scene: VisualScene
}

/// VisualScene struct
/// Representation for a visual_scene XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VisualScene {
    id: String,
    name: String,

    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>
}

/// Node struct
/// Representation for items under a nodes XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Node {
    pub id: String,
    pub name: String,

    #[serde(rename = "type")]
    pub node_type: String,

    pub matrix: Matrix,

    #[serde(default)]
    pub instance_geometry: Option<Instance>,

    #[serde(default)]
    pub instance_camera: Option<Instance>,

    #[serde(default)]
    pub instance_light: Option<Instance>
}

/// Matrix struct
/// Representation for a matrix XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Matrix {
    sid: String,

    #[serde(rename = "$value", default)]
    values: String
}

impl Matrix {
    pub fn decode_element_data(&self) -> Vec<f32> {
        let numbers: Result<Vec<f32>, _> = self.values.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for matrix")
    }
}

/// Instance struct
/// Representation for an instance_geometry, instance_camera, or instance_light XML tag
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Instance {
    pub url: String
}
