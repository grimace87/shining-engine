
use crate::{ImageUsage, TexturePixelFormat, TextureCreationData};
use error::EngineError;
use model::{Model, StaticVertex, StoresAsFile};
use std::io::Cursor;
use image::{
    DynamicImage,
    codecs::jpeg::JpegDecoder,
    codecs::png::PngDecoder
};

#[derive(Copy, Clone)]
pub enum TextureCodec {
    Jpeg,
    Png
}

pub struct ResourceUtilities;

impl ResourceUtilities {
    /// Decode a model file generated by the model crate's utility functions.
    pub unsafe fn decode_model(model_file_bytes: &[u8]) -> (Vec<StaticVertex>, usize) {
        let model: Model<StaticVertex> = unsafe {
            Model::new_from_bytes(model_file_bytes).unwrap()
        };
        let vertex_count: usize = model.vertices.len();
        (model.vertices, vertex_count)
    }

    /// Decode texture data from a file, returning a defs::render::TextureCreationData instance
    pub fn decode_texture(
        image_file_bytes: &[u8],
        codec: TextureCodec,
        usage: ImageUsage
    ) -> Result<TextureCreationData, EngineError> {
        let (data, width, height) = match codec {
            TextureCodec::Jpeg => {
                let src_cursor = Cursor::new(image_file_bytes.to_vec());
                let decoder = JpegDecoder::new(src_cursor).unwrap();
                let image_pixel_data = DynamicImage::from_decoder(decoder)
                    .map_err(|e| EngineError::OpFailed(format!("Failed decoding image: {:?}", e)))?;
                let image_data_rgba = image_pixel_data.to_rgba8();
                (image_data_rgba.to_vec(), image_data_rgba.width(), image_data_rgba.height())
            },
            TextureCodec::Png => {
                let src_cursor = Cursor::new(image_file_bytes.to_vec());
                let decoder = PngDecoder::new(src_cursor).unwrap();
                let image_pixel_data = DynamicImage::from_decoder(decoder)
                    .map_err(|e| EngineError::OpFailed(format!("Failed decoding image: {:?}", e)))?;
                let image_data_rgba = image_pixel_data.to_rgba8();
                (image_data_rgba.to_vec(), image_data_rgba.width(), image_data_rgba.height())
            }
        };
        Ok(TextureCreationData {
            layer_data: Some(vec![data]),
            width,
            height,
            format: TexturePixelFormat::Rgba,
            usage
        })
    }
}
