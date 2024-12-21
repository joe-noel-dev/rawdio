use std::collections::{hash_map::Keys, HashMap};

use crate::commands::Id;

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
        EdgeIterator::new(node_id, None, direction, &self.nodes, &self.edges).last()
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

    fn find_edge_between_nodes(&self, from_node_id: Id, to_node_id: Id) -> Option<Id> {
        self.edges
            .iter()
            .find(|(_, edge)| edge.from_node_id == from_node_id && edge.to_node_id == to_node_id)
            .map(|(id, _)| *id)
    }

    pub fn remove_edge(&mut self, from_node_id: Id, to_node_id: Id) -> Option<EdgeData> {
        let id = self.find_edge_between_nodes(from_node_id, to_node_id);

        if let Some(id) = id {
            if let Some((_, edge)) = self.edges.remove_entry(&id) {
                replace_connections(self, Direction::Outgoing, id, edge.next_out);
                replace_connections(self, Direction::Incoming, id, edge.next_in);

                return Some(edge.edge_data);
            }
        }

        None
    }

    pub fn remove_node(&mut self, id: Id) -> Option<NodeData> {
        if let Some((_, node)) = self.nodes.remove_entry(&id) {
            assert!(node.incoming.is_none());
            assert!(node.outgoing.is_none());
            return Some(node.node_data);
        }

        None
    }

    pub fn get_node_mut(&mut self, id: Id) -> Option<&mut NodeData> {
        self.nodes.get_mut(&id).map(|node| &mut node.node_data)
    }

    pub fn add_node_with_id(&mut self, id: Id, node_data: NodeData) {
        assert!(!self.nodes.contains_key(&id));
        self.nodes.insert(id, Node::new(node_data));
    }

    pub fn is_connected_to(&self, from_node_id: Id, to_node_id: Id) -> bool {
        NodeIterator::new(from_node_id, Direction::Outgoing, &self.nodes, &self.edges)
            .any(|id| id == to_node_id)
    }

    pub fn node_iter(&self, node_id: Id, direction: Direction) -> NodeIterator<NodeData, EdgeData> {
        NodeIterator::new(node_id, direction, &self.nodes, &self.edges)
    }

    pub fn edge_iterator(
        &self,
        node_id: Id,
        direction: Direction,
    ) -> EdgeIterator<NodeData, EdgeData> {
        EdgeIterator::new(node_id, None, direction, &self.nodes, &self.edges)
    }

    pub fn get_edge(&self, edge_id: Id) -> Option<&Edge<EdgeData>> {
        self.edges.get(&edge_id)
    }

    pub fn connection_count(&self, node_id: Id, direction: Direction) -> usize {
        self.node_iter(node_id, direction).count()
    }

    pub fn all_node_ids(&self) -> Keys<Id, Node<NodeData>> {
        self.nodes.keys()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

fn replace_edge_connections<N, E>(
    graph: &mut Graph<N, E>,
    direction: Direction,
    find_edge_id: Id,
    replace_edge_id: Option<Id>,
) {
    for (_, edge) in graph.edges.iter_mut() {
        match direction {
            Direction::Outgoing => {
                if let Some(next_out) = edge.next_out {
                    if next_out == find_edge_id {
                        edge.next_out = replace_edge_id;
                    }
                }
            }
            Direction::Incoming => {
                if let Some(next_in) = edge.next_in {
                    if next_in == find_edge_id {
                        edge.next_in = replace_edge_id;
                    }
                }
            }
        }
    }
}

fn replace_node_connections<N, E>(
    graph: &mut Graph<N, E>,
    direction: Direction,
    find_edge_id: Id,
    replace_edge_id: Option<Id>,
) {
    for (_, node) in graph.nodes.iter_mut() {
        match direction {
            Direction::Outgoing => {
                if let Some(outgoing) = node.outgoing {
                    if outgoing == find_edge_id {
                        node.outgoing = replace_edge_id;
                    }
                }
            }
            Direction::Incoming => {
                if let Some(incoming) = node.incoming {
                    if incoming == find_edge_id {
                        node.incoming = replace_edge_id;
                    }
                }
            }
        }
    }
}

fn replace_connections<N, E>(
    graph: &mut Graph<N, E>,
    direction: Direction,
    find_edge_id: Id,
    replace_edge_id: Option<Id>,
) {
    replace_edge_connections(graph, direction, find_edge_id, replace_edge_id);
    replace_node_connections(graph, direction, find_edge_id, replace_edge_id);
}

pub struct EdgeIterator<'a, NodeData, EdgeData> {
    node_id: Id,
    edge_id: Option<Id>,
    nodes: &'a NodeMap<NodeData>,
    edges: &'a EdgeMap<EdgeData>,
    direction: Direction,
}

impl<NodeData, EdgeData> Iterator for EdgeIterator<'_, NodeData, EdgeData> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.get(&self.node_id)?;

        let next_edge_id = match self.edge_id {
            Some(edge_id) => {
                let edge = self.edges.get(&edge_id)?;

                match self.direction {
                    Direction::Outgoing => edge.next_out,
                    Direction::Incoming => edge.next_in,
                }
            }
            None => match self.direction {
                Direction::Outgoing => node.outgoing,
                Direction::Incoming => node.incoming,
            },
        };

        self.edge_id = next_edge_id;

        self.edge_id
    }
}

