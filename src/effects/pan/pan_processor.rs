use crate::{commands::Id, graph::DspProcessor, SampleLocation};

pub struct PanProcessor {
    pan_id: Id,
}

impl PanProcessor {
    pub fn new(pan_id: Id) -> Self {
        Self { pan_id }
    }
}

impl DspProcessor for PanProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        debug_assert_eq!(context.input_buffer.channel_count(), 2);
        debug_assert_eq!(context.output_buffer.channel_count(), 2);

        let pan_values = context
            .parameters
            .get_parameter_values(self.pan_id, context.output_buffer.frame_count());

        (0..context.output_buffer.frame_count()).for_each(|frame| {
            let pan = pan_values[frame];

            let l_gain = (1.0 - pan).min(1.0);
            let r_gain = (1.0 + pan).min(1.0);

            let l_location = SampleLocation::new(0, frame);
            let r_location = SampleLocation::new(1, frame);

            let l_input = context.input_buffer.get_sample(l_location);
            let r_input = context.input_buffer.get_sample(r_location);

            let l_value = l_input * l_gain;
            let r_value = r_input * r_gain;

            context.output_buffer.set_sample(l_location, l_value);
            context.output_buffer.set_sample(r_location, r_value);
        });
    }
}
