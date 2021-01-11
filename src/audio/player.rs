use std::sync::{Arc, Mutex};

use crate::sequence::{Pitch, Sequence};

use super::controller::{Controller, Event, EventData};
use std::cmp::max;

pub struct Player {
    notes: Arc<Mutex<Sequence>>,
    sequence: usize,
    start_sample: usize,
    looping: Option<(usize, usize)>,
    samples_per_tick: usize,
    controller: Box<dyn Controller>,
    cursor: usize,
    playing: bool,
    preview: Option<Pitch>,
}

impl Player {
    pub fn new(
        samples_per_tick: usize,
        notes: Arc<Mutex<Sequence>>,
        controller: Box<dyn Controller>,
    ) -> Self {
        Self {
            notes,
            sequence: 0,
            start_sample: 0,
            looping: None,
            samples_per_tick,
            controller,
            cursor: 0,
            playing: false,
            preview: None,
        }
    }

    pub fn play_at(&mut self, start_sample: usize, looping: Option<(usize, usize)>, cursor: usize) {
        self.start_sample = start_sample;
        self.looping = looping.map(|x| (x.0 * self.samples_per_tick, x.1 * self.samples_per_tick));
        self.cursor = cursor * self.samples_per_tick;
        self.playing = true;
    }

    pub fn get_position_at(&self, sample: usize) -> i32 {
        if !self.playing {
            return 0;
        }

        if sample < self.start_sample {
            return 0;
        }

        let sample_delta = sample - self.start_sample;

        return (sample_delta / self.samples_per_tick) as i32;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn process(&mut self, samples: usize) {
        if !self.playing {
            return;
        }

        self.scan_events(samples);
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

    fn scan_events(&mut self, mut length: usize) {
        let cursor_end = self.cursor + length;

        if let Some((loop_start_sample, loop_end_sample)) = self.looping {
            if self.cursor < loop_end_sample && cursor_end >= loop_end_sample {
                self.scan_event_range(self.cursor, loop_end_sample);
                length -= loop_end_sample - self.cursor;
                self.start_sample += self.cursor - loop_start_sample;
                self.cursor = loop_start_sample;

                // length may be zero at the exact end sample of a loop, in which case
                // it is pointless to scan notes again
                if length == 0 {
                    return;
                }
            }
        }

        self.scan_event_range(self.cursor, self.cursor + length);
        self.cursor += length;
    }

    fn scan_event_range(&mut self, range_start: usize, range_end: usize) {
        let notes = self.notes.lock().unwrap();

        for note in notes.iter() {
            let start_sample = note.tick as usize * self.samples_per_tick;
            let end_sample = start_sample + note.length as usize * self.samples_per_tick;
            if start_sample >= range_start && start_sample < range_end {
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
    }
}
