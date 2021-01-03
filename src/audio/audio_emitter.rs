use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::source::Source;

pub struct AudioEmitter {
    stream: Option<cpal::Stream>,
    counter: Arc<AtomicUsize>,
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
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        return self.config.clone();
    }

    pub fn get_sample_pos(&self) -> usize {
        self.counter.load(Ordering::Acquire)
    }

    pub fn start(&mut self, source: Box<dyn Source>) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        self.stream = Some(Self::make_output_stream(
            device,
            &self.config,
            source,
            self.counter.clone(),
        ));
    }

    fn make_output_stream(
        device: cpal::Device,
        config: &cpal::StreamConfig,
        mut source: Box<dyn Source>,
        counter_atomic: Arc<AtomicUsize>,
    ) -> cpal::Stream {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let mut counter = 0usize;
        let ch = config.channels as usize;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    source.output_audio(counter, data);
                    counter += data.len() / ch;
                    counter_atomic.store(counter, Ordering::Release);
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();
        stream
    }
}
