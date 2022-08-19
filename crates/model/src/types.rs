
/// Model struct
/// Represents a model with a name, along with a set of vertices of a generic sized type.
pub struct Model<E> where E : Sized {
    pub name: String,
    pub vertices: Vec<E>
}

impl<E> Model<E> {

    /// Construct a new instance from a set of vertices
    pub fn new_from_components(name: String, vertices: Vec<E>) -> Model<E> {
        Model {
            name,
            vertices
        }
    }

    /// Merge a set of models into a new model under a new name
    pub fn merge(name: &str, source_models: Vec<Model<E>>) -> Model<E> {
        let mut all_vertices = vec![];
        for model in source_models.into_iter() {
            for vertex in model.vertices.into_iter() {
                all_vertices.push(vertex);
            }
        }
        Model {
            name: name.to_string(),
            vertices: all_vertices
        }
    }
}

/// StaticVertex struct
/// Vertex definition for a three-dimensional vertex with a position, normal and two-
/// dimensional texture coordinate
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaticVertex {
    pub px: f32,
    pub py: f32,
    pub pz: f32,
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
    pub tu: f32,
    pub tv: f32
}

impl StaticVertex {

    /// Construct a new instance from individual components
    pub fn from_components(
        p: (f32, f32, f32),
        n: (f32, f32, f32),
        t: (f32, f32)
    ) -> StaticVertex {
        StaticVertex { px: p.0, py: p.1, pz: p.2, nx: n.0, ny: n.1, nz: n.2, tu: t.0, tv: t.1 }
    }
}

impl Default for StaticVertex {

    /// Construct a new instance with position at the origin, texture coordinates at the origin,
    /// and a normal vector pointing in the positive Z direction.
    fn default() -> Self {
        StaticVertex { px: 0.0, py: 0.0, pz: 0.0, nx: 0.0, ny: 0.0, nz: 1.0, tu: 0.0, tv: 0.0 }
    }
}
