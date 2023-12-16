use crate::{
    commands::Id, graph::DspNode, parameter::Parameters, prelude::*, utility::create_parameters,
};

use super::{compressor_parameters::get_range, compressor_processor::CompressorProcessor};

/// A basic dynamics compressor
///
/// # Parameters
/// - attack
/// - release
/// - ratio
/// - threshold
/// - knee
/// - wet
/// - dry
pub struct Compressor {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    params: Parameters,
}

impl DspNode for Compressor {
    fn get_parameters(&self) -> &Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut Parameters {
        &mut self.params
    }
}

impl Compressor {
    /// Create a new compressor node
    pub fn new(context: &dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();

        let param_ids = [
            "attack",
            "release",
            "ratio",
            "threshold",
            "knee",
            "wet",
            "dry",
        ];

        let (params, realtime_params) =
            create_parameters(id, context, param_ids.map(|id| (id, get_range(id))));

        let processor = Box::new(CompressorProcessor::new(
            channel_count,
            context.get_sample_rate(),
            context.maximum_frame_count(),
        ));

        Self {
            node: GraphNode::new(
                id,
                context,
                channel_count,
                channel_count,
                processor,
                realtime_params,
            ),
            params,
        }
    }

    /// Get the attack parameter
    pub fn attack(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("attack")
    }
    /// Get the release parameter
    pub fn release(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("release")
    }
    /// Get the ratio parameter
    pub fn ratio(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("ratio")
    }
    /// Get the threshold parameter
    pub fn threshold(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("threshold")
    }
    /// Get the knee parameter
    pub fn knee(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("knee")
    }
    /// Get the wet parameter
    pub fn wet(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("wet")
    }
    /// Get the dry parameter
    pub fn dry(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("dry")
    }
}
