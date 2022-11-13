use crate::{
    graph::{DspParameterMap, DspProcessor},
    AudioBuffer, SampleLocation, Timestamp,
};

pub struct SplitterProcessor {}

impl SplitterProcessor {
    pub fn new() -> Self {
        SplitterProcessor {}
    }
}

impl DspProcessor for SplitterProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        _start_time: &Timestamp,
        _parameters: &DspParameterMap,
    ) {
        let input_channel_count = input_buffer.num_channels();
        let output_channel_count = output_buffer.num_channels();

        let common_channel_count = input_channel_count.min(output_channel_count);

        output_buffer.copy_from(
            input_buffer,
            SampleLocation::origin(),
            SampleLocation::origin(),
            common_channel_count,
            output_buffer.num_frames(),
        );

        for output_channel in
            (input_channel_count..output_channel_count).step_by(input_channel_count)
        {
            let input_channel = output_channel % input_channel_count;

            let input_location = SampleLocation::new(input_channel, 0);
            let output_location = SampleLocation::new(output_channel, 0);

            let channel_count = input_channel_count.min(output_channel_count - output_channel);

            output_buffer.copy_from(
                input_buffer,
                input_location,
                output_location,
                channel_count,
                output_buffer.num_frames(),
            );
        }
    }
}
