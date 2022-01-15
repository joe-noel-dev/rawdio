use std::collections::HashMap;

use crate::commands::id::Id;

use super::{edge::Edge, node::Node};

pub type NodeMap<NodeData> = HashMap<Id, Node<NodeData>>;
pub type EdgeMap<EdgeData> = HashMap<Id, Edge<EdgeData>>;

pub enum Direction {
    Outgoing,
    Incoming,
}

pub fn get_last_connected_edge_id<NodeData, EdgeData>(
    node_id: Id,
    direction: Direction,
    nodes: &NodeMap<NodeData>,
    edges: &EdgeMap<EdgeData>,
) -> Option<Id> {
    if let Some(from_node) = nodes.get(&node_id) {
        let start_id = match direction {
            Direction::Outgoing => from_node.outgoing,
            Direction::Incoming => from_node.incoming,
        };

        if let Some(start_id) = start_id {
            return match EdgeIterator::new(start_id, edges).last() {
                Some(last_id) => Some(last_id),
                None => Some(start_id),
            };
        }
    }

    None
}

pub fn add_connection<NodeData, EdgeData>(
    from_node_id: Id,
    to_node_id: Id,
    with_edge_data: EdgeData,
    nodes: &mut NodeMap<NodeData>,
    edges: &mut EdgeMap<EdgeData>,
) -> Id {
    assert!(nodes.contains_key(&from_node_id));
    assert!(nodes.contains_key(&to_node_id));

    let edge_id = Id::generate();

    match get_last_connected_edge_id(from_node_id, Direction::Outgoing, nodes, edges) {
        Some(last_edge_id) => {
            if let Some(edge) = edges.get_mut(&last_edge_id) {
                edge.next = Some(edge_id);
            }
        }
        None => {
            if let Some(node) = nodes.get_mut(&from_node_id) {
                node.outgoing = Some(edge_id);
            }
        }
    }

    match get_last_connected_edge_id(to_node_id, Direction::Incoming, nodes, edges) {
        Some(last_edge_id) => {
            if let Some(edge) = edges.get_mut(&last_edge_id) {
                edge.next = Some(edge_id);
            }
        }
        None => {
            if let Some(node) = nodes.get_mut(&to_node_id) {
                node.incoming = Some(edge_id);
            }
        }
    }

    edges.insert(edge_id, Edge::new(from_node_id, to_node_id, with_edge_data));
    edge_id
}

pub struct EdgeIterator<'a, EdgeData> {
    edge_id: Option<Id>,
    edges: &'a EdgeMap<EdgeData>,
}

impl<'a, EdgeData> Iterator for EdgeIterator<'a, EdgeData> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(edge_id) = self.edge_id {
            if let Some(edge) = self.edges.get(&edge_id) {
                self.edge_id = edge.next;
                return edge.next;
            }
        }

        None
    }
}

impl<'a, EdgeData> EdgeIterator<'a, EdgeData> {
    pub fn new(edge_id: Id, edges: &'a EdgeMap<EdgeData>) -> Self {
        Self {
            edge_id: Some(edge_id),
            edges,
        }
    }
}

pub struct NodeIterator<'a, NodeData, EdgeData> {
    node_id: Id,
    edge_id: Option<Id>,
    nodes: &'a NodeMap<NodeData>,
    edges: &'a EdgeMap<EdgeData>,
    direction: Direction,
}

impl<'a, NodeData, EdgeData> Iterator for NodeIterator<'a, NodeData, EdgeData> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = if let Some(edge_id) = self.edge_id {
            if let Some(edge) = self.edges.get(&edge_id) {
                edge.next
            } else {
                None
            }
        } else if let Some(node) = self.nodes.get(&self.node_id) {
            match self.direction {
                Direction::Outgoing => node.outgoing,
                Direction::Incoming => node.incoming,
            }
        } else {
            None
        };

        self.edge_id = next_value;
        next_value
    }
}

impl<'a, NodeData, EdgeData> NodeIterator<'a, NodeData, EdgeData> {
    pub fn new(
        node_id: Id,
        direction: Direction,
        nodes: &'a NodeMap<NodeData>,
        edges: &'a EdgeMap<EdgeData>,
    ) -> Self {
        Self {
            node_id,
            edge_id: None,
            nodes,
            edges,
            direction,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iterate_edges() {
        let node_a = Node::new(());
        let node_b = Node::new(());
        let node_c = Node::new(());
        let node_d = Node::new(());

        let node_a_id = Id::generate();
        let node_b_id = Id::generate();
        let node_c_id = Id::generate();
        let node_d_id = Id::generate();

        let mut nodes = NodeMap::from([
            (node_a_id, node_a),
            (node_b_id, node_b),
            (node_c_id, node_c),
            (node_d_id, node_d),
        ]);

        let mut edges = EdgeMap::new();

        let a_to_b_id = add_connection(node_a_id, node_b_id, (), &mut nodes, &mut edges);
        let a_to_c_id = add_connection(node_a_id, node_c_id, (), &mut nodes, &mut edges);
        let a_to_d_id = add_connection(node_a_id, node_d_id, (), &mut nodes, &mut edges);

        let iterated_edges: Vec<Id> = EdgeIterator::new(a_to_b_id, &edges).collect();

        assert_eq!(iterated_edges.len(), 2);
        assert_eq!(iterated_edges[0], a_to_c_id);
        assert_eq!(iterated_edges[1], a_to_d_id);
    }

    #[test]
    fn iterate_outgoing_nodes() {
        let node_a = Node::new(());
        let node_b = Node::new(());
        let node_c = Node::new(());
        let node_d = Node::new(());

        let node_a_id = Id::generate();
        let node_b_id = Id::generate();
        let node_c_id = Id::generate();
        let node_d_id = Id::generate();

        let mut nodes = NodeMap::from([
            (node_a_id, node_a),
            (node_b_id, node_b),
            (node_c_id, node_c),
            (node_d_id, node_d),
        ]);

        let mut edges = EdgeMap::new();

        add_connection(node_a_id, node_b_id, (), &mut nodes, &mut edges);
        add_connection(node_a_id, node_c_id, (), &mut nodes, &mut edges);
        add_connection(node_a_id, node_d_id, (), &mut nodes, &mut edges);

        let connected_nodes: Vec<Id> =
            NodeIterator::new(node_a_id, Direction::Outgoing, &nodes, &edges).collect();

        assert_eq!(connected_nodes.len(), 3);
        assert!(connected_nodes.contains(&node_b_id));
        assert!(connected_nodes.contains(&node_c_id));
        assert!(connected_nodes.contains(&node_d_id));
    }
}
