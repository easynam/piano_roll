pub trait Effect: Send {
    fn process_audio(&mut self, sample: usize, data: &mut [f64]);
}

pub struct Delay {
    delay: Vec<f64>,
    cursor: usize,
    wet: f64,
    dry: f64,
    feedback: f64,
}

impl Delay {
    pub fn new(len: usize, wet: f64, dry: f64, feedback: f64) -> Self {
        let delay = vec![Default::default(); len];
        Self {delay, cursor: 0, wet, dry, feedback}
    }

    fn process_sample(&mut self, sample: f64) -> f64 {
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
    fn process_audio(&mut self, _sample_pos: usize, data: &mut [f64]) {
        for sample in data.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}
