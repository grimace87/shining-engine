
use crate::types::{Model, StaticVertex};
use std::{
    path::Path,
    fs::File,
    io::Write
};

const VERTEX_SIZE_BYTES: usize = 32;

pub trait StoresAsFile<E> where E : Sized {

    /// # Safety
    ///
    /// Bytes should come from a file previously written by write_to_binary_file, and which used
    /// the same generic type
    unsafe fn new_from_bytes(bytes: &[u8]) -> Result<Self, String> where Self : Sized;

    /// # Safety
    ///
    /// Should be fine?
    unsafe fn write_to_binary_file(&self, file_path: &Path) -> Result<(), String>;
}

impl StoresAsFile<StaticVertex> for Model<StaticVertex> {

    unsafe fn new_from_bytes(
        bytes: &[u8]
    ) -> Result<Model<StaticVertex>, String> {

        // Read in vertex data
        let name_length: usize = *(bytes as *const [u8] as *const u32) as usize;
        let name = String::from_utf8_unchecked(bytes[4..(4 + name_length)].to_vec());
        let vertex_count: u32 =
            *(&bytes[(4 + name_length)..(8 + name_length)] as *const [u8] as *const u32);
        let mut vertices: Vec<StaticVertex> =
            vec![StaticVertex::default(); vertex_count as usize];
        let vertex_src_ptr =
            bytes[(8 + name_length)..(8 + name_length + vertex_count as usize * VERTEX_SIZE_BYTES)]
                .as_ptr() as *const StaticVertex;
        let vertex_src_slice =
            std::slice::from_raw_parts(vertex_src_ptr, vertex_count as usize);
        vertices.copy_from_slice(vertex_src_slice);

        // Done
        Ok(Model::<StaticVertex> {
            name,
            vertices
        })
    }

    unsafe fn write_to_binary_file(&self, file_path: &Path) -> Result<(), String> {

        // Open the file for writing
        let mut file = File::create(file_path)
            .map_err(|e| format!("Error opening file: {:?} - {:?}", file_path, e))?;

        // Put all the model's data in there
        file.write_all(&(self.name.len() as u32).to_ne_bytes()).unwrap();
        file.write_all(self.name.as_bytes()).unwrap();
        file.write_all(&(self.vertices.len() as u32).to_ne_bytes()).unwrap();
        for vertex in self.vertices.iter() {
            file.write_all(
                &*(vertex as *const StaticVertex as *const [u8; VERTEX_SIZE_BYTES])
            ).unwrap();
        }

        // Done
        Ok(())
    }
}
