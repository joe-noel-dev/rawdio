use super::dsp_graph::DspGraph;
use crate::{commands::Command, prelude::*};
use std::sync::{atomic::AtomicI64, atomic::Ordering, Arc};

type CommandReceiver = crossbeam::channel::Receiver<Command>;

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: CommandReceiver,

    frame_position: usize,
    current_time: Arc<AtomicI64>,
    graph: DspGraph,

    maximum_frame_count: usize,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        maximum_channel_count: usize,
        maximum_frame_count: usize,
        command_rx: CommandReceiver,
        current_time: Arc<AtomicI64>,
    ) -> Self {
        Self {
            started: false,
            sample_rate,
            command_rx,
            frame_position: 0,
            current_time,
            graph: DspGraph::new(maximum_frame_count, maximum_channel_count, sample_rate),
            maximum_frame_count,
        }
    }

    fn process_graph(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
    ) {
        let mut offset = 0;

        let current_time = Timestamp::from_raw_i64(self.current_time.load(Ordering::Acquire));

        while offset < output_buffer.frame_count() {
            let frame_count = std::cmp::min(
                output_buffer.frame_count() - offset,
                self.maximum_frame_count,
            );

            let input_slice = BorrowedAudioBuffer::slice_frames(input_buffer, offset, frame_count);

            let mut output_slice =
                MutableBorrowedAudioBuffer::slice_frames(output_buffer, offset, frame_count);

            let start_time = current_time.incremented_by_samples(offset, self.sample_rate);

            self.graph
                .process(&input_slice, &mut output_slice, &start_time);

            offset += frame_count;
        }
    }
}

impl AudioProcess for Processor {
    fn process(&mut self, input_buffer: &dyn AudioBuffer, output_buffer: &mut dyn AudioBuffer) {
        debug_assert_eq!(input_buffer.frame_count(), output_buffer.frame_count());

        output_buffer.clear();

        self.process_commands();

        if !self.started {
            return;
        }

        let frame_count = output_buffer.frame_count();
        self.process_graph(input_buffer, output_buffer);
        self.update_position(frame_count);
    }
}

impl Processor {
    fn process_commands(&mut self) {
        while let Ok(command) = self.command_rx.try_recv() {
            match command {
                Command::Start => self.started = true,
                Command::Stop => self.started = false,

                Command::AddDsp(dsp) => self.graph.add_dsp(dsp),
                Command::RemoveDsp(id) => self.graph.remove_dsp(id),

                Command::ParameterValueChange(change_request) => {
                    self.graph.request_parameter_change(change_request)
                }
                Command::CancelParameterChanges(change_request) => {
                    self.graph.cancel_parameter_changes(change_request)
                }

                Command::AddConnection(connection) => self.graph.add_connection(connection),
                Command::RemoveConnection(connection) => self.graph.remove_connection(connection),
                Command::ConnectToOutput(output_endpoint) => {
                    self.graph.connect_to_output(output_endpoint)
                }
                Command::ConnectToInput(input_endpoint) => {
                    self.graph.connect_to_input(input_endpoint)
                }
            }
        }
    }

    fn update_position(&mut self, frame_count: usize) {
        self.frame_position += frame_count;

        let seconds = Timestamp::from_seconds(self.frame_position as f64 / self.sample_rate as f64);
        self.current_time
            .store(seconds.as_raw_i64(), Ordering::Release);
    }
}
