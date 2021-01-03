pub trait Source: Send {
    fn output_audio(&mut self, sample: usize, data: &mut [f32]);
}
