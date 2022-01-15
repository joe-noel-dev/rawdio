use crate::commands::id::Id;

pub struct Node<NodeData> {
    pub node_data: NodeData,
    pub outgoing: Option<Id>,
    pub incoming: Option<Id>,
}

impl<NodeData> Node<NodeData> {
    pub fn new(node_data: NodeData) -> Self {
        Self {
            node_data,
            outgoing: None,
            incoming: None,
        }
    }
}
