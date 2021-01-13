// use fraction::{Fraction, BigFraction, DynaFraction, ToPrimitive, Ratio};
use num_rational::{Rational32, Ratio};
use num_traits::cast::ToPrimitive;
use std::ops::{Neg, Add, Sub, Div};
use num_bigint::BigInt;
use slotmap::{new_key_type, SlotMap};
use slotmap::basic::Iter;

new_key_type! {
    pub struct NoteId;
}

#[derive(Debug, Clone)]
pub struct Sequence {
    slotmap: SlotMap<NoteId, Note>,
    last_added: Option<NoteId>,
    note_starts: Vec<(i32, NoteId)>
}

#[derive(Debug, Clone)]
pub enum SequenceChange {
    Add(Note),
    Remove(NoteId),
    Update(NoteId, Note),
}

impl Sequence {
    pub fn new() -> Self {
        Self {
            slotmap: SlotMap::with_key(),
            last_added: None,
            note_starts: vec![]
        }
    }

    pub fn update_sequence(&mut self, message: SequenceChange) {
        match message {
            SequenceChange::Add(note) => {
                let new_id = self.slotmap.insert(note.clone());
                self.last_added = Some(new_id);
                let start_idx = self.note_starts
                    .binary_search(&(note.tick, new_id))
                    .expect_err("note_starts out of sync with slotmap");

                self.note_starts.insert(start_idx, (note.tick, new_id));
            },
            SequenceChange::Remove(id) => {
                if let Some(old_note) = self.slotmap.remove(id) {
                    let _ = self.note_starts
                        .binary_search(&(old_note.tick, id))
                        .map(|idx| self.note_starts.remove(idx))
                        .expect("note_starts out of sync with slotmap");
                }
            },
            SequenceChange::Update(id, new_note) => {
                if let Some(note) = self.slotmap.get_mut(id) {
                    let idx = self.note_starts
                        .binary_search(&(note.tick, id))
                        .expect("note_starts out of sync with slotmap");

                    *note = new_note.clone();

                    self.note_starts.remove(idx);

                    let start_idx = self.note_starts
                        .binary_search(&(new_note.tick, id))
                        .expect_err("note_starts out of sync with slotmap");

                    self.note_starts.insert(start_idx, (new_note.tick, id));
                }
            },
        }
    }

    /// exclusive range
    pub fn get_notes_in_range(&self, start_tick: i32, end_tick: i32) -> Vec<(NoteId, Note)> {
        let start_idx = self.note_starts
            .binary_search_by_key(&start_tick, |(tick, _id)| *tick)
            .unwrap_or_else(|idx| idx);

        let end_idx = self.note_starts
            .binary_search_by_key(&end_tick, |(tick, _id)| *tick)
            .unwrap_or_else(|idx| idx);

        self.note_starts[start_idx..end_idx].iter().map(|(_tick, id)| (*id, self.slotmap.get(*id).unwrap().clone())).collect()
    }

    pub fn last_added(&self) -> Option<(NoteId, &Note)> {
        self.last_added.and_then(|key| {
            self.slotmap.get(key).map(|note| (key, note))
        })
    }

    pub fn iter(&self) -> Iter<NoteId, Note> {
        self.slotmap.iter()
    }

    pub fn get(&self, id: NoteId) -> Option<&Note> {
        self.slotmap.get(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub tick: i32,
    pub pitch: Pitch,
    pub length: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pitch(pub Rational32);

impl Add for Pitch {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pitch(self.0.add(rhs.0))
    }
}

impl Sub for Pitch {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pitch(self.0.sub(rhs.0))
    }
}

impl Neg for Pitch {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Pitch(self.0.neg())
    }
}

impl Pitch {
    pub fn new(num: i32, den: i32) -> Self {
        Pitch(Ratio::new(num, den))
    }

    pub fn from_octave(octave: Rational32) -> Self {
        Pitch(octave)
    }

    pub fn from_octave_f32(octave: f32) -> Self {
        let (mut num, mut den) = {
            let ratio = Ratio::from_float(octave).unwrap();
            (ratio.numer().clone(), ratio.denom().clone())
        };

        let max: BigInt = BigInt::from(std::i32::MAX);

        while num > max || den > max {
            num = num.div(2);
            den = den.div(2);
        }

        Pitch(Ratio::new(num.to_i32().unwrap(), den.to_i32().unwrap()))
    }

    pub fn hz(&self) -> f32 {
        let octave = self.0.to_f32().unwrap();
        440.0 * 2.0_f32.powf(octave)
    }

    pub fn midi_pitch(&self, pitch_bend_range: f32) -> (u32, u32) {
        let midi_pitch = self.0.to_f32().unwrap() * 12.0 + 69.0;
        let rounded = midi_pitch.round();
        let pitch_bend = ((midi_pitch - rounded) * 8192.0 / pitch_bend_range + 8192.0) as u32;
        (rounded as u32, pitch_bend)
    }

    pub fn to_f32(&self) -> f32 {
        self.0.to_f32().unwrap()
    }
}

impl Note {
    pub fn end_tick(&self) -> i32 {
        self.tick + self.length
    }
}