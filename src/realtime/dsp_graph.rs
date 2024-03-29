use super::{garbage_collector::*, graph::*, topological_sort::TopologicalSort};
use crate::{buffer::BufferPool, commands::*, graph::*, prelude::*};

struct BufferPools {
    free: BufferPool,
    assigned: AssignedBufferPool<Endpoint>,
}

pub struct DspGraph {
    graph: Graph<Box<Dsp>, Connection>,
    topological_sort: TopologicalSort,
    input_endpoint: Option<Endpoint>,
    output_endpoint: Option<Endpoint>,
    garbage_collection_tx: GarbaseCollectionSender,
    graph_needs_sort: bool,
    buffer_pools: BufferPools,
    maximum_channel_count: usize,
    maximum_frame_count: usize,
}

static MAXIMUM_BUFFER_COUNT: usize = 1024;
static MAXIMUM_GRAPH_NODE_COUNT: usize = 512;
static MAXIMUM_GRAPH_EDGE_COUNT: usize = 512;
static GARBAGE_COLLECTION_CHANNEL_CAPACITY: usize = 512;

impl DspGraph {
    pub fn new(
        maximum_frame_count: usize,
        maximum_channel_count: usize,
        sample_rate: usize,
    ) -> Self {
        let (garbage_collection_tx, garbage_collection_rx) =
            crossbeam::channel::bounded(GARBAGE_COLLECTION_CHANNEL_CAPACITY);
        run_garbage_collector(garbage_collection_rx);

        Self {
            graph: Graph::with_capacity(MAXIMUM_GRAPH_NODE_COUNT, MAXIMUM_GRAPH_EDGE_COUNT),
            topological_sort: TopologicalSort::with_capacity(MAXIMUM_GRAPH_NODE_COUNT),
            graph_needs_sort: false,
            input_endpoint: None,
            output_endpoint: None,
            garbage_collection_tx,
            buffer_pools: BufferPools {
                free: BufferPool::new(
                    MAXIMUM_BUFFER_COUNT,
                    maximum_frame_count,
                    maximum_channel_count,
                    sample_rate,
                ),
                assigned: AssignedBufferPool::with_capacity(MAXIMUM_BUFFER_COUNT),
            },
            maximum_channel_count,
            maximum_frame_count,
        }
    }

    pub fn process(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        let input_channel_count =
            std::cmp::min(input_buffer.channel_count(), self.maximum_channel_count);
        let output_channel_count =
            std::cmp::min(output_buffer.channel_count(), self.maximum_channel_count);
        let frame_count = std::cmp::min(output_buffer.frame_count(), self.maximum_frame_count);

        self.sort_graph();

        if let Some(input_endpoint) = self.input_endpoint {
            if let Some(mut buffer) = self.buffer_pools.free.remove() {
                buffer.copy_from(
                    input_buffer,
                    SampleLocation::origin(),
                    SampleLocation::origin(),
                    input_channel_count,
                    frame_count,
                );

                self.buffer_pools.assigned.add(buffer, &input_endpoint);
            }
        }

        self.process_dsps(frame_count, start_time);

        if let Some(output_endpoint) = self.output_endpoint {
            let source_channel = 0;
            let destination_channel = 0;

            mix_endpoint(
                &mut self.buffer_pools.assigned,
                &output_endpoint,
                output_buffer,
                source_channel,
                destination_channel,
                output_channel_count,
                frame_count,
                MixBehaviour::Overwrite,
            );
        }

        while let Some((_, buffer)) = self.buffer_pools.assigned.remove_next() {
            self.buffer_pools.free.add(buffer);
        }

        debug_assert!(self.buffer_pools.assigned.is_empty());
    }

    pub fn add_dsp(&mut self, dsp: Box<Dsp>) {
        let id = dsp.get_id();
        self.graph.add_node_with_id(id, dsp);
        self.mark_graph_needs_sort();
    }

    fn mark_graph_needs_sort(&mut self) {
        self.graph_needs_sort = true;
    }

    fn sort_graph(&mut self) {
        if self.graph_needs_sort {
            self.topological_sort.sort(&self.graph);
            self.graph_needs_sort = false;
        }
    }

    pub fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.graph.remove_node(id) {
            let _ = self
                .garbage_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }

