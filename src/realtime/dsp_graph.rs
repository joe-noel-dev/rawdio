use std::cell::RefCell;

use lockfree::channel::{spsc, spsc::Sender};

use crate::{
    buffer::{audio_buffer::AudioBuffer, sample_location::SampleLocation},
    commands::{command::ParameterChangeRequest, id::Id},
    graph::{
        buffer_pool::BufferPool,
        connection::Connection,
        dsp::Dsp,
        endpoint::{Endpoint, EndpointType},
    },
    timestamp::Timestamp,
    utility::garbage_collector::{run_garbage_collector, GarbageCollectionCommand},
};

use super::{
    graph::{Direction, Graph},
    topological_sort::TopologicalSort,
};

pub struct DspGraph {
    graph: Graph<RefCell<Dsp>, Connection>,
    topological_sort: TopologicalSort,
    output_endpoint: Option<Endpoint>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
    graph_needs_sort: bool,
    buffer_pool: BufferPool,
}

impl DspGraph {
    pub fn new(
        maximum_number_of_frames: usize,
        maximum_number_of_channels: usize,
        sample_rate: usize,
    ) -> Self {
        let (garbase_collection_tx, garbage_collection_rx) = spsc::create();
        run_garbage_collector(garbage_collection_rx);

        Self {
            graph: Graph::with_capacity(512, 512),
            topological_sort: TopologicalSort::with_capacity(512),
            graph_needs_sort: false,
            output_endpoint: None,
            garbase_collection_tx,
            buffer_pool: BufferPool::with_capacity(
                128,
                maximum_number_of_frames,
                maximum_number_of_channels,
                sample_rate,
            ),
        }
    }

    pub fn process(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        output_buffer.clear();

        let num_channels = output_buffer.num_channels();
        let num_frames = output_buffer.num_frames();

        if self.graph_needs_sort {
            self.topological_sort.sort(&self.graph);
            self.graph_needs_sort = false;
        }

        for dsp_id in self.topological_sort.get_sorted_graph() {
            let output_endpoint = Endpoint::new(*dsp_id, EndpointType::Output);

            let mut node_input_buffer = self.buffer_pool.get_unassigned_buffer().unwrap();
            let mut node_output_buffer = self.buffer_pool.get_unassigned_buffer().unwrap();

            for connected_node_id in self.graph.node_iter(*dsp_id, Direction::Incoming) {
                let endpoint = Endpoint::new(connected_node_id, EndpointType::Output);
                if let Some(buffer) = self.buffer_pool.get_assigned_buffer(endpoint) {
                    let sample_location = SampleLocation::new(0, 0);
                    node_input_buffer.add_from(
                        &buffer,
                        &sample_location,
                        &sample_location,
                        num_channels,
                        num_frames,
                    );

                    self.buffer_pool
                        .return_buffer_with_assignment(buffer, endpoint);
                }
            }

            self.process_dsp(
                *dsp_id,
                &node_input_buffer,
                &mut node_output_buffer,
                start_time,
            );

            self.buffer_pool.return_buffer(node_input_buffer);
            self.buffer_pool
                .return_buffer_with_assignment(node_output_buffer, output_endpoint);
        }

        if let Some(output_endpoint) = self.output_endpoint {
            if let Some(buffer) = self.buffer_pool.get_assigned_buffer(output_endpoint) {
                let sample_location = SampleLocation::new(0, 0);
                output_buffer.add_from(
                    &buffer,
                    &sample_location,
                    &sample_location,
                    num_channels,
                    num_frames,
                );

                self.buffer_pool
                    .return_buffer_with_assignment(buffer, output_endpoint);
            }
        }

        self.buffer_pool.clear_assignments();
        assert!(self.buffer_pool.all_buffers_are_available())
    }

    pub fn add_dsp(&mut self, dsp: RefCell<Dsp>) {
        let id = dsp.borrow().get_id();
        self.graph.add_node_with_id(id, dsp);
        self.mark_graph_needs_sort();
    }

    pub fn mark_graph_needs_sort(&mut self) {
        self.graph_needs_sort = true;
    }

    pub fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.graph.remove_node(id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }

