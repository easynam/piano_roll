use crate::sequence::Pitch;
use std::ops::{Mul, Div};
use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    White,
    Black,
}

pub struct GridLine {
    pub pitch: Pitch,
    pub line_type: LineType,
}

pub trait PitchGrid {
    fn get_grid_lines(&self, start: Pitch, end: Pitch) -> Vec<GridLine>;
    fn quantize_pitch(&self, pitch: Pitch) -> Pitch;
}

pub struct TetGrid {
    pub tones_per_octave: i32,
    pub pattern: Vec<LineType>,
}

impl TetGrid {
    pub fn new(tones_per_octave: i32, pattern: Vec<LineType>) -> Self {
        TetGrid { tones_per_octave, pattern }
    }
}

impl PitchGrid for TetGrid {
    fn get_grid_lines(&self, start: Pitch, end: Pitch) -> Vec<GridLine> {
        let steps_from_0 = start.0.mul(self.tones_per_octave).ceil().to_integer();
        let mut ratio = Ratio::new(steps_from_0, self.tones_per_octave);

        let pattern_offset = steps_from_0.rem_euclid(self.tones_per_octave) as usize;

        let mut ratios = vec![];
        while ratio <= end.0 {
            ratios.push(ratio.clone());
            ratio = ratio + Ratio::new(1, self.tones_per_octave);
        }

        ratios.iter().enumerate()
            .map(|(idx, ratio)| GridLine {
                pitch: Pitch::from_octave(*ratio),
                line_type: self.pattern[(idx + pattern_offset) % self.pattern.len()],
            })
            .collect()
    }

    fn quantize_pitch(&self, pitch: Pitch) -> Pitch {
        let ratio = pitch.0.mul(self.tones_per_octave).round().div(self.tones_per_octave);
        Pitch::from_octave(ratio)
    }
}