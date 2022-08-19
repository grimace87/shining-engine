
mod elements;

use elements::*;
use crate::types::{
    Model,
    StaticVertex
};
use crate::config::Config;
use serde::Deserialize;
use serde_xml_rs::from_reader;

/// COLLADA struct
/// Target for deserialising root element of Collada XML file
#[derive(Debug, Deserialize)]
pub struct COLLADA {
    library_geometries: GeometryLibrary,
    library_visual_scenes: VisualScenesLibrary
}

impl COLLADA {

    /// Create new instance from file data
    pub fn new(file_data: &[u8]) -> COLLADA {
        from_reader(file_data).unwrap()
    }

    /// Translate the data held by this instance into instances of model::types::Model.
    /// Alter behaviour of this translation according to the supplied configuration.
    pub fn extract_models(&self, config: Config) -> Vec<Model<StaticVertex>> {
        let mut pre_merge_models: Vec<Model<StaticVertex>> = vec![];
        for geometry in self.library_geometries.items.iter() {
            let mesh = &geometry.mesh;
            let mut vertex_data = mesh.get_vertex_data();
            if let Some(scene_matrix) = self.find_transform_for(&geometry.id) {
                Self::transform_vertices(&mut vertex_data, scene_matrix);
            }
            let model_name = String::from(&geometry.name);
            pre_merge_models.push(
                Model::new_from_components(model_name, vertex_data));
        }

        if config.merges.is_empty() {
            return pre_merge_models;
        }

        let mut merged_models: Vec<Model<StaticVertex>> = vec![];
        for merge_config in config.merges.iter() {
            let name = &merge_config.name;
            let mut source_models: Vec<Model<StaticVertex>> = vec![];
            for model_name in merge_config.geometries.iter() {
                let model_index = pre_merge_models.iter()
                    .position(|m| m.name.eq(model_name))
                    .expect(format!("Did not find mesh named {}", model_name).as_str());
                let model = pre_merge_models.remove(model_index);
                source_models.push(model);
            }
            let merged_model = Model::merge(name.as_str(), source_models);
            merged_models.push(merged_model);
        }
        for unmerged_model in pre_merge_models.into_iter() {
            merged_models.push(unmerged_model);
        }
        merged_models
    }

    /// Look up the transformation matrix for a given geometry.
    /// For internal use.
    fn find_transform_for(&self, geometry_id: &String) -> Option<&Matrix> {
        let node = self.library_visual_scenes.visual_scene.nodes.iter().find(|n| {
            match n {
                Node {
                    id: _id,
                    name: _name,
                    node_type: _node_type,
                    matrix: _matrix,
                    instance_camera: _instance_camera,
                    instance_light: _instance_light,
                    instance_geometry: Some(i)
                } => &i.url[1..i.url.len()] == geometry_id.as_str(),
                _ => false
            }
        });
        match node {
            Some(n) => Some(&n.matrix),
            None => None
        }
    }

    /// Transform a set of vertices using a given matrix.
    /// For internal use.
    fn transform_vertices(vertices: &mut Vec<StaticVertex>, matrix: &Matrix) {
        let m = matrix.decode_element_data();
        for vertex in vertices.iter_mut() {

            // Transform positions
            let x = vertex.px;
            let y = vertex.py;
            let z = vertex.pz;
            vertex.px = x * m[0] + y * m[1] + z * m[2] + m[3];
            vertex.py = x * m[4] + y * m[5] + z * m[6] + m[7];
            vertex.pz = x * m[8] + y * m[9] + z * m[10] + m[11];

            // Transform normals
            let x = vertex.nx;
            let y = vertex.ny;
            let z = vertex.nz;
            vertex.nx = x * m[0] + y * m[1] + z * m[2];
            vertex.ny = x * m[4] + y * m[5] + z * m[6];
            vertex.nz = x * m[8] + y * m[9] + z * m[10];
        }
    }
}
