use itertools::izip;

use crate::{
    commands::Id, graph::DspProcessor, AudioBuffer, Level, OwnedAudioBuffer, SampleLocation,
};

use super::{
    parameters::{OVERDRIVE_GAIN_DB_MAX, OVERDRIVE_GAIN_DB_MIN},
    shape::shape,
};

const MAX_BUFFER_SIZE: usize = 512;
const OVERSAMPLING_RATIO: usize = 2;

pub struct WaveshaperProcessor {
    transfer_function: Vec<f32>,
    overdrive_id: Id,
    mix_id: Id,
    oversampling_buffer: OwnedAudioBuffer,
}

impl WaveshaperProcessor {
    pub fn new(
        shaper: &dyn Fn(f32) -> f32,
        overdrive_id: Id,
        mix_id: Id,
        sample_rate: usize,
    ) -> Self {
        const NUM_POINTS: usize = 512 - 1;

        let transfer_function: Vec<f32> = (0..NUM_POINTS)
            .map(|index| map_index_to_input_value(index, NUM_POINTS))
            .map(shaper)
            .collect();

        Self {
            transfer_function,
            overdrive_id,
            mix_id,
            oversampling_buffer: OwnedAudioBuffer::new(
                MAX_BUFFER_SIZE * OVERSAMPLING_RATIO,
                1,
                sample_rate * OVERSAMPLING_RATIO,
            ),
        }
    }

    fn apply_shape(&mut self, output_buffer: &mut dyn AudioBuffer, channel: usize) {
        let location = SampleLocation::channel(channel);

        self.oversampling_buffer.sample_rate_convert_from(
            output_buffer,
            location,
            SampleLocation::origin(),
        );

        shape(
            self.oversampling_buffer
                .get_channel_data_mut(SampleLocation::origin()),
            &self.transfer_function,
        );

        output_buffer.sample_rate_convert_from(
            &self.oversampling_buffer,
            SampleLocation::origin(),
            location,
        );
    }
}

fn overdrive_to_gain(overdrive: f64) -> f64 {
    let gain_db =
        OVERDRIVE_GAIN_DB_MIN + overdrive * (OVERDRIVE_GAIN_DB_MAX - OVERDRIVE_GAIN_DB_MIN);
    Level::from_db(gain_db).as_gain()
}

fn apply_overdrive(overdrive: &[f64], samples: &mut [f32]) {
    samples
        .iter_mut()
        .zip(overdrive.iter())
        .for_each(|(sample, overdrive)| *sample *= overdrive_to_gain(*overdrive) as f32);
}

fn reverse_overdrive(overdrive: &[f64], samples: &mut [f32]) {
    samples
        .iter_mut()
        .zip(overdrive.iter())
        .for_each(|(sample, overdrive)| *sample /= overdrive_to_gain(*overdrive) as f32);
}

fn mix_input(input: &[f32], output: &mut [f32], mix: &[f64]) {
    izip!(input.iter(), output.iter_mut(), mix.iter()).for_each(
        |(&input_sample, output_sample, &mix_coefficient)| {
            *output_sample = *output_sample * mix_coefficient as f32
                + input_sample * (1.0 - mix_coefficient as f32);
        },
    );
}

impl DspProcessor for WaveshaperProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn crate::AudioBuffer,
        output_buffer: &mut dyn crate::AudioBuffer,
        _start_time: &crate::Timestamp,
        parameters: &crate::graph::DspParameters,
    ) {
        let overdrive = parameters.get_parameter_values(self.overdrive_id);
        let mix = parameters.get_parameter_values(self.mix_id);

        (0..output_buffer.channel_count()).for_each(|channel| {
            let location = SampleLocation::channel(channel);
            let input_data = input_buffer.get_channel_data(location);

            {
                let output_data = output_buffer.get_channel_data_mut(location);
                output_data.copy_from_slice(input_data);
                apply_overdrive(overdrive, output_data);
            }

            self.apply_shape(output_buffer, channel);

            {
                let output_data = output_buffer.get_channel_data_mut(location);
                reverse_overdrive(overdrive, output_data);
                mix_input(input_data, output_data, mix);
            }
        });
    }
}

fn map_index_to_input_value(index: usize, element_count: usize) -> f32 {
    let normalised = index as f32 / (element_count as f32 - 1.0);
    const MAX_VALUE: f32 = 1.0;
    const MIN_VALUE: f32 = -1.0;
    MIN_VALUE + normalised * (MAX_VALUE - MIN_VALUE)
}
