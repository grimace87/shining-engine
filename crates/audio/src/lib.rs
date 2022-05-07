pub mod consumer;

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

pub trait AudioStreamProducer {
    type Sample;
    unsafe fn fill_buffer(&mut self, data: &mut [Self::Sample], size_bytes: usize);
    fn get_properties(&self) -> AudioStreamProperties;
}

#[cfg(test)]
mod test {
    use crate::{
        consumer::AudioConsumer,
        AudioStreamProducer, AudioStreamProperties, AudioSampleFormat
    };

    const FRAMES_PER_VALUE: usize = 64;
    const MAX_VOLUME: i16 = 0x2000;
    const VOLUME_STEP: i16 = 0x0200;
    const FEEDBACK_MASK: u32 = 0x4000;

    struct NoiseTest {
        progress: usize,
        volume: i16,
        lfsr: u32
    }

    impl AudioStreamProducer for NoiseTest {
        type Sample = i16;

        unsafe fn fill_buffer(&mut self, data: &mut [i16], size_bytes: usize) {
            let frames = size_bytes / 4;
            for f in 0..frames {
                let buffer_index = f * 2;
                let sample = self.next_value();
                data[buffer_index] = sample;
                data[buffer_index + 1] = sample;
            }
        }

        fn get_properties(&self) -> AudioStreamProperties {
            AudioStreamProperties {
                sample_rate: 48000,
                channels: 2,
                sample_format: AudioSampleFormat::I16
            }
        }
    }

    impl NoiseTest {

        fn new() -> Self {
            Self { progress: 0, lfsr: 0x0001, volume: MAX_VOLUME }
        }

        fn next_value(&mut self) -> i16 {

            // Update progress ticker
            self.progress = self.progress + 1;
            if self.progress >= FRAMES_PER_VALUE {
                self.progress = self.progress - FRAMES_PER_VALUE;

                // When ticking over, decrease volume
                self.volume = self.volume - VOLUME_STEP;
                if self.volume < 0 {
                    self.volume = MAX_VOLUME;
                }

                // Also update the shift register
                let feedback_bits: u32 = ((self.lfsr & 0x0001) ^ ((self.lfsr & 0x0002) >> 1)) * FEEDBACK_MASK;
                let shifted: u32 = self.lfsr >> 1;
                self.lfsr = (shifted & !feedback_bits) | feedback_bits;
            }

            // Get value based on current LSB of the shift register and the current volume
            let sign = (self.lfsr & 0x0001) as i16 * 2 - 1;
            sign * self.volume
        }
    }

    #[test]
    fn plays_back_for_five_seconds() {
        let producer = NoiseTest::new();
        let consumer_result = AudioConsumer::try_new(producer.get_properties());
        assert!(consumer_result.is_some());

        let mut consumer = consumer_result.unwrap();
        consumer.start(producer);
        std::thread::sleep(std::time::Duration::from_millis(5000));
        consumer.stop();
    }
}
