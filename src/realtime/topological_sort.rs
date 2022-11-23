use std::collections::HashMap;

use crate::{commands::Id, realtime::graph::Direction};

use super::graph::Graph;

pub struct TopologicalSort {
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

    pub fn get_sorted_graph(&self) -> &[Id] {
        &self.order
    }

    pub fn sort<NodeData, EdgeData>(&mut self, graph: &Graph<NodeData, EdgeData>) -> &[Id] {
        self.dependency_count.clear();
        self.order.clear();
        self.ready_to_process.clear();

        for node_id in graph.all_node_ids() {
            let num_incoming_nodes = graph.num_connections(*node_id, Direction::Incoming);
            self.dependency_count.insert(*node_id, num_incoming_nodes);
        }

        while let Some(next_node_id) = self.node_without_dependencies() {
            self.order.push(next_node_id);
            self.dependency_count.remove(&next_node_id);

            for node_id in graph.all_node_ids() {
                if graph.is_connected_to(next_node_id, *node_id) {
                    let previous_value = self.dependency_count.get_mut(node_id).unwrap();
                    assert!(*previous_value > 0);
                    *previous_value -= 1;
                }
            }
        }

        assert_eq!(self.order.len(), graph.num_nodes()); // cycle detected

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
    fn sorts_graph_into_order() {
        //         C
        //       /   \
        // A - B       E
        //       \   /
        //         D

        let mut graph = Graph::with_capacity(5, 5);

        let a_id = add_node(&mut graph, String::from("A"));
        let b_id = add_node(&mut graph, String::from("B"));
        let c_id = add_node(&mut graph, String::from("C"));
        let d_id = add_node(&mut graph, String::from("D"));
        let e_id = add_node(&mut graph, String::from("E"));

        graph.add_edge(a_id, b_id, ());
        graph.add_edge(b_id, c_id, ());
        graph.add_edge(b_id, d_id, ());
        graph.add_edge(c_id, e_id, ());
        graph.add_edge(d_id, e_id, ());

        let mut topo_sort = TopologicalSort::with_capacity(5);
        let sorted = topo_sort.sort(&graph);

        assert_eq!(sorted.len(), 5);
        assert_eq!(sorted[0], a_id);
        assert!(sorted[1] == b_id);
        assert!(sorted[2] == c_id || sorted[2] == d_id);
        assert!(sorted[3] == c_id || sorted[3] == d_id);
        assert_eq!(sorted[4], e_id);
    }
}
