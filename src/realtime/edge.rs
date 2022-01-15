use crate::commands::id::Id;

pub struct Edge<EdgeData> {
    pub from_node_id: Id,
    pub to_node_id: Id,
    pub edge_data: EdgeData,
    pub next: Option<Id>,
}

impl<EdgeData> Edge<EdgeData> {
    pub fn new(from_node_id: Id, to_node_id: Id, edge_data: EdgeData) -> Self {
        Self {
            from_node_id,
            to_node_id,
            edge_data,
            next: None,
        }
    }
}
