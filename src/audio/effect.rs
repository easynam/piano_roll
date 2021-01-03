pub trait Effect: Send {
    fn process_audio(&mut self, sample: usize, data: &mut [f32]);
}

pub struct Delay {
    delay: Vec<f32>,
    cursor: usize,
    wet: f32,
    dry: f32,
    feedback: f32,
}

impl Delay {
    pub fn new(len: usize, wet: f32, dry: f32, feedback: f32) -> Self {
        let delay = vec![Default::default(); len];
        Self {delay, cursor: 0, wet, dry, feedback}
    }

    fn process_sample(&mut self, sample: f32) -> f32 {
        let delay_sample = self.delay[self.cursor];
        self.delay[self.cursor] = self.delay[self.cursor] * self.feedback + sample;
        self.cursor += 1;
        if self.cursor >= self.delay.len() {
            self.cursor = 0;
        }
        return delay_sample * self.wet + sample * self.dry;
    }
}

impl Effect for Delay {
    fn process_audio(&mut self, _sample_pos: usize, data: &mut [f32]) {
        for sample in data.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}
