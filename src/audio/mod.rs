use std::{sync::mpsc::{Receiver, SyncSender, sync_channel}};
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub enum Command {
    Play,
    Stop,
}


pub struct Synth {
    recv: Receiver<Command>,
}

impl Synth {
    pub fn create() -> (SyncSender<Command>, Synth) {
        let (tx, rx) = sync_channel(64);
        (
            tx,
            Synth {
                recv: rx
            }
        )
    }

    pub fn test_run<T>(device: cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let settings = redoxsynth::Settings::new().unwrap();
        let mut synth = redoxsynth::Synth::new(settings).unwrap();
        synth.set_sample_rate(sample_rate);
        synth.sfload("gm.sf2", true).unwrap();
        synth.note_on(0, 60, 127).unwrap();
        synth.note_on(0, 64, 127).unwrap();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                synth.write(data).unwrap();
            },
            err_fn,
        ).unwrap();
        stream.play().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    pub fn run(self) {
        loop {
            sleep(Duration::from_millis(100));
            while let Ok(command) = self.recv.try_recv() {
                match command {
                    Command::Play => {
                        let host = cpal::default_host();
                        let device = host.default_output_device().unwrap();
                        Synth::test_run::<f32>(device, &cpal::StreamConfig{
                            channels: 2,
                            sample_rate: cpal::SampleRate(48000),
                            buffer_size: cpal::BufferSize::Default,
                        });
                    }
                    Command::Stop => println!("stop!"),
                }
            }
        }
    }
}