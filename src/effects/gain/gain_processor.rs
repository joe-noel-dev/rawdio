use crate::{commands::Id, graph::DspProcessor, ProcessContext, SampleLocation};

pub struct GainProcessor {
    gain_id: Id,
}

impl GainProcessor {
    pub fn new(gain_id: Id) -> Self {
        Self { gain_id }
    }
}

impl DspProcessor for GainProcessor {
    fn process_audio(&mut self, context: &mut ProcessContext) {
        let gain = context
            .parameters
            .get_parameter_values(self.gain_id, context.output_buffer.frame_count());

        let channel_count = std::cmp::min(
            context.output_buffer.channel_count(),
            context.input_buffer.channel_count(),
        );

        context.output_buffer.copy_from(
            context.input_buffer,
            SampleLocation::origin(),
            SampleLocation::origin(),
            channel_count,
            context.output_buffer.frame_count(),
        );

        context.output_buffer.apply_gain(gain);
    }
}
