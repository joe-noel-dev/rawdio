use crate::{
    commands::id::Id,
    graph::dsp::{DspParameterMap, DspProcessor},
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
        parameters: &DspParameterMap,
    ) {
        let sample_rate = output_buffer.sample_rate() as f64;

        let gain = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        for frame in 0..output_buffer.num_frames() {
            let frame_time = start_time.incremented_by_samples(frame, sample_rate);
            let gain = gain.get_value_at_time(&frame_time);

            for channel in 0..output_buffer.num_channels() {
                let location = SampleLocation::new(channel, frame);
                let value = input_buffer.get_sample(&location);
                output_buffer.set_sample(&location, value * (gain as f32));
            }
        }
    }
}
