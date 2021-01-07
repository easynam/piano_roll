use std::sync::{Arc, Mutex};

use crate::sequence::{Sequence, Pitch};

use super::controller::{Controller, Event, EventData};

pub struct Player {
    notes: Arc<Mutex<Sequence>>,
    sequence: usize,
    start_sample: usize,
    samples_per_tick: usize,
    controller: Box<dyn Controller>,
    cursor: usize,
    playing: bool,
    preview: Option<Pitch>,
}

impl Player {
    pub fn new(samples_per_tick: usize, notes: Arc<Mutex<Sequence>>, controller: Box<dyn Controller>) -> Self {
        Self {
            notes,
            sequence: 0,
            start_sample: 0,
            samples_per_tick,
            controller,
            cursor: 0,
            playing: false,
            preview: None,
        }
    }

    pub fn play(&mut self, start_sample: usize) {
        self.start_sample = start_sample;
        self.cursor = 0;
        self.playing = true;
    }

    pub fn get_position_at(&self, sample: usize) -> Option<f64> {
        if !self.playing {
            return None;
        }

        if sample < self.start_sample {
            return Some(0.0);
        }

        let sample_delta = sample - self.start_sample;

        return Some(sample_delta as f64 / self.samples_per_tick as f64);
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn process(&mut self, sample: usize) {
        if !self.playing {
            return;
        }
        if self.start_sample + self.cursor > sample {
            return;
        }
        self.scan_events(sample - self.start_sample - self.cursor);
    }

    pub fn play_preview(&mut self, pitch: Pitch) {
        if let Some(old_pitch) = &self.preview {
            self.controller.send_event(Event {
                sample: 0,
                sequence: self.sequence,
                data: EventData::NoteOff(1, old_pitch.clone()),
            });
            self.sequence += 1;
        }
        self.controller.send_event(Event {
            sample: 0,
            sequence: self.sequence,
            data: EventData::NoteOn(1, pitch.clone()),
        });
        self.sequence += 1;

        self.preview = Some(pitch.clone());
    }

    pub fn stop_preview(&mut self) {
        if let Some(old_pitch) = &self.preview {
            self.controller.send_event(Event {
                sample: 0,
                sequence: self.sequence,
                data: EventData::NoteOff(1, old_pitch.clone()),
            });
            self.sequence += 1;
        }

        self.preview = None;
    }

    fn scan_events(
        &mut self,
        length: usize,
    ) {
        let cursor_end = self.cursor + length;
        let notes = self.notes.lock().unwrap();

        for note in notes.iter() {
            let start_sample = note.tick as usize * self.samples_per_tick;
            let end_sample = start_sample + note.length as usize * self.samples_per_tick;
            if start_sample >= self.cursor && start_sample < cursor_end {
                self.controller.send_event(Event {
                    sample: self.start_sample + start_sample,
                    sequence: self.sequence,
                    data: EventData::NoteOn(0, note.pitch.clone()),
                });
                self.sequence += 1;
                self.controller.send_event(Event {
                    sample: self.start_sample + end_sample,
                    sequence: self.sequence,
                    data: EventData::NoteOff(0, note.pitch.clone()),
                });
                self.sequence += 1;
            }
        }

        self.cursor += length;
    }
}
