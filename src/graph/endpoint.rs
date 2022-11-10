use crate::commands::Id;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointType {
    Input,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub endpoint_type: EndpointType,
    pub dsp_id: Id,
}

impl Endpoint {
    pub fn new(node_id: Id, endpoint_type: EndpointType) -> Self {
        Self {
            dsp_id: node_id,
            endpoint_type,
        }
    }
}
