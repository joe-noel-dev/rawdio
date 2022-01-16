use std::collections::{hash_map::Keys, HashMap};

use crate::commands::id::Id;

use super::{edge::Edge, node::Node};

pub type NodeMap<NodeData> = HashMap<Id, Node<NodeData>>;
pub type EdgeMap<EdgeData> = HashMap<Id, Edge<EdgeData>>;

#[derive(Clone, Copy)]
pub enum Direction {
    Outgoing,
    Incoming,
}

pub struct Graph<NodeData, EdgeData> {
    nodes: NodeMap<NodeData>,
    edges: EdgeMap<EdgeData>,
}

impl<NodeData, EdgeData> Graph<NodeData, EdgeData> {
    pub fn with_capacity(number_of_nodes: usize, number_of_edges: usize) -> Self {
        Self {
            nodes: NodeMap::with_capacity(number_of_nodes),
            edges: EdgeMap::with_capacity(number_of_edges),
        }
    }

    pub fn get_last_connected_edge_id(&self, node_id: Id, direction: Direction) -> Option<Id> {
        if let Some(node) = self.nodes.get(&node_id) {
            let start_id = match direction {
                Direction::Outgoing => node.outgoing,
                Direction::Incoming => node.incoming,
            };

            return match start_id {
                Some(start_id) => {
                    match EdgeIterator::new(start_id, direction, &self.edges).last() {
                        Some(last_id) => Some(last_id),
                        None => Some(start_id),
                    }
                }
                None => None,
            };
        }

        None
    }

    pub fn add_edge(&mut self, from_node_id: Id, to_node_id: Id, with_edge_data: EdgeData) -> Id {
        assert!(self.nodes.contains_key(&from_node_id));
        assert!(self.nodes.contains_key(&to_node_id));

        let edge_id = Id::generate();

        match self.get_last_connected_edge_id(from_node_id, Direction::Outgoing) {
            Some(last_edge_id) => {
                if let Some(edge) = self.edges.get_mut(&last_edge_id) {
                    edge.next_out = Some(edge_id);
                }
            }
            None => {
                if let Some(node) = self.nodes.get_mut(&from_node_id) {
                    node.outgoing = Some(edge_id);
                }
            }
        }

        match self.get_last_connected_edge_id(to_node_id, Direction::Incoming) {
            Some(last_edge_id) => {
                if let Some(edge) = self.edges.get_mut(&last_edge_id) {
                    edge.next_in = Some(edge_id);
                }
            }
            None => {
                if let Some(node) = self.nodes.get_mut(&to_node_id) {
                    node.incoming = Some(edge_id);
                }
            }
        }

        self.edges
            .insert(edge_id, Edge::new(from_node_id, to_node_id, with_edge_data));
        edge_id
    }

    pub fn remove_edge(&mut self, from_node_id: Id, to_node_id: Id) {
        // TODO: Fixup connections

        let id = self
            .edges
            .iter()
            .find(|(_, edge)| edge.from_node_id == from_node_id && edge.to_node_id == to_node_id)
            .map(|(id, _)| *id);

        if let Some(id) = id {
            self.edges.remove(&id);
        }
    }

    pub fn _add_node(&mut self, node_data: NodeData) -> Id {
        let id = Id::generate();
        self.add_node_with_id(id, node_data);
        id
    }

    pub fn remove_node(&mut self, id: Id) -> Option<NodeData> {
        // TODO: Remove associated connections
        self.nodes.remove(&id).map(|node| node.node_data)
    }

    pub fn get_node_mut(&mut self, id: Id) -> Option<&mut NodeData> {
        self.nodes.get_mut(&id).map(|node| &mut node.node_data)
    }

    pub fn get_node(&self, id: Id) -> Option<&NodeData> {
        self.nodes.get(&id).map(|node| &node.node_data)
    }

    pub fn add_node_with_id(&mut self, id: Id, node_data: NodeData) {
        assert!(!self.nodes.contains_key(&id));
        self.nodes.insert(id, Node::new(node_data));
    }

    pub fn is_connected_to(&self, from_node_id: Id, to_node_id: Id) -> bool {
        NodeIterator::new(from_node_id, Direction::Outgoing, &self.nodes, &self.edges)
            .any(|id| id == to_node_id)
    }

    fn _edge_iter(&self, edge_id: Id, direction: Direction) -> EdgeIterator<EdgeData> {
        EdgeIterator::new(edge_id, direction, &self.edges)
    }

