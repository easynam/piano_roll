use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::futures::channel::mpsc::{Receiver, channel};
use super::source::Source;

pub struct AudioEmitter {
    stream: Option<cpal::Stream>,
    config: cpal::StreamConfig,
}

impl AudioEmitter {
    pub fn new() -> Self {
        Self {
            stream: None,
            config: cpal::StreamConfig {
                channels: 2,
                sample_rate: cpal::SampleRate(48000),
                buffer_size: cpal::BufferSize::Default,
            },
        }
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        return self.config.clone();
    }

    pub fn start(&mut self, source: Box<dyn Source>) -> Receiver<usize> {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let (stream, rx) = Self::make_output_stream(
            device,
            &self.config,
            source,
        );

        self.stream = Some(stream);
        rx
    }

    fn make_output_stream(
        device: cpal::Device,
        config: &cpal::StreamConfig,
        mut source: Box<dyn Source>,
    ) -> (cpal::Stream, Receiver<usize>) {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let mut counter = 0usize;
        let ch = config.channels as usize;

        let mut buffer = Vec::new();

        let (mut tx, rx) = channel(64);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    buffer.resize(data.len(), Default::default());
                    source.output_audio(counter, buffer.as_mut_slice());
                    data.iter_mut().zip(buffer.iter()).for_each(|(dst, src)| *dst = *src as f32);
                    tx.try_send(data.len() / ch);
                    counter += data.len() / ch;
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();
        (stream, rx)
    }
}
