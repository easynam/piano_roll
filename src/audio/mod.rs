use std::sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
};
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::sequence::{Note, Sequence};

pub enum Command {
    Play,
    Stop,
}

pub struct Synth {
    recv: Receiver<Command>,
}

pub struct Event {
    sample: usize,
    data: EventData,
}

pub enum EventData {
    NoteOn(Note),
    NoteOff(Note),
}

impl Synth {
    pub fn create() -> (SyncSender<Command>, Synth) {
        let (tx, rx) = sync_channel(64);
        (tx, Synth { recv: rx })
    }

    fn insert_event(events: &mut Vec<Event>, sample: usize, data: EventData) {
        let event = Event { sample, data };

        let pos = events
            .binary_search_by_key(&event.sample, |e: &Event| e.sample)
            .unwrap_or_else(|e| e);

            events.insert(pos, event);
    }

    fn scan_events(
        cursor: usize,
        length: usize,
        samples_per_tick: usize,
        notes: &Vec<Note>,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let cursor_end = cursor + length;

        for note in notes.iter() {
            let start_sample = note.tick as usize * samples_per_tick;
            let end_sample = start_sample + note.length as usize * samples_per_tick;

            if start_sample >= cursor && start_sample < cursor_end {
                Self::insert_event(&mut events, start_sample, EventData::NoteOn(*note));
            }

            if end_sample >= cursor && end_sample < cursor_end {
                Self::insert_event(&mut events, end_sample, EventData::NoteOff(*note));
            }
        }

        return events;
    }

    fn test_run<T>(notes: Arc<Mutex<Sequence>>, device: cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let settings = redoxsynth::Settings::new().unwrap();
        let mut synth = redoxsynth::Synth::new(settings).unwrap();
        synth.set_sample_rate(sample_rate);
        synth.sfload("gm.sf2", true).unwrap();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let mut cursor = 0;
        let samples_per_tick = 200;
        let ch = config.channels as usize;
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let notes = notes.lock().unwrap();
                    let length = data.len() / ch;
                    let events = Self::scan_events(cursor, length, samples_per_tick, notes.as_ref());
                    let mut i = 0;

                    for event in events.iter()
                    {
                        if event.sample > cursor + i {
                            let gen_samples = event.sample - (cursor + i);

                            synth
                                .write(&mut data[i * ch..(i + gen_samples) * ch])
                                .unwrap();

                            i += gen_samples;
                        }

                        match event.data {
                            EventData::NoteOn(n) => {
                                synth.note_on(0, 96 - n.note as u32, 127);
                            }
                            EventData::NoteOff(n) => {
                                synth.note_off(0, 96 - n.note as u32);
                            }
                        }
                    }

                    synth.write(&mut data[i * ch..length * ch]).unwrap();
                    cursor += length;
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(2000));
    }

    pub fn run(self, notes: Arc<Mutex<Sequence>>) {
        loop {
            sleep(Duration::from_millis(100));
            while let Ok(command) = self.recv.try_recv() {
                match command {
                    Command::Play => {
                        let host = cpal::default_host();
                        let device = host.default_output_device().unwrap();
                        Self::test_run::<f32>(
                            notes.clone(),
                            device,
                            &cpal::StreamConfig {
                                channels: 2,
                                sample_rate: cpal::SampleRate(48000),
                                buffer_size: cpal::BufferSize::Default,
                            },
                        );
                    }
                    Command::Stop => println!("stop!"),
                }
            }
        }
    }
}
