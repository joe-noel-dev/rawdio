use crate::{commands::Id, graph::DspParameters, AudioParameter, Context, GraphNode, Level};

use super::{
    parameters::{
        MIX_PARAMETER_DEFAULT, MIX_PARAMETER_MAX, MIX_PARAMETER_MIN, OVERDRIVE_PARAMETER_DEFAULT,
        OVERDRIVE_PARAMETER_MAX, OVERDRIVE_PARAMETER_MIN,
    },
    waveshaper_processor::WaveshaperProcessor,
};

pub struct Waveshaper {
    pub node: GraphNode,
    pub overdrive: AudioParameter,
    pub mix: AudioParameter,
}

impl Waveshaper {
    pub fn tanh(context: &dyn Context, channel_count: usize) -> Self {
        const NUM_POINTS: usize = 511;

        let constant = 2.0;

        let shape: Vec<f32> = (0..NUM_POINTS)
            .map(|index| map_index_to_sample_range(index, NUM_POINTS))
            .map(|x_value| (constant * (x_value / constant).tanh()) as f32)
            .collect();

        Self::new(context, channel_count, &shape)
    }

    pub fn hard_clip(context: &dyn Context, channel_count: usize, threshold: Level) -> Self {
        const NUM_POINTS: usize = 511;
        let threshold = threshold.as_gain();

        let shape: Vec<f32> = (0..NUM_POINTS)
            .map(|index| map_index_to_sample_range(index, NUM_POINTS))
            .map(|x_value| {
                if x_value < -threshold {
                    return -threshold;
                }

                if x_value > threshold {
                    return threshold;
                }

                x_value
            })
            .map(|value| value as f32)
            .collect();

        Self::new(context, channel_count, &shape)
    }

    pub fn new(context: &dyn Context, channel_count: usize, wave_shape: &[f32]) -> Self {
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
            Vec::from(wave_shape),
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

fn map_index_to_sample_range(index: usize, element_count: usize) -> f64 {
    let normalised = index as f64 / (element_count as f64 - 1.0);
    const MAX_VALUE: f64 = 1.0;
    const MIN_VALUE: f64 = -1.0;
    MIN_VALUE + normalised * (MAX_VALUE - MIN_VALUE)
}
