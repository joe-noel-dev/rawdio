use crate::commands::Id;

use super::endpoint::{Endpoint, EndpointType};

#[derive(Clone, PartialEq, Eq)]
pub struct Connection {
    pub source: Endpoint,
    pub destination: Endpoint,
}

impl Connection {
    pub fn new(source_id: Id, destination_id: Id) -> Self {
        Self {
            source: Endpoint::new(source_id, EndpointType::Output),
            destination: Endpoint::new(destination_id, EndpointType::Input),
        }
    }
}
