use crate::commands::id::Id;

#[derive(Clone, PartialEq)]
pub struct Endpoint {
    pub node_id: Id,
}

impl Endpoint {
    pub fn new(node_id: Id) -> Self {
        Self { node_id }
    }
}
