use crate::commands::Id;

use super::endpoint::{Endpoint, EndpointType};

#[derive(Clone, PartialEq, Eq)]
pub struct Connection {
    pub source: Endpoint,
    pub source_output_channel: usize,
    pub destination: Endpoint,
    pub destination_input_channel: usize,
    pub channel_count: usize,
}

impl Connection {
    pub fn new(source_id: Id, destination_id: Id, channel_count: usize) -> Self {
        Self {
            source: Endpoint::new(source_id, EndpointType::Output),
            destination: Endpoint::new(destination_id, EndpointType::Input),
            source_output_channel: 0,
            destination_input_channel: 0,
            channel_count,
        }
    }

    pub fn with_source_output_channel(mut self, channel: usize) -> Self {
        self.source_output_channel = channel;
        self
    }

    pub fn with_destination_input_channel(mut self, channel: usize) -> Self {
        self.destination_input_channel = channel;
        self
    }
}
