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

use itertools::izip;

impl DspProcessor for GainProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        _start_time: &Timestamp,
        parameters: &DspParameters,
    ) {
        let gain = parameters.get_parameter_values(self.gain_id);

        let channel_count =
            std::cmp::min(output_buffer.channel_count(), input_buffer.channel_count());

        for channel in 0..channel_count {
            for (output, input, gain) in izip!(
                output_buffer.get_channel_data_mut(SampleLocation::new(channel, 0)),
                input_buffer.get_channel_data(SampleLocation::new(channel, 0)),
                gain
            ) {
                *output = *input * (*gain as f32);
            }
        }
    }
}
