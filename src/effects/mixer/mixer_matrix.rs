use crate::Level;

const MAX_INPUT_COUNT: usize = 8;
const MAX_OUTPUT_COUNT: usize = 8;

#[derive(Clone)]
pub struct MixerMatrix {
    input_count: usize,
    output_count: usize,
    matrix: [Level; MAX_INPUT_COUNT * MAX_OUTPUT_COUNT],
}

impl MixerMatrix {
    pub fn new(input_count: usize, output_count: usize) -> Self {
        assert!(input_count <= MAX_INPUT_COUNT);
        assert!(output_count <= MAX_OUTPUT_COUNT);

        Self {
            input_count,
            output_count,
            matrix: [Level::zero(); MAX_INPUT_COUNT * MAX_OUTPUT_COUNT],
        }
    }

    pub fn empty() -> Self {
        Self::new(0, 0)
    }

    pub fn set_level(&mut self, input_channel: usize, output_channel: usize, level: Level) {
        let index = self.get_index(input_channel, output_channel);
        self.matrix[index] = level;
    }

    pub fn get_level(&self, input_channel: usize, output_channel: usize) -> Level {
        if input_channel < self.input_count && output_channel < self.output_count {
            let index = self.get_index(input_channel, output_channel);
            return self.matrix[index];
        }

        Level::zero()
    }

    fn get_index(&self, input_channel: usize, output_channel: usize) -> usize {
        assert!(input_channel < self.input_count);
        assert!(output_channel < self.output_count);
        output_channel * self.input_count + input_channel
    }
}