    pub fn node_iter(&self, node_id: Id, direction: Direction) -> NodeIterator<NodeData, EdgeData> {
        NodeIterator::new(node_id, direction, &self.nodes, &self.edges)
    }

    pub fn num_connections(&self, node_id: Id, direction: Direction) -> usize {
        self.node_iter(node_id, direction).count()
    }

    pub fn all_node_ids(&self) -> Keys<Id, Node<NodeData>> {
        self.nodes.keys()
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

pub struct EdgeIterator<'a, EdgeData> {
    edge_id: Option<Id>,
    edges: &'a EdgeMap<EdgeData>,
    direction: Direction,
}

impl<'a, EdgeData> Iterator for EdgeIterator<'a, EdgeData> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(edge_id) = self.edge_id {
            if let Some(edge) = self.edges.get(&edge_id) {
                self.edge_id = match self.direction {
                    Direction::Outgoing => edge.next_out,
                    Direction::Incoming => edge.next_in,
                };
                return self.edge_id;
            }
        }

        None
    }
}

impl<'a, EdgeData> EdgeIterator<'a, EdgeData> {
    pub fn new(edge_id: Id, direction: Direction, edges: &'a EdgeMap<EdgeData>) -> Self {
        Self {
            edge_id: Some(edge_id),
            edges,
            direction,
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
        let next_edge_id = match self.edge_id {
            Some(edge_id) => EdgeIterator::new(edge_id, self.direction, self.edges).next(),
            None => {
                if let Some(node) = self.nodes.get(&self.node_id) {
                    match self.direction {
                        Direction::Outgoing => node.outgoing,
                        Direction::Incoming => node.incoming,
                    }
                } else {
                    None
                }
            }
        };

        let next_node_id = match next_edge_id {
            Some(next_edge_id) => match self.edges.get(&next_edge_id) {
                Some(edge) => match self.direction {
                    Direction::Outgoing => Some(edge.to_node_id),
                    Direction::Incoming => Some(edge.from_node_id),
                },
                None => None,
            },
            None => None,
        };

        self.edge_id = next_edge_id;
        next_node_id
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
        let mut graph = Graph::with_capacity(5, 5);

        let node_a_id = graph._add_node(());
        let node_b_id = graph._add_node(());
        let node_c_id = graph._add_node(());
        let node_d_id = graph._add_node(());

        let a_to_b_id = graph.add_edge(node_a_id, node_b_id, ());
        let a_to_c_id = graph.add_edge(node_a_id, node_c_id, ());
        let a_to_d_id = graph.add_edge(node_a_id, node_d_id, ());

        let iterated_edges: Vec<Id> = graph._edge_iter(a_to_b_id, Direction::Outgoing).collect();

        assert_eq!(iterated_edges.len(), 2);
        assert_eq!(iterated_edges[0], a_to_c_id);
        assert_eq!(iterated_edges[1], a_to_d_id);
    }

    #[test]
    fn iterate_outgoing_nodes() {
        let mut graph = Graph::with_capacity(5, 5);

        let node_a_id = graph._add_node(());
        let node_b_id = graph._add_node(());
        let node_c_id = graph._add_node(());
        let node_d_id = graph._add_node(());

        graph.add_edge(node_a_id, node_b_id, ());
        graph.add_edge(node_a_id, node_c_id, ());
        graph.add_edge(node_a_id, node_d_id, ());

        let connected_nodes: Vec<Id> = graph.node_iter(node_a_id, Direction::Outgoing).collect();

        assert_eq!(connected_nodes.len(), 3);
        assert!(connected_nodes.contains(&node_b_id));
        assert!(connected_nodes.contains(&node_c_id));
        assert!(connected_nodes.contains(&node_d_id));
    }

    #[test]
    fn iterate_incoming_nodes() {
        let mut graph = Graph::with_capacity(5, 5);

        let node_a_id = graph._add_node(());
        let node_b_id = graph._add_node(());
        let node_c_id = graph._add_node(());
        let node_d_id = graph._add_node(());

        graph.add_edge(node_b_id, node_a_id, ());
        graph.add_edge(node_c_id, node_a_id, ());
        graph.add_edge(node_d_id, node_a_id, ());

        let connected_nodes: Vec<Id> = graph.node_iter(node_a_id, Direction::Incoming).collect();

        assert_eq!(connected_nodes.len(), 3);
        assert!(connected_nodes.contains(&node_b_id));
        assert!(connected_nodes.contains(&node_c_id));
        assert!(connected_nodes.contains(&node_d_id));
    }
}
