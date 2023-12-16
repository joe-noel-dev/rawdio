use crate::{graph::DspProcessor, ProcessContext, SampleLocation};

pub struct GainProcessor;

impl GainProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DspProcessor for GainProcessor {
    fn process_audio(&mut self, context: &mut ProcessContext) {
        let gain = context
            .parameters
            .get_parameter_values("gain", context.output_buffer.frame_count());

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
