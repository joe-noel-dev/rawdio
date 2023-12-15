use crate::{
    commands::Id,
    parameter::{ParameterRange, Parameters},
    utility::create_parameters,
    AudioParameter, Context, DspNode, GraphNode, Level,
};

use super::{
    parameters::{
        MIX_PARAMETER_DEFAULT, MIX_PARAMETER_MAX, MIX_PARAMETER_MIN, OVERDRIVE_PARAMETER_DEFAULT,
        OVERDRIVE_PARAMETER_MAX, OVERDRIVE_PARAMETER_MIN,
    },
    waveshaper_processor::WaveshaperProcessor,
};

/// A node that will distort the input signal using a specified function
///
/// # Parameters
/// - overdrive
/// - mix
pub struct Waveshaper {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    params: Parameters,
}

impl DspNode for Waveshaper {
    fn get_parameters(&self) -> &Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut Parameters {
        &mut self.params
    }
}

impl Waveshaper {
    /// Create a waveshaper that uses the mathematical tanh function to shape the
    /// signal
    pub fn tanh(context: &dyn Context, channel_count: usize) -> Self {
        let shaper = |input: f32| {
            const CONSTANT: f32 = 2.0;
            CONSTANT * (input / CONSTANT).tanh()
        };

        Self::new(context, channel_count, &shaper)
    }

    /// Create a waveshaper that uses a soft saturation function to shape the
    /// input
    pub fn soft_saturator(context: &dyn Context, channel_count: usize, threshold: Level) -> Self {
        let threshold = threshold.as_linear() as f32;
        assert!(0.0 <= threshold);
        assert!(threshold <= 1.0);

        let shaper = move |input: f32| {
            if input < threshold {
                input
            } else {
                threshold
                    + (input - threshold)
                        / (1.0 + ((input - threshold) / (1.0 - threshold)).powf(2.0))
            }
        };

        Self::new(context, channel_count, &shaper)
    }

    /// Create a waveshaper that hard clips the signal when it goes over the
    /// specified threshold
    pub fn hard_clip(context: &dyn Context, channel_count: usize, threshold: Level) -> Self {
        let threshold = threshold.as_linear() as f32;

        let shaper = Box::new(move |input: f32| {
            if input < -threshold {
                return -threshold;
            }

            if input > threshold {
                return threshold;
            }

            input
        });

        Self::new(context, channel_count, &shaper)
    }

    /// Create a new waveshaper using a custom shaper function
    pub fn new(context: &dyn Context, channel_count: usize, shaper: &dyn Fn(f32) -> f32) -> Self {
        let id = Id::generate();

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [
                (
                    "overdrive",
                    ParameterRange::new(
                        OVERDRIVE_PARAMETER_DEFAULT,
                        OVERDRIVE_PARAMETER_MIN,
                        OVERDRIVE_PARAMETER_MAX,
                    ),
                ),
                (
                    "mix",
                    ParameterRange::new(
                        MIX_PARAMETER_DEFAULT,
                        MIX_PARAMETER_MIN,
                        MIX_PARAMETER_MAX,
                    ),
                ),
            ],
        );

        let processor = Box::new(WaveshaperProcessor::new(
            shaper,
            params.get("overdrive").unwrap().get_id(),
            params.get("mix").unwrap().get_id(),
            context.get_sample_rate(),
            context.maximum_frame_count(),
        ));

        let node = GraphNode::new(
            id,
            context,
            channel_count,
            channel_count,
            processor,
            realtime_params,
        );

        Self { node, params }
    }

    /// Get the overdrive parameter
    pub fn overdrive(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("overdrive")
    }
    /// Get the mix parameter
    pub fn mix(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("mix")
    }
}
