use crate::commands::id::Id;

use super::endpoint::Endpoint;

#[derive(Clone, PartialEq)]
pub struct Connection {
    pub source: Endpoint,
    pub destination: Endpoint,
}

impl Connection {


    pub fn new(source_id: Id, destination_id: Id) -> Self {
        Self {
            source: Endpoint::new(source_id),
            destination: Endpoint::new(destination_id),
        }
    }

    
}
