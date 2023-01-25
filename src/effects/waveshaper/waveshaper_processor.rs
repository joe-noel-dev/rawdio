use crate::{commands::Id, graph::DspProcessor, utility::macros::unwrap_or_return};

pub struct WaveshaperProcessor {
    shape: Vec<f64>,
    overdrive_id: Id,
    mix_id: Id,
}

impl WaveshaperProcessor {
    pub fn new(shape: Vec<f64>, overdrive_id: Id, mix_id: Id) -> Self {
        debug_assert!(shape.is_sorted());

        Self {
            shape,
            overdrive_id,
            mix_id,
        }
    }
}

impl DspProcessor for WaveshaperProcessor {
    fn process_audio(
        &mut self,
        _input_buffer: &dyn crate::AudioBuffer,
        _output_buffer: &mut dyn crate::AudioBuffer,
        _start_time: &crate::Timestamp,
        parameters: &crate::graph::DspParameters,
    ) {
        let overdrive = unwrap_or_return!(parameters.get(&self.overdrive_id)).get_values();
        let mix = unwrap_or_return!(parameters.get(&self.mix_id)).get_values();

        todo!()
    }
}
