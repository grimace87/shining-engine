mod consumer;
mod producer;

pub use consumer::AudioConsumer;
pub use producer::AudioStreamProducer;

#[derive(Clone, PartialEq)]
pub struct AudioStreamProperties {
    pub sample_rate: u32,
    pub channels: u32,
    pub sample_format: AudioSampleFormat
}

#[derive(Clone, PartialEq)]
pub enum AudioSampleFormat {
    I16
}

#[cfg(test)]
mod tests;
