use crate::{
    commands::Id,
    graph::{DspParameters, GraphNode},
    parameter::{AudioParameter, ParameterRange},
    Context, Level,
};

use super::oscillator_processor::OscillatorProcessor;

/// An oscillator node
///
/// Oscillator nodes don't have inputs, they only produce output
pub struct Oscillator {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    /// The frequency of the oscillator
    ///
    /// This should probably be between 20 Hz and 20 kHz
    pub frequency: AudioParameter,

    /// The (linear) gain of the oscillator
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
    /// Create a sine wave oscillator at the given frequency
    pub fn sine(context: &dyn Context, frequency: f64, output_count: usize) -> Self {
        let table_size = 8192;
        let sine_wavetable = make_sine_wavetable(table_size, 1);
        Self::new(context, frequency, output_count, sine_wavetable)
    }

    /// Create an oscillator that is made up of various sine waves
    ///
    /// Each entry in the `harmonics` array represents a harmonic in the output
    /// Index 0 represents the level of the fundamental, index 1 represents
    /// the level of the first harmonic, and so on
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
                *wavetable_sample += *harmonic_sample * level.as_linear();
            }
        }

        Self::new(context, frequency, output_count, wavetable)
    }

    /// Create a new oscillator with a custom wavetable at a given frequency
    ///
    /// When playing, it will play at a rate such that it plays through
    /// wavetable every `1.0 / frequency` seconds. If there isn't a sample
    /// at the desired index, it will linearly interpolate between the two
    /// nearest samples
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
            ParameterRange::new(frequency, MIN_FREQUENCY, MAX_FREQUENCY),
            context,
        );

        let (gain, realtime_gain) = AudioParameter::new(
            id,
            ParameterRange::new(DEFAULT_GAIN, MIN_GAIN, MAX_GAIN),
            context,
        );

        let input_count = 0;

        let processor = Box::new(OscillatorProcessor::new(
            frequency.get_id(),
            gain.get_id(),
            wavetable,
        ));

        let node = GraphNode::new(
            id,
            context,
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