impl<'a, NodeData, EdgeData> EdgeIterator<'a, NodeData, EdgeData> {
    pub fn new(
        node_id: Id,
        edge_id: Option<Id>,
        direction: Direction,
        nodes: &'a NodeMap<NodeData>,
        edges: &'a EdgeMap<EdgeData>,
    ) -> Self {
        Self {
            node_id,
            edge_id,
            nodes,
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

impl<NodeData, EdgeData> Iterator for NodeIterator<'_, NodeData, EdgeData> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let next_edge_id = EdgeIterator::new(
            self.node_id,
            self.edge_id,
            self.direction,
            self.nodes,
            self.edges,
        )
        .next();

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

#[cfg(test)]
mod tests {

    use super::*;

    pub fn add_node<NodeData, EdgeData>(
        graph: &mut Graph<NodeData, EdgeData>,
        node_data: NodeData,
    ) -> Id {
        let id = Id::generate();
        graph.add_node_with_id(id, node_data);
        id
    }

    #[test]
    fn iterate_edges() {
        let mut graph = Graph::with_capacity(5, 5);

        //    A
        //  / | \
        // B  C  D

        let node_a_id = add_node(&mut graph, ());
        let node_b_id = add_node(&mut graph, ());
        let node_c_id = add_node(&mut graph, ());
        let node_d_id = add_node(&mut graph, ());

        let a_to_b_id = graph.add_edge(node_a_id, node_b_id, ());
        let a_to_c_id = graph.add_edge(node_a_id, node_c_id, ());
        let a_to_d_id = graph.add_edge(node_a_id, node_d_id, ());

        let iterated_edges: Vec<Id> = graph
            .edge_iterator(node_a_id, Direction::Outgoing)
            .collect();

        assert_eq!(iterated_edges.len(), 3);
        assert_eq!(iterated_edges[0], a_to_b_id);
        assert_eq!(iterated_edges[1], a_to_c_id);
        assert_eq!(iterated_edges[2], a_to_d_id);
    }

    #[test]
    fn iterate_outgoing_nodes() {
        let mut graph = Graph::with_capacity(5, 5);

        let node_a_id = add_node(&mut graph, ());
        let node_b_id = add_node(&mut graph, ());
        let node_c_id = add_node(&mut graph, ());
        let node_d_id = add_node(&mut graph, ());

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

        let node_a_id = add_node(&mut graph, ());
        let node_b_id = add_node(&mut graph, ());
        let node_c_id = add_node(&mut graph, ());
        let node_d_id = add_node(&mut graph, ());

        graph.add_edge(node_b_id, node_a_id, ());
        graph.add_edge(node_c_id, node_a_id, ());
        graph.add_edge(node_d_id, node_a_id, ());

        let connected_nodes: Vec<Id> = graph.node_iter(node_a_id, Direction::Incoming).collect();

        assert_eq!(connected_nodes.len(), 3);
        assert!(connected_nodes.contains(&node_b_id));
        assert!(connected_nodes.contains(&node_c_id));
        assert!(connected_nodes.contains(&node_d_id));
    }

    #[test]
    fn fixes_outgoing_connections() {
        let mut graph = Graph::with_capacity(3, 3);

        //    A
        //  /   \
        // B     C

        let node_a_id = add_node(&mut graph, ());
        let node_b_id = add_node(&mut graph, ());
        let node_c_id = add_node(&mut graph, ());

        graph.add_edge(node_a_id, node_b_id, ());
        graph.add_edge(node_a_id, node_c_id, ());

        let connected_nodes = graph.node_iter(node_a_id, Direction::Outgoing).count();
        assert_eq!(connected_nodes, 2);

        graph.remove_edge(node_a_id, node_b_id);

        let connected_nodes: Vec<Id> = graph.node_iter(node_a_id, Direction::Outgoing).collect();
        assert_eq!(connected_nodes.len(), 1);
        assert_eq!(connected_nodes[0], node_c_id);
    }

    #[test]
    fn fixes_incoming_connections() {
        let mut graph = Graph::with_capacity(3, 3);

        // A    B
        //  \  /
        //    C

        let node_a_id = add_node(&mut graph, ());
        let node_b_id = add_node(&mut graph, ());
        let node_c_id = add_node(&mut graph, ());

        graph.add_edge(node_a_id, node_c_id, ());
        graph.add_edge(node_b_id, node_c_id, ());

        let connected_nodes = graph.node_iter(node_c_id, Direction::Incoming).count();
        assert_eq!(connected_nodes, 2);

        graph.remove_edge(node_a_id, node_c_id);

        let connected_nodes: Vec<Id> = graph.node_iter(node_c_id, Direction::Incoming).collect();
        assert_eq!(connected_nodes.len(), 1);
        assert_eq!(connected_nodes[0], node_b_id);
    }
}
