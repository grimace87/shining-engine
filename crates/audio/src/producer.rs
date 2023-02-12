
use crate::AudioStreamProperties;

pub trait AudioStreamProducer {
    type Sample;

    /// Fills a chunk of data for the audio stream.
    ///
    /// # Safety
    /// Applications must ensure that the size of the buffer is correctly represented in the
    /// size_bytes field
    unsafe fn fill_buffer(&mut self, data: &mut [Self::Sample], size_bytes: usize);

    /// Gets the audio format properties for the stream
    fn get_properties(&self) -> AudioStreamProperties;
}
