use std::sync::{Arc, atomic::{AtomicUsize, Ordering}, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::futures::channel::mpsc::{Receiver, Sender, channel};

use super::source::Source;
use std::borrow::BorrowMut;

pub struct AudioEmitter {
    stream: Option<cpal::Stream>,
    sender: Arc<Mutex<Sender<usize>>>,
    config: cpal::StreamConfig,
}

impl AudioEmitter {
    pub fn new() -> (Self, Receiver<usize>) {
        let (tx, rx) = channel(64);

        (
            Self {
                stream: None,
                config: cpal::StreamConfig {
                    channels: 2,
                    sample_rate: cpal::SampleRate(48000),
                    buffer_size: cpal::BufferSize::Default,
                },
                sender: Arc::new(Mutex::new(tx)),
            },
            rx
        )
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        return self.config.clone();
    }

    pub fn start(&mut self, source: Box<dyn Source>) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        self.stream = Some(Self::make_output_stream(
            device,
            &self.config,
            source,
            self.sender.clone(),
        ));
    }

    fn make_output_stream(
        device: cpal::Device,
        config: &cpal::StreamConfig,
        mut source: Box<dyn Source>,
        mut sender: Arc<Mutex<Sender<usize>>>,
    ) -> cpal::Stream {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let mut counter = 0usize;
        let ch = config.channels as usize;

        let mut buffer = Vec::new();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    buffer.resize(data.len(), Default::default());
                    source.output_audio(counter, buffer.as_mut_slice());
                    data.iter_mut().zip(buffer.iter()).for_each(|(dst, src)| *dst = *src as f32);
                    sender.lock().unwrap().try_send(data.len() / ch);
                    counter += data.len() / ch;
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();
        stream
    }
}
