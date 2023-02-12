
use crate::{AudioStreamProducer, AudioStreamProperties};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioConsumer {
    device: cpal::Device,
    config: cpal::StreamConfig,
    pub properties: AudioStreamProperties,
    stream: Option<cpal::Stream>
}

impl AudioConsumer {

    pub fn try_new(properties: AudioStreamProperties) -> Option<Self> {

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("Default output device not available");
                return None;
            }
        };

        let mut supported_configs = match device.supported_output_configs() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Could not query output configs: {:?}", e);
                return None;
            }
        };

        let lib_sample_format = match properties.sample_format {
            crate::AudioSampleFormat::I16 => cpal::SampleFormat::I16
        };

        let matching_range = supported_configs.find(|range| {
            let matched_rate = properties.sample_rate >= range.min_sample_rate().0 &&
                properties.sample_rate <= range.max_sample_rate().0;
            let matched_channels = properties.channels == range.channels().into();
            let matched_format = lib_sample_format == range.sample_format();
            matched_rate && matched_channels && matched_format
        });

        if matching_range.is_none() {
            eprintln!("Default config not available");
            return None;
        }

        let config = cpal::StreamConfig {
            channels: properties.channels as cpal::ChannelCount,
            sample_rate: cpal::SampleRate(properties.sample_rate),
            buffer_size: cpal::BufferSize::Default
        };

        Some(Self {
            device,
            config,
            properties,
            stream: None
        })
    }

    pub fn start<P>(&mut self, mut producer: P)
            where P: AudioStreamProducer + Send + 'static,
                  <P as AudioStreamProducer>::Sample: cpal::Sample {
        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [P::Sample], _: &cpal::OutputCallbackInfo| {
                unsafe { producer.fill_buffer(data, data.len()); }
            },
            move |err| {
                eprintln!("Error during playback: {:?}", err);
            }
        ).unwrap();
        if let Err(e) = stream.play() {
            eprintln!("Error trying to start playback: {:?}", e);
        }
        self.stream = Some(stream);
    }

    pub fn stop(&mut self) {
        if let Some(stream) = &self.stream {
            if let Err(e) = stream.pause() {
                eprintln!("Error trying to pause playback: {:?}", e);
            }
        }
        self.stream = None;
    }
}
