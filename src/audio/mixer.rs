use super::source::Source;

pub struct Mixer {
    sources: Vec<Box<dyn Source>>,
    buffer: Vec<f32>,
}

impl Mixer {
    /// Creates a new mixer with the provided buffer length. Generally, the
    /// buffer length should be large enough to fill the entire output buffer;
    /// otherwise, the mixing process will run in multiple chunks.
    fn new(buflen: usize) -> Self {
        Self {
            sources: Vec::new(),
            buffer: vec![Default::default(); buflen],
        }
    }
}

impl Source for Mixer {
    fn output_audio(&mut self, mut sample: usize, output: &mut [f32]) {
        for output_chunk in output.chunks_mut(self.buffer.len()) {
            // Replace with output_chunk.fill(Default::default()) in Rust 1.50.
            output_chunk
                .iter_mut()
                .for_each(|f| *f = Default::default());

            for source in self.sources.iter_mut() {
                let buffer_chunk = &mut self.buffer[0..output_chunk.len()];
                source.output_audio(sample, buffer_chunk);

                // TODO: This is ugly. Should figure out how to best handle
                // mixing when not using f32, or perhaps simply always use f32
                // internally.
                output_chunk
                    .iter_mut()
                    .zip(buffer_chunk)
                    .for_each(|(a, b)| *a += *b);
            }
                
            // TODO: hardcoded channel count
            sample += output_chunk.len() / 2;
        }
    }
}
