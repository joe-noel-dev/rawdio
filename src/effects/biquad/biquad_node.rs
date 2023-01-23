use std::collections::HashMap;

use crate::{commands::Id, AudioParameter, Context, GraphNode, Level};

use super::{biquad_processor::BiquadProcessor, filter_type::FilterType};

pub struct BiquadNode {
    pub node: GraphNode,
    pub frequency: AudioParameter,
    pub q: AudioParameter,
    pub shelf_gain: AudioParameter,
}

impl BiquadNode {
    pub fn new(context: &dyn Context, channel_count: usize, filter_type: FilterType) -> Self {
        let id = Id::generate();

        let command_queue = context.get_command_queue();

        let (frequency, realtime_frequency) =
            AudioParameter::new(id, 1_000.0, 20.0, 20000.0, command_queue.clone());

        let (q, realtime_q) =
            AudioParameter::new(id, 1.0 / 2.0_f64.sqrt(), 0.1, 10.0, command_queue.clone());

        let (shelf_gain, realtime_shelf_gain) = AudioParameter::new(
            id,
            Level::unity().as_gain(),
            0.0,
            Level::from_db(100.0).as_gain(),
            command_queue.clone(),
        );

        let processor = Box::new(BiquadProcessor::new(
            context.get_sample_rate(),
            channel_count,
            filter_type,
            frequency.get_id(),
            q.get_id(),
            shelf_gain.get_id(),
        ));

        let parameters = HashMap::from(
            [realtime_frequency, realtime_q, realtime_shelf_gain]
                .map(|parameter| (parameter.get_id(), parameter)),
        );

        let node = GraphNode::new(
            id,
            command_queue,
            channel_count,
            channel_count,
            processor,
            parameters,
        );

        Self {
            node,
            frequency,
            q,
            shelf_gain,
        }
    }
}
