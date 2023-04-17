use crate::{commands::Id, graph::DspParameters, AudioParameter, Context, GraphNode, Level};

use super::{
    parameters::{
        MIX_PARAMETER_DEFAULT, MIX_PARAMETER_MAX, MIX_PARAMETER_MIN, OVERDRIVE_PARAMETER_DEFAULT,
        OVERDRIVE_PARAMETER_MAX, OVERDRIVE_PARAMETER_MIN,
    },
    waveshaper_processor::WaveshaperProcessor,
};

/// A node that will distort the input signal using a specified function
pub struct Waveshaper {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    /// The amount of 'drive' to apply to the signal
    ///
    /// This is a normalised value (0.0 - 1.0) that will apply gain to the
    /// signal
    pub overdrive: AudioParameter,

    /// The proportion of wet/processed signal to send to the output
    ///
    /// 1.0 represents a fully processed signal
    /// 0.0 represents a full dry signal
    pub mix: AudioParameter,
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
        let threshold = threshold.as_gain() as f32;
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
        let threshold = threshold.as_gain() as f32;

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

        let (overdrive, realtime_overdrive) = AudioParameter::new(
            id,
            OVERDRIVE_PARAMETER_DEFAULT,
            OVERDRIVE_PARAMETER_MIN,
            OVERDRIVE_PARAMETER_MAX,
            context.get_command_queue(),
        );

        let (mix, realtime_mix) = AudioParameter::new(
            id,
            MIX_PARAMETER_DEFAULT,
            MIX_PARAMETER_MIN,
            MIX_PARAMETER_MAX,
            context.get_command_queue(),
        );

        let processor = Box::new(WaveshaperProcessor::new(
            shaper,
            overdrive.get_id(),
            mix.get_id(),
            context.get_sample_rate(),
        ));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            channel_count,
            channel_count,
            processor,
            DspParameters::new([realtime_overdrive, realtime_mix]),
        );

        Self {
            node,
            overdrive,
            mix,
        }
    }
}
