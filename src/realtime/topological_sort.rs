use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use crate::commands::id::Id;

use super::{
    edge::Edge,
    graph_utils::{is_connected_to, Direction, EdgeMap, NodeIterator, NodeMap},
    node::Node,
};

struct TopologicalSort {
    dependency_count: HashMap<Id, usize>,
    order: Vec<Id>,
    ready_to_process: Vec<Id>,
}

impl TopologicalSort {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dependency_count: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
            ready_to_process: Vec::with_capacity(capacity),
        }
    }

    pub fn sort<NodeData, EdgeData>(
        &mut self,
        nodes: &NodeMap<NodeData>,
        edges: &EdgeMap<EdgeData>,
    ) -> &[Id] {
        self.dependency_count.clear();
        self.order.clear();
        self.ready_to_process.clear();

        for node_id in nodes.keys() {
            let num_incoming_nodes =
                NodeIterator::new(*node_id, Direction::Incoming, nodes, edges).count();
            self.dependency_count.insert(*node_id, num_incoming_nodes);
        }

        while let Some(next_node_id) = self.node_without_dependencies() {
            self.order.push(next_node_id);
            self.dependency_count.remove(&next_node_id);

            for node_id in nodes.keys() {
                if is_connected_to(next_node_id, *node_id, nodes, edges) {
                    let previous_value = self.dependency_count.get_mut(node_id).unwrap();
                    assert!(*previous_value > 0);
                    *previous_value -= 1;
                }
            }
        }

        assert_eq!(self.order.len(), nodes.len()); // cycle detected

        &self.order
    }

    fn node_without_dependencies(&self) -> Option<Id> {
        self.dependency_count
            .iter()
            .find(|(_, count)| **count == 0)
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use crate::realtime::graph_utils::add_connection;

    use super::*;

    #[test]
    fn sorts_graph_into_order() {
        //         C
        //       /   \
        // A - B       E
        //       \   /
        //         D

        let node_a = Node::new(RefCell::new(String::from("A")));
        let node_b = Node::new(RefCell::new(String::from("B")));
        let node_c = Node::new(RefCell::new(String::from("C")));
        let node_d = Node::new(RefCell::new(String::from("D")));
        let node_e = Node::new(RefCell::new(String::from("E")));

        let a_id = Id::generate();
        let b_id = Id::generate();
        let c_id = Id::generate();
        let d_id = Id::generate();
        let e_id = Id::generate();

        let mut nodes = NodeMap::from([
            (a_id, node_a),
            (b_id, node_b),
            (c_id, node_c),
            (d_id, node_d),
            (e_id, node_e),
        ]);

        let mut edges = EdgeMap::new();

        add_connection(a_id, b_id, (), &mut nodes, &mut edges);
        add_connection(b_id, c_id, (), &mut nodes, &mut edges);
        add_connection(b_id, d_id, (), &mut nodes, &mut edges);
        add_connection(c_id, e_id, (), &mut nodes, &mut edges);
        add_connection(d_id, e_id, (), &mut nodes, &mut edges);

        let mut topo_sort = TopologicalSort::with_capacity(5);
        let sorted = topo_sort.sort(&nodes, &edges);

        assert_eq!(sorted.len(), 5);
        assert_eq!(sorted[0], a_id);
        assert!(sorted[1] == b_id || sorted[1] == c_id);
        assert!(sorted[2] == b_id || sorted[2] == c_id);
        assert_eq!(sorted[3], d_id);
        assert_eq!(sorted[4], e_id);
    }
}
