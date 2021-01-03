use super::effect::Effect;

pub trait Source: Send {
    fn output_audio(&mut self, sample: usize, data: &mut [f32]);
}

pub struct FxSource {
    source: Box<dyn Source>,
    effects: Vec<Box<dyn Effect>>,
}

impl FxSource {
    pub fn new(source: Box<dyn Source>, effects: Vec<Box<dyn Effect>>) -> FxSource {
        Self{source, effects}
    }
}

impl Source for FxSource {
    fn output_audio(&mut self, sample: usize, data: &mut [f32]) {
        self.source.output_audio(sample, data);
        for effect in self.effects.iter_mut() {
            effect.process_audio(sample, data);
        }
    }
}