        self.mark_graph_needs_sort();
    }

    pub fn request_parameter_change(&mut self, change_request: ParameterChangeRequest) {
        if let Some(dsp) = self.graph.get_node_mut(change_request.dsp_id) {
            dsp.get_mut().request_parameter_change(change_request);
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
        // TODO: Remove conflicting connections

        self.graph.add_edge(
            connection.source.dsp_id,
            connection.destination.dsp_id,
            connection,
        );

        self.mark_graph_needs_sort();
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        self.graph
            .remove_edge(connection.source.dsp_id, connection.destination.dsp_id);

        self.mark_graph_needs_sort();
    }

    pub fn connect_to_output(&mut self, output_endpoint: Endpoint) {
        self.output_endpoint = Some(output_endpoint);
    }

    fn process_dsp(
        &self,
        dsp_id: Id,
        input_buffer: &impl AudioBuffer,
        output_buffer: &mut impl AudioBuffer,
        start_time: &Timestamp,
    ) {
        let dsp = match self.graph.get_node(dsp_id) {
            Some(node) => node,
            None => return,
        };

        dsp.borrow_mut()
            .process_audio(input_buffer, output_buffer, start_time);
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_relative_eq, assert_relative_ne};

    use crate::{
        buffer::owned_audio_buffer::OwnedAudioBuffer,
        graph::dsp::{DspParameterMap, DspProcessor},
    };

    use super::*;

    struct Processor {
        value_to_write: f32,
        location_to_write: SampleLocation,
    }

    impl Processor {
        fn new(value_to_write: f32, location_to_write: SampleLocation) -> Self {
            Self {
                value_to_write,
                location_to_write,
            }
        }
    }

    impl DspProcessor for Processor {
        fn process_audio(
            &mut self,
            input_buffer: &dyn AudioBuffer,
            output_buffer: &mut dyn AudioBuffer,
            _start_time: &Timestamp,
            _parameters: &DspParameterMap,
        ) {
            output_buffer.add_from(
                input_buffer,
                &SampleLocation::new(0, 0),
                &SampleLocation::new(0, 0),
                output_buffer.num_channels(),
                output_buffer.num_frames(),
            );

            output_buffer.set_sample(&self.location_to_write, self.value_to_write);
        }
    }

    fn make_dsp(value_to_write: f32, location_to_write: SampleLocation) -> RefCell<Dsp> {
        let processor = Box::new(Processor::new(value_to_write, location_to_write));
        let parameters = DspParameterMap::new();
        RefCell::new(Dsp::new(Id::generate(), processor, parameters))
    }

    #[test]
    fn renders_when_connected_to_output() {
        let value = 0.456;
        let location = SampleLocation::new(0, 27);

        let dsp = make_dsp(value, location);

        let dsp_id = dsp.borrow().get_id();
        let sample_rate = 44100;

        let mut graph = DspGraph::new(128, 2, sample_rate);
        graph.add_dsp(dsp);

        let num_frames = 128;

        let mut audio_buffer = OwnedAudioBuffer::new(num_frames, 2, sample_rate);
        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_relative_ne!(audio_buffer.get_sample(&location), value);

        graph.connect_to_output(Endpoint::new(dsp_id, EndpointType::Output));

        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_relative_eq!(audio_buffer.get_sample(&location), value);
    }

    #[test]
    fn renders_chain() {
        let value_1 = 0.123;
        let value_2 = 0.456;

        let location_1 = SampleLocation::new(0, 27);
        let location_2 = SampleLocation::new(1, 38);

        let dsp_1 = make_dsp(value_1, location_1);
        let dsp_2 = make_dsp(value_2, location_2);

        let dsp_id_1 = dsp_1.borrow().get_id();
        let dsp_id_2 = dsp_2.borrow().get_id();

        let sample_rate = 44100;

        let mut graph = DspGraph::new(128, 2, sample_rate);

        graph.add_dsp(dsp_1);
        graph.add_dsp(dsp_2);

        let num_frames = 128;

        graph.connect_to_output(Endpoint::new(dsp_id_2, EndpointType::Output));

        graph.add_connection(Connection::new(dsp_id_1, dsp_id_2));

        let mut audio_buffer = OwnedAudioBuffer::new(num_frames, 2, 44100);
        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_relative_eq!(audio_buffer.get_sample(&location_1), value_1);
        assert_relative_eq!(audio_buffer.get_sample(&location_2), value_2);
    }
}
