use crate::{
    commands::Id,
    graph::{DspParameters, GraphNode},
    parameter::AudioParameter,
    Context, Level,
};

use super::oscillator_processor::OscillatorProcessor;

pub struct Oscillator {
    pub node: GraphNode,
    pub frequency: AudioParameter,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const MIN_FREQUENCY: f64 = 20.0;
const MAX_FREQUENCY: f64 = 20000.0;
const DEFAULT_GAIN: f64 = 1.0;

fn make_sine_wavetable(length: usize, harmonic: usize) -> Vec<f64> {
    let mut values = Vec::with_capacity(length);

    for frame in 0..length {
        let time = harmonic as f64 * frame as f64 / length as f64;
        let value = (std::f64::consts::TAU * time).sin();
        values.push(value);
    }

    values
}

impl Oscillator {
    pub fn sine(context: &dyn Context, frequency: f64, output_count: usize) -> Self {
        let table_size = 8192;
        let sine_wavetable = make_sine_wavetable(table_size, 1);
        Self::new(context, frequency, output_count, sine_wavetable)
    }

    pub fn with_harmonics(
        context: &dyn Context,
        frequency: f64,
        output_count: usize,
        harmonics: &[Level],
    ) -> Self {
        let table_size = 8192;
        let mut wavetable = Vec::new();
        wavetable.resize(table_size, 0.0);

        for (index, level) in harmonics.iter().enumerate() {
            let harmonic = index + 1;
            let harmonic_wavetable = make_sine_wavetable(table_size, harmonic);

            for (harmonic_sample, wavetable_sample) in
                harmonic_wavetable.iter().zip(wavetable.iter_mut())
            {
                *wavetable_sample += *harmonic_sample * level.as_gain();
            }
        }

        Self::new(context, frequency, output_count, wavetable)
    }

    pub fn new(
        context: &dyn Context,
        frequency: f64,
        output_count: usize,
        wavetable: Vec<f64>,
    ) -> Self {
        debug_assert!(output_count > 0);
        debug_assert!(wavetable.len() > 2);

        let id = Id::generate();

        let (frequency, realtime_frequency) = AudioParameter::new(
            id,
            frequency,
            MIN_FREQUENCY,
            MAX_FREQUENCY,
            context.get_command_queue(),
        );

        let (gain, realtime_gain) = AudioParameter::new(
            id,
            DEFAULT_GAIN,
            MIN_GAIN,
            MAX_GAIN,
            context.get_command_queue(),
        );

        let input_count = 0;

        let processor = Box::new(OscillatorProcessor::new(
            frequency.get_id(),
            gain.get_id(),
            wavetable,
        ));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            input_count,
            output_count,
            processor,
            DspParameters::new([realtime_frequency, realtime_gain]),
        );

        Self {
            node,
            frequency,
            gain,
        }
    }
}
