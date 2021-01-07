use std::{fmt::Display, path::Path, sync::Arc};

use crossbeam::queue::SegQueue;

use super::{
    controller::{Controller, Event, EventData},
    source::Source,
};

pub struct RedoxSynthController {
    event_queue: Arc<SegQueue<Event>>,
}

pub struct RedoxSynthSource {
    synth: redoxsynth::Synth,
    events: Vec<Event>,
    event_queue: Arc<SegQueue<Event>>,
}

pub struct RedoxSynthGenerator {}

fn redoxsynth_error<T: Display>(err: T) -> String {
    format!("RedoxSynth error: {}", err)
}

impl RedoxSynthGenerator {
    pub fn new<P: AsRef<Path>>(
        sample_rate: f32,
        soundfont_filename: P,
    ) -> Result<(RedoxSynthController, RedoxSynthSource), String> {
        let settings = redoxsynth::Settings::new().map_err(redoxsynth_error)?;
        let mut synth = redoxsynth::Synth::new(settings).map_err(redoxsynth_error)?;
        synth.set_sample_rate(sample_rate);
        synth
            .sfload(soundfont_filename, true)
            .map_err(redoxsynth_error)?;

        let event_queue = Arc::new(SegQueue::new());
        Ok((
            RedoxSynthController::new(event_queue.clone()),
            RedoxSynthSource::new(synth, event_queue.clone()),
        ))
    }
}

impl RedoxSynthController {
    fn new(event_queue: Arc<SegQueue<Event>>) -> Self {
        Self { event_queue }
    }
}

impl Controller for RedoxSynthController {
    fn send_event(&self, event: Event) {
        self.event_queue.push(event);
    }
}

impl RedoxSynthSource {
    fn new(synth: redoxsynth::Synth, event_queue: Arc<SegQueue<Event>>) -> Self {
        Self {
            synth,
            events: Vec::new(),
            event_queue,
        }
    }

    fn insert_event(&mut self, event: Event) {
        let pos = self
            .events
            .binary_search_by_key(&(event.sample, event.sequence), |e: &Event| {
                (e.sample, e.sequence)
            })
            .unwrap_or_else(|e| e);

        self.events.insert(pos, event);
    }
}

impl Source for RedoxSynthSource {
    fn output_audio(&mut self, sample: usize, data: &mut [f64]) {
        // TODO: hardcoded channel count
        let length = data.len() / 2;

        loop {
            match self.event_queue.pop() {
                Some(event) => self.insert_event(event),
                None => break,
            }
        }

        let mut i = 0;
        for event in self.events.iter().filter(|e| e.sample < sample + length) {
            if event.sample > sample + i {
                let gen_samples = event.sample - (sample + i);

                // TODO: hardcoded channel count
                self.synth
                    .write(&mut data[i * 2..(i + gen_samples) * 2])
                    .unwrap();

                i += gen_samples;
            }

            match &event.data {
                EventData::NoteOn(chan, n) => {
                    let (key, bend) = n.midi_pitch(2.0);
                    self.synth.note_on(*chan, key, 127);
                    self.synth.pitch_bend(*chan, bend);
                }
                EventData::NoteOff(chan, n) => {
                    self.synth.note_off(*chan, n.midi_pitch(2.0).0);
                }
            }
        }

        self.events.retain(|e| e.sample >= sample + length);

        // TODO: hardcoded channel count
        self.synth.write(&mut data[i * 2..length * 2]).unwrap();
    }
}
