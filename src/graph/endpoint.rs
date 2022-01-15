use crate::commands::id::Id;

#[derive(Clone, PartialEq)]
pub struct Endpoint {
    pub dsp_id: Id,
}

impl Endpoint {
    pub fn new(node_id: Id) -> Self {
        Self { dsp_id: node_id }
    }
}
