use std::collections::HashMap;

use crate::{
    commands::Id, graph::DspParameters, parameter::RealtimeAudioParameter, AudioParameter, Context,
    GraphNode,
};

use super::{
    compressor_parameters::{get_range, CompressorParameter},
    compressor_processor::CompressorProcessor,
};

/// A basic dynamics compressor
pub struct Compressor {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    /// The attack time of the compressor (in ms)
    ///
    /// This is how fast or slow the compressor tracks the input signal when the signal
    /// level is rising
    pub attack: AudioParameter,

    /// The release time of the compressor (in ms)
    ///
    /// This is how fast or slow the compressor releases the tracked input signal
    /// when the signal level is falling
    pub release: AudioParameter,

    /// The ratio of the compressor
    ///
    /// This controls how aggresively the signal is attentuated when it goes
    /// over the threshold
    pub ratio: AudioParameter,

    /// The threshold at which the signal will begin to apply gain reduction
    pub threshold: AudioParameter,

    /// The knee of the compressor (in dB)
    ///
    /// At larger values, the compressor's gain reduction will begin more gradually
    pub knee: AudioParameter,

    /// The gain to apply to the wet (compressed) signal (linear gain)
    pub wet_level: AudioParameter,

    /// The gain to apply to the dry (uncompressed) signal
    pub dry_level: AudioParameter,
}

impl Compressor {
    /// Create a new compressor node
    pub fn new(context: &dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();

        let parameters = [
            CompressorParameter::Attack,
            CompressorParameter::Release,
            CompressorParameter::Ratio,
            CompressorParameter::Threshold,
            CompressorParameter::Knee,
            CompressorParameter::WetLevel,
            CompressorParameter::DryLevel,
        ];

        let mut audio_parameters = HashMap::new();
        let mut realtime_paramters = HashMap::new();

        for param in parameters.iter() {
            let (audio_param, realtime_param) = AudioParameter::new(id, get_range(*param), context);
            audio_parameters.insert(*param, audio_param);
            realtime_paramters.insert(*param, realtime_param);
        }

        let mut parameter_ids = HashMap::new();
        for (parameter, audio_param) in audio_parameters.iter() {
            parameter_ids.insert(*parameter, audio_param.get_id());
        }

        let processor = Box::new(CompressorProcessor::new(
            channel_count,
            context.get_sample_rate(),
            context.maximum_frame_count(),
            parameter_ids,
        ));

        let dsp_parameters = DspParameters::from(
            realtime_paramters
                .into_values()
                .collect::<Vec<RealtimeAudioParameter>>(),
        );

        let node = GraphNode::new(
            id,
            context,
            channel_count,
            channel_count,
            processor,
            dsp_parameters,
        );

        Self {
            node,
            attack: audio_parameters
                .remove(&CompressorParameter::Attack)
                .unwrap_or_else(|| panic!("Missing parameter")),
            release: audio_parameters
                .remove(&CompressorParameter::Release)
                .unwrap_or_else(|| panic!("Missing parameter")),
            ratio: audio_parameters
                .remove(&CompressorParameter::Ratio)
                .unwrap_or_else(|| panic!("Missing parameter")),
            threshold: audio_parameters
                .remove(&CompressorParameter::Threshold)
                .unwrap_or_else(|| panic!("Missing parameter")),
            knee: audio_parameters
                .remove(&CompressorParameter::Knee)
                .unwrap_or_else(|| panic!("Missing parameter")),
            wet_level: audio_parameters
                .remove(&CompressorParameter::WetLevel)
                .unwrap_or_else(|| panic!("Missing parameter")),
            dry_level: audio_parameters
                .remove(&CompressorParameter::DryLevel)
                .unwrap_or_else(|| panic!("Missing parameter")),
        }
    }
}