        self.mark_graph_needs_sort();
    }

    pub fn request_parameter_change(&mut self, change_request: ParameterChangeRequest) {
        if let Some(dsp) = self.graph.get_node_mut(change_request.dsp_id) {
            dsp.request_parameter_change(change_request);
        }
    }

    pub fn cancel_parameter_changes(&mut self, change_request: CancelChangeRequest) {
        if let Some(dsp) = self.graph.get_node_mut(change_request.dsp_id) {
            dsp.cancel_parameter_changes(change_request);
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
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

    pub fn connect_to_input(&mut self, input_endpoint: Endpoint) {
        self.input_endpoint = Some(input_endpoint);
    }

    fn process_dsps(&mut self, frame_count: usize, start_time: &Timestamp) {
        let sorted_graph = self.topological_sort.get_sorted_graph();
        for dsp_id in sorted_graph {
            debug_assert!(can_process_dsp(
                dsp_id,
                &self.graph,
                &self.buffer_pools.assigned
            ));

            process_dsp(
                &mut self.buffer_pools,
                &mut self.graph,
                *dsp_id,
                frame_count,
                start_time,
            );
        }
    }
}

fn process_dsp(
    buffer_pools: &mut BufferPools,
    graph: &mut Graph<Box<Dsp>, Connection>,
    dsp_id: Id,
    frame_count: usize,
    start_time: &Timestamp,
) {
    let output_endpoint = Endpoint::new(dsp_id, EndpointType::Output);

    let mut output_buffer = get_buffer_with_endpoint(&output_endpoint, buffer_pools);

    let (input_buffer, input_endpoint) = prepare_input(buffer_pools, graph, dsp_id, frame_count);

    if let Some(dsp) = graph.get_node_mut(dsp_id) {
        let mut output_slice = MutableBorrowedAudioBuffer::slice_channels_and_frames(
            &mut output_buffer,
            frame_count,
            dsp.output_count(),
        );

        let input_slice = BorrowedAudioBuffer::slice_channels_and_frames(
            &input_buffer,
            frame_count,
            dsp.input_count(),
        );

        dsp.process_audio(&input_slice, &mut output_slice, start_time);
    };

    buffer_pools.assigned.add(output_buffer, &output_endpoint);

    if input_endpoint.endpoint_type == EndpointType::Input {
        buffer_pools.free.add(input_buffer);
    } else {
        buffer_pools.assigned.add(input_buffer, &input_endpoint);
    }
}

fn can_process_dsp(
    id: &Id,
    graph: &Graph<Box<Dsp>, Connection>,
    assigned_buffer_pool: &AssignedBufferPool<Endpoint>,
) -> bool {
    graph
        .node_iter(*id, Direction::Incoming)
        .all(|incoming_node_id| {
            assigned_buffer_pool.has(&Endpoint::new(incoming_node_id, EndpointType::Output))
        })
}

enum MixBehaviour {
    Overwrite,
    Mix,
}

#[allow(clippy::too_many_arguments)]
fn mix_endpoint(
    assigned_buffer_pool: &mut AssignedBufferPool<Endpoint>,
    endpoint: &Endpoint,
    output_buffer: &mut dyn AudioBuffer,
    source_channel: usize,
    destination_channel: usize,
    channel_count: usize,
    frame_count: usize,
    mix_behaviour: MixBehaviour,
) {
    if let Some(buffer) = assigned_buffer_pool.remove(endpoint) {
        let source_location = SampleLocation::channel(source_channel);
        let destination_location = SampleLocation::channel(destination_channel);

        match mix_behaviour {
            MixBehaviour::Overwrite => output_buffer.copy_from(
                &buffer,
                source_location,
                destination_location,
                channel_count,
                frame_count,
            ),
            MixBehaviour::Mix => output_buffer.add_from(
                &buffer,
                source_location,
                destination_location,
                channel_count,
                frame_count,
            ),
        }

        assigned_buffer_pool.add(buffer, endpoint);
    }
}

fn copy_output_from_dependencies(
    assigned_buffer_pool: &mut AssignedBufferPool<Endpoint>,
    graph: &Graph<Box<Dsp>, Connection>,
    dsp_id: Id,
    destination_buffer: &mut dyn AudioBuffer,
    frame_count: usize,
) {
    let mut mix_behaviour = MixBehaviour::Overwrite;

    for edge_id in graph.edge_iterator(dsp_id, Direction::Incoming) {
        let edge = graph.get_edge(edge_id).expect("Edge not found");

        let endpoint = Endpoint::new(edge.from_node_id, EndpointType::Output);

        mix_endpoint(
            assigned_buffer_pool,
            &endpoint,
            destination_buffer,
            edge.edge_data.source_output_channel,
            edge.edge_data.destination_input_channel,
            edge.edge_data.channel_count,
            frame_count,
            mix_behaviour,
        );

        mix_behaviour = MixBehaviour::Mix;
    }
}

fn get_buffer_with_endpoint(
    endpoint: &Endpoint,
    buffer_pools: &mut BufferPools,
) -> OwnedAudioBuffer {
    match buffer_pools.assigned.remove(endpoint) {
        Some(buffer) => buffer,
        None => buffer_pools
            .free
            .remove()
            .expect("No buffers available for processing"),
    }
}

fn prepare_n_input_node(
    buffer_pools: &mut BufferPools,
    graph: &Graph<Box<Dsp>, Connection>,
    dsp_id: Id,
    frame_count: usize,
) -> (OwnedAudioBuffer, Endpoint) {
    let input_endpoint = Endpoint::new(dsp_id, EndpointType::Input);

    let mut node_input_buffer = get_buffer_with_endpoint(&input_endpoint, buffer_pools);

    copy_output_from_dependencies(
        &mut buffer_pools.assigned,
        graph,
        dsp_id,
        &mut node_input_buffer,
        frame_count,
    );

    (node_input_buffer, input_endpoint)
}

fn prepare_zero_input_node(
    buffer_pools: &mut BufferPools,
    dsp_id: Id,
) -> (OwnedAudioBuffer, Endpoint) {
    let input_endpoint = Endpoint::new(dsp_id, EndpointType::Input);

    (
        get_buffer_with_endpoint(&input_endpoint, buffer_pools),
        input_endpoint,
    )
}

fn prepare_single_input_node(
    buffer_pools: &mut BufferPools,
    graph: &Graph<Box<Dsp>, Connection>,
    dsp_id: Id,
    frame_count: usize,
) -> (OwnedAudioBuffer, Endpoint) {
    let edge_id = graph
        .edge_iterator(dsp_id, Direction::Incoming)
        .next()
        .unwrap();

    let edge = graph.get_edge(edge_id).unwrap();

    if edge.edge_data.destination_input_channel == 0 && edge.edge_data.source_output_channel == 0 {
        let input_endpoint = Endpoint::new(edge.from_node_id, EndpointType::Output);

        return (
            get_buffer_with_endpoint(&input_endpoint, buffer_pools),
            input_endpoint,
        );
    }

    prepare_n_input_node(buffer_pools, graph, dsp_id, frame_count)
}

fn prepare_input(
    buffer_pools: &mut BufferPools,
    graph: &Graph<Box<Dsp>, Connection>,
    dsp_id: Id,
    frame_count: usize,
) -> (OwnedAudioBuffer, Endpoint) {
    match graph.connection_count(dsp_id, Direction::Incoming) {
        0 => prepare_zero_input_node(buffer_pools, dsp_id),
        1 => prepare_single_input_node(buffer_pools, graph, dsp_id, frame_count),
        _ => prepare_n_input_node(buffer_pools, graph, dsp_id, frame_count),
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_relative_eq, assert_relative_ne};

    use crate::{
        graph::{DspParameters, DspProcessor},
        AudioBuffer, OwnedAudioBuffer,
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
        fn process_audio(&mut self, context: &mut crate::ProcessContext) {
            context.output_buffer.add_from(
                context.input_buffer,
                SampleLocation::origin(),
                SampleLocation::origin(),
                context.output_buffer.channel_count(),
                context.output_buffer.frame_count(),
            );

            context
                .output_buffer
                .set_sample(self.location_to_write, self.value_to_write);
        }
    }

    fn make_dsp(
        value_to_write: f32,
        location_to_write: SampleLocation,
        input_count: usize,
        output_count: usize,
    ) -> Box<Dsp> {
        let processor = Box::new(Processor::new(value_to_write, location_to_write));

        Box::new(Dsp::new(
            Id::generate(),
            input_count,
            output_count,
            processor,
            DspParameters::empty(),
        ))
    }

    #[test]
    fn renders_when_connected_to_output() {
        let value = 0.456;
        let location = SampleLocation::frame(27);
        let channel_count = 2;

        let dsp = make_dsp(value, location, channel_count, channel_count);

        let dsp_id = dsp.get_id();
        let sample_rate = 44100;

        let mut graph = DspGraph::new(128, 2, sample_rate);
        graph.add_dsp(dsp);

        let frame_count = 128;

        let input_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::default());

        assert_relative_ne!(output_buffer.get_sample(location), value);

        graph.connect_to_output(Endpoint::new(dsp_id, EndpointType::Output));

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::default());

        assert_relative_eq!(output_buffer.get_sample(location), value);
    }

    #[test]
    fn renders_chain() {
        let value_1 = 0.123;
        let value_2 = 0.456;

        let location_1 = SampleLocation::new(0, 27);
        let location_2 = SampleLocation::new(1, 38);

        let channel_count = 2;

        let dsp_1 = make_dsp(value_1, location_1, channel_count, channel_count);
        let dsp_2 = make_dsp(value_2, location_2, channel_count, channel_count);

        let dsp_id_1 = dsp_1.get_id();
        let dsp_id_2 = dsp_2.get_id();

        let sample_rate = 44100;

        let mut graph = DspGraph::new(128, channel_count, sample_rate);

        graph.add_dsp(dsp_1);
        graph.add_dsp(dsp_2);

        let frame_count = 128;

        graph.connect_to_output(Endpoint::new(dsp_id_2, EndpointType::Output));

        graph.add_connection(Connection::new(dsp_id_1, dsp_id_2, channel_count));

        let input_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::default());

        assert_relative_eq!(output_buffer.get_sample(location_1), value_1);
        assert_relative_eq!(output_buffer.get_sample(location_2), value_2);
    }

    #[test]
    fn doesnt_write_too_many_channels() {
        let channel_count = 2;
        let dsp = make_dsp(0.0, SampleLocation::origin(), channel_count, channel_count);
        let dsp_id = dsp.get_id();
        let sample_rate = 44100;

        let mut graph = DspGraph::new(128, channel_count, sample_rate);

        graph.add_dsp(dsp);

        let frame_count = 128;

        graph.connect_to_output(Endpoint::new(dsp_id, EndpointType::Output));

        let input_buffer = OwnedAudioBuffer::new(frame_count, channel_count * 2, sample_rate);
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count * 2, sample_rate);

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::default());
    }

    #[test]
    fn doesnt_write_too_many_frames() {
        let channel_count = 2;

        let dsp = make_dsp(0.0, SampleLocation::origin(), channel_count, channel_count);
        let dsp_id = dsp.get_id();
        let sample_rate = 44100;
        let maximum_number_of_frames = 512;

        let mut graph = DspGraph::new(maximum_number_of_frames, 2, sample_rate);

        graph.add_dsp(dsp);

        graph.connect_to_output(Endpoint::new(dsp_id, EndpointType::Output));

        let input_buffer =
            OwnedAudioBuffer::new(maximum_number_of_frames * 2, channel_count, sample_rate);
        let mut output_buffer =
            OwnedAudioBuffer::new(maximum_number_of_frames * 2, channel_count, sample_rate);

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::default());
    }

    #[test]
    fn multichannel_routing() {
        let maximum_frame_count = 1024;
        let graph_channel_count = 10;
        let sample_rate = 48_000;

        let dsp_1_channel_count = 4;

        let value_1 = 1.0;
        let value_2 = 2.0;

        let value_1_location = SampleLocation::new(2, 35);
        let value_2_location = SampleLocation::new(0, 89);

        let dsp_1 = make_dsp(
            value_1,
            value_1_location,
            dsp_1_channel_count,
            dsp_1_channel_count,
        );

        let dsp_2 = make_dsp(
            value_2,
            value_2_location,
            graph_channel_count,
            graph_channel_count,
        );

        let dsp_1_id = dsp_1.get_id();
        let dsp_2_id = dsp_2.get_id();

        let mut graph = DspGraph::new(maximum_frame_count, graph_channel_count, sample_rate);

        graph.add_dsp(dsp_1);
        graph.add_dsp(dsp_2);

        let destination_input_channel = 5;
        let source_output_channel = value_1_location.channel;
        let connection_channel_count = 1;

        graph.add_connection(
            Connection::new(dsp_1_id, dsp_2_id, connection_channel_count)
                .with_destination_input_channel(destination_input_channel)
                .with_source_output_channel(source_output_channel),
        );

        graph.connect_to_output(Endpoint::new(dsp_2_id, EndpointType::Output));

        let input_buffer =
            OwnedAudioBuffer::new(maximum_frame_count, graph_channel_count, sample_rate);
        let mut output_buffer =
            OwnedAudioBuffer::new(maximum_frame_count, graph_channel_count, sample_rate);

        graph.process(&input_buffer, &mut output_buffer, &Timestamp::zero());

        assert_relative_eq!(
            output_buffer.get_sample(value_1_location.with_channel(destination_input_channel)),
            value_1
        );
        assert_relative_eq!(output_buffer.get_sample(value_2_location), value_2);
    }
}
