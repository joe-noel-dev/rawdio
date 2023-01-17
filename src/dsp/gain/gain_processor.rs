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
        let gain_parameter = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        let num_channels = std::cmp::min(output_buffer.num_channels(), input_buffer.num_channels());

        for channel in 0..num_channels {
            for (output, input, gain) in izip!(
                output_buffer.get_channel_data_mut(SampleLocation::new(channel, 0)),
                input_buffer.get_channel_data(SampleLocation::new(channel, 0)),
                gain_parameter.get_values()
            ) {
                *output = *input * (*gain as f32);
            }
        }
    }
}
