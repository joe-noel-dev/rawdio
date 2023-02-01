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
        _start_time: &Timestamp,
        parameters: &DspParameters,
    ) {
        let gain = parameters.get_parameter_values(self.gain_id, output_buffer.frame_count());

        let channel_count =
            std::cmp::min(output_buffer.channel_count(), input_buffer.channel_count());

        output_buffer.copy_from(
            input_buffer,
            SampleLocation::origin(),
            SampleLocation::origin(),
            channel_count,
            output_buffer.frame_count(),
        );

        output_buffer.apply_gain(gain);
    }
}
