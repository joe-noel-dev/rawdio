use crate::{
    commands::Id,
    graph::{DspParameters, DspProcessor},
    AudioBuffer, SampleLocation, Timestamp,
};

pub struct GainProcessor {
    gain_id: Id,
}

impl GainProcessor {
    pub fn new(gain_id: Id) -> Self {
        Self { gain_id }
    }
}

impl DspProcessor for GainProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameters,
    ) {
        let sample_rate = output_buffer.sample_rate();

        let gain_parameter = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        for (frame, (output, input)) in output_buffer
            .get_data_mut(SampleLocation::origin())
            .iter_mut()
            .zip(input_buffer.get_data(SampleLocation::origin()))
            .enumerate()
        {
            let frame_time = start_time.incremented_by_samples(frame, sample_rate);
            let gain = gain_parameter.get_value_at_time(&frame_time);
            *output = *input * (gain as f32);
        }
    }
}
