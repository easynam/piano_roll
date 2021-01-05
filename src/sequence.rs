// use fraction::{Fraction, BigFraction, DynaFraction, ToPrimitive, Ratio};
use num_rational::{Rational32, Ratio};
use num_traits::cast::ToPrimitive;
use std::ops::{Neg, Add, Sub, Div};
use num_bigint::BigInt;

pub type Sequence = Vec<Note>;

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

    pub fn midi_pitch(&self) -> u32 {
        (self.0.to_f32().unwrap() * 12.0 + 69.0).round() as u32
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

#[derive(Debug, Clone)]
pub enum SequenceChange {
    Add(Note),
    Remove(usize),
    Update(usize, Note),
}

pub fn update_sequence(seq: &mut Sequence, message: SequenceChange) {
    match message {
        SequenceChange::Add(note) => {
            seq.push(note);
        },
        SequenceChange::Remove(idx) => {
            seq.remove(idx);
        },
        SequenceChange::Update(idx, note) => {
            seq[idx] = note;
        },
    }
}