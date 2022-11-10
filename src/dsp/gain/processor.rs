use crate::{
    commands::id::Id,
    graph::dsp::{DspParameterMap, DspProcessor},
    AudioBuffer, Timestamp,
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
        parameters: &DspParameterMap,
    ) {
        let sample_rate = output_buffer.sample_rate();

        let gain_parameter = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        for (frame, (output_location, input_location)) in output_buffer
            .frame_iter()
            .zip(input_buffer.frame_iter())
            .enumerate()
        {
            let frame_time = start_time.incremented_by_samples(frame, sample_rate);
            let gain = gain_parameter.get_value_at_time(&frame_time);

            let input_value = input_buffer.get_sample(input_location);
            let output_value = input_value * (gain as f32);
            output_buffer.set_sample(output_location, output_value);
        }
    }
}
