use std::collections::HashMap;

use crate::{commands::Id, graph::GraphNode, parameter::AudioParameter, CommandQueue};

use super::oscillator_processor::OscillatorProcessor;

pub struct Oscillator {
    pub node: GraphNode,
    pub frequency: AudioParameter,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const MIN_FREQUENCY: f64 = 20.0;
const MAX_FREQUENCY: f64 = 20000.0;
const DEFAULT_GAIN: f64 = 1.0;

impl Oscillator {
    pub fn new(command_queue: CommandQueue, frequency: f64, output_count: usize) -> Self {
        debug_assert!(output_count > 0);

        let id = Id::generate();

        let (frequency, realtime_frequency) = AudioParameter::new(
            id,
            frequency,
            MIN_FREQUENCY,
            MAX_FREQUENCY,
            command_queue.clone(),
        );

        let (gain, realtime_gain) =
            AudioParameter::new(id, DEFAULT_GAIN, MIN_GAIN, MAX_GAIN, command_queue.clone());

        let parameters = HashMap::from(
            [realtime_frequency, realtime_gain].map(|parameter| (parameter.get_id(), parameter)),
        );

        let input_count = 0;

        let processor = Box::new(OscillatorProcessor::new(frequency.get_id(), gain.get_id()));

        let node = GraphNode::new(
            id,
            command_queue,
            input_count,
            output_count,
            processor,
            parameters,
        );

        Self {
            node,
            frequency,
            gain,
        }
    }
}
