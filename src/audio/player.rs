use std::sync::{Arc, Mutex};

use crate::sequence::{Pitch, Sequence};

use super::controller::{Controller, Event, EventData};
use std::cmp::max;

pub struct Player {
    notes: Arc<Mutex<Sequence>>,
    sequence: usize,
    start_sample: usize,
    start_cursor: usize,
    looping: Option<(usize, usize)>,
    samples_per_tick: usize,
    controller: Box<dyn Controller>,
    cursor: usize,
    playing: bool,
    preview: Option<Pitch>,
    buffer_size: usize,
    playing_frame: usize,
}

impl Player {
    pub fn new(
        samples_per_tick: usize,
        notes: Arc<Mutex<Sequence>>,
        controller: Box<dyn Controller>,
        buffer_size: usize,
    ) -> Self {
        Self {
            notes,
            sequence: 0,
            start_sample: 0,
            start_cursor: 0,
            looping: None,
            samples_per_tick,
            controller,
            cursor: 0,
            playing: false,
            preview: None,
            buffer_size,
            playing_frame: 0,
        }
    }

    pub fn set_loop(&mut self, looping: Option<(i32, i32)>) {
        self.looping = looping.map(|x| (x.0 as usize * self.samples_per_tick, x.1 as usize * self.samples_per_tick));
    }

    pub fn seek(&mut self, start_sample: usize, cursor: i32) {
        self.playing_frame = cursor as usize * self.samples_per_tick;
        if self.playing {
            self.playing = false;
            self.play(start_sample);
        }
    }

    pub fn play(&mut self, start_sample: usize) {
        if !self.playing {
            self.start_sample = start_sample;
            self.cursor = self.playing_frame;
            self.playing = true;
            self.start_cursor = self.cursor;
            self.scan_events(self.buffer_size);
        }
    }

    pub fn get_position(&self) -> i32 {
        return self.playing_frame as i32 / self.samples_per_tick as i32;
    }

    pub fn pause(&mut self) {
        self.playing = false;

        self.controller.send_event(Event {
            sample: 0,
            sequence: self.sequence,
            data: EventData::ClearEvents,
        });
        self.sequence += 1;
    }

    pub fn process(&mut self, samples: usize) {
        if !self.playing {
            return;
        }

        self.advance_playing_frame(samples);
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

    fn advance_playing_frame(&mut self, length: usize) {
        let playing_frame_end = self.playing_frame + length;

        if let Some((loop_start_sample, loop_end_sample)) = self.looping {
            if self.playing_frame < loop_end_sample && playing_frame_end >= loop_end_sample {
                self.playing_frame -= loop_end_sample - loop_start_sample;

                return;
            }
        }

        self.playing_frame = playing_frame_end;
    }

    fn scan_events(&mut self, mut length: usize) {
        let cursor_end = self.cursor + length;

        if let Some((loop_start_sample, loop_end_sample)) = self.looping {
            if self.cursor < loop_end_sample && cursor_end >= loop_end_sample {
                self.scan_event_range(self.cursor, loop_end_sample);
                length -= loop_end_sample - self.cursor;
                self.start_sample += loop_end_sample - loop_start_sample;
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
                    sample: self.start_sample + start_sample - self.start_cursor,
                    sequence: self.sequence,
                    data: EventData::NoteOn(0, note.pitch.clone()),
                });
                self.sequence += 1;
                self.controller.send_event(Event {
                    sample: self.start_sample + end_sample - self.start_cursor,
                    sequence: self.sequence,
                    data: EventData::NoteOff(0, note.pitch.clone()),
                });
                self.sequence += 1;
            }
        }
    }
}
