use crate::{
    commands::Id,
    graph::{DspParameters, DspProcessor},
    utility::macros::unwrap_or_return,
    AudioBuffer, SampleLocation, Timestamp,
};

pub struct PanProcessor {
    pan_id: Id,
}

impl PanProcessor {
    pub fn new(pan_id: Id) -> Self {
        Self { pan_id }
    }
}

impl DspProcessor for PanProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        _start_time: &Timestamp,
        parameters: &DspParameters,
    ) {
        debug_assert_eq!(input_buffer.channel_count(), 2);
        debug_assert_eq!(output_buffer.channel_count(), 2);

        let pan_values = unwrap_or_return!(parameters.get(&self.pan_id)).get_values();

        (0..output_buffer.frame_count()).for_each(|frame| {
            let pan = pan_values[frame];

            let l_gain = (1.0 - pan).min(1.0);
            let r_gain = (1.0 + pan).min(1.0);

            let l_location = SampleLocation::new(0, frame);
            let r_location = SampleLocation::new(1, frame);

            let l_input = input_buffer.get_sample(l_location);
            let r_input = input_buffer.get_sample(r_location);

            let l_value = l_input * l_gain as f32;
            let r_value = r_input * r_gain as f32;

            output_buffer.set_sample(l_location, l_value);
            output_buffer.set_sample(r_location, r_value);
        });
    }
}
