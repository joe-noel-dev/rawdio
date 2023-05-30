use std::collections::HashMap;

use crate::{commands::Id, graph::DspParameters, AudioParameter, Context, GraphNode};

use super::{
    compressor_parameters::{
        attack_range, dry_range, knee_range, makeup_range, ratio_range, release_range,
        threshold_range, wet_range, CompressorParameter,
    },
    compressor_processor::CompressorProcessor,
};

pub struct Compressor {
    node: GraphNode,
    pub attack: AudioParameter,
    pub release: AudioParameter,
    pub ratio: AudioParameter,
    pub threshold: AudioParameter,
    pub knee: AudioParameter,
    pub makeup: AudioParameter,
    pub wet_level: AudioParameter,
    pub dry_level: AudioParameter,
}

impl Compressor {
    pub fn new(context: &mut dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();

        let (attack, realtime_attack) = AudioParameter::new(id, attack_range(), context);
        let (release, realtime_release) = AudioParameter::new(id, release_range(), context);
        let (ratio, realtime_ratio) = AudioParameter::new(id, ratio_range(), context);
        let (threshold, realtime_threshold) = AudioParameter::new(id, threshold_range(), context);
        let (knee, realtime_knee) = AudioParameter::new(id, knee_range(), context);
        let (makeup, realtime_makeup) = AudioParameter::new(id, makeup_range(), context);
        let (wet_level, realtime_wet_level) = AudioParameter::new(id, wet_range(), context);
        let (dry_level, realtime_dry_level) = AudioParameter::new(id, dry_range(), context);

        let parameter_ids = HashMap::from([
            (CompressorParameter::Attack, attack.get_id()),
            (CompressorParameter::Release, release.get_id()),
            (CompressorParameter::Ratio, ratio.get_id()),
            (CompressorParameter::Threshold, threshold.get_id()),
            (CompressorParameter::Knee, knee.get_id()),
            (CompressorParameter::Makeup, makeup.get_id()),
            (CompressorParameter::WetLevel, wet_level.get_id()),
            (CompressorParameter::DryLevel, dry_level.get_id()),
        ]);

        let processor = Box::new(CompressorProcessor::new(
            channel_count,
            context.get_sample_rate(),
            context.maximum_frame_count(),
            parameter_ids,
        ));

        let parameters = DspParameters::new([
            realtime_attack,
            realtime_release,
            realtime_ratio,
            realtime_threshold,
            realtime_knee,
            realtime_makeup,
            realtime_wet_level,
            realtime_dry_level,
        ]);

        let node = GraphNode::new(
            id,
            context,
            channel_count,
            channel_count,
            processor,
            parameters,
        );

        Self {
            node,
            attack,
            release,
            ratio,
            threshold,
            knee,
            makeup,
            wet_level,
            dry_level,
        }
    }
}
