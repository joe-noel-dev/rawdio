use std::collections::HashMap;

use crate::{commands::Id, AudioParameter, Context, Level, Node};

use super::{biquad_processor::BiquadProcessor, filter_type::FilterType};

pub struct BiquadNode {
    pub node: Node,
    pub output_gain: AudioParameter,
    pub frequency: AudioParameter,
    pub q: AudioParameter,
    pub shelf_gain: AudioParameter,
}

impl BiquadNode {
    pub fn new(context: &dyn Context, channel_count: usize, filter_type: FilterType) -> Self {
        let id = Id::generate();

        let command_queue = context.get_command_queue();

        let (output_gain, realtime_output_gain) =
            AudioParameter::new(id, 0.0, 0.0, 1.0, command_queue.clone());

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
            output_gain.get_id(),
            frequency.get_id(),
            q.get_id(),
            shelf_gain.get_id(),
        ));

        let parameters = HashMap::from(
            [
                realtime_output_gain,
                realtime_frequency,
                realtime_q,
                realtime_shelf_gain,
            ]
            .map(|parameter| (parameter.get_id(), parameter)),
        );

        let node = Node::new(
            id,
            command_queue,
            channel_count,
            channel_count,
            processor,
            parameters,
        );

        Self {
            node,
            output_gain,
            frequency,
            q,
            shelf_gain,
        }
    }
}
