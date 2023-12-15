use itertools::izip;

use crate::{
    commands::Id,
    graph::GraphNode,
    parameter::{ParameterRange, Parameters},
    utility::create_parameters,
    AudioParameter, Context, DspNode, Level,
};

use super::oscillator_processor::OscillatorProcessor;

/// An oscillator node
///
/// Oscillator nodes don't have inputs, they only produce output
///
/// # Parameters
///
/// - frequency
/// - gain
pub struct Oscillator {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    params: Parameters,
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

impl DspNode for Oscillator {
    fn get_parameters(&self) -> &Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut Parameters {
        &mut self.params
    }
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
                izip!(harmonic_wavetable.iter(), wavetable.iter_mut())
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

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [
                (
                    "frequency",
                    ParameterRange::new(frequency, MIN_FREQUENCY, MAX_FREQUENCY),
                ),
                (
                    "gain",
                    ParameterRange::new(DEFAULT_GAIN, MIN_GAIN, MAX_GAIN),
                ),
            ],
        );

        let input_count = 0;

        let processor = Box::new(OscillatorProcessor::new(
            params.get("frequency").unwrap().get_id(),
            params.get("gain").unwrap().get_id(),
            wavetable,
        ));

        let node = GraphNode::new(
            id,
            context,
            input_count,
            output_count,
            processor,
            realtime_params,
        );

        Self { node, params }
    }

    /// Get the frequency parameter
    pub fn frequency(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("frequency")
    }

    /// Get the gain parameter
    pub fn gain(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("gain")
    }
}
