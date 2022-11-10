use crate::commands::Id;

pub struct Edge<EdgeData> {
    pub from_node_id: Id,
    pub to_node_id: Id,
    pub edge_data: EdgeData,
    pub next_out: Option<Id>,
    pub next_in: Option<Id>,
}

impl<EdgeData> Edge<EdgeData> {
    pub fn new(from_node_id: Id, to_node_id: Id, edge_data: EdgeData) -> Self {
        Self {
            from_node_id,
            to_node_id,
            edge_data,
            next_out: None,
            next_in: None,
        }
    }
}
