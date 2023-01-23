use std::sync::{atomic::AtomicI64, atomic::Ordering, Arc};

use crate::{
    commands::Command, AudioBuffer, AudioProcess, BorrowedAudioBuffer, MutableBorrowedAudioBuffer,
    Timestamp,
};
use lockfree::channel::mpsc::Receiver;

use super::dsp_graph::DspGraph;

const MAXIMUM_NUMBER_OF_FRAMES: usize = 512;
const MAXIMUM_NUMBER_OF_CHANNELS: usize = 2;

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,

    sample_position: usize,
    current_time: Arc<AtomicI64>,
    graph: DspGraph,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        command_rx: Receiver<Command>,
        current_time: Arc<AtomicI64>,
    ) -> Self {
        Self {
            started: false,
            sample_rate,
            command_rx,
            sample_position: 0,
            current_time,
            graph: DspGraph::new(
                MAXIMUM_NUMBER_OF_FRAMES,
                MAXIMUM_NUMBER_OF_CHANNELS,
                sample_rate,
            ),
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
            let num_frames = std::cmp::min(
                output_buffer.frame_count() - offset,
                self.get_maximum_number_of_frames(),
            );

            let input_slice = BorrowedAudioBuffer::slice_frames(input_buffer, offset, num_frames);

            let mut output_slice =
                MutableBorrowedAudioBuffer::slice_frames(output_buffer, offset, num_frames);

            let start_time = current_time.incremented_by_samples(offset, self.sample_rate);

            self.graph
                .process(&input_slice, &mut output_slice, &start_time);

            offset += num_frames;
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

        let num_frames = output_buffer.frame_count();
        self.process_graph(input_buffer, output_buffer);
        self.update_position(num_frames);
    }
}

impl Processor {
    fn process_commands(&mut self) {
        while let Ok(command) = self.command_rx.recv() {
            match command {
                Command::Start => self.started = true,
                Command::Stop => self.started = false,

                Command::AddDsp(dsp) => self.graph.add_dsp(dsp),
                Command::RemoveDsp(id) => self.graph.remove_dsp(id),

                Command::ParameterValueChange(change_request) => {
                    self.graph.request_parameter_change(change_request)
                }
                Command::CancelParamaterChanges(change_request) => {
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

    fn get_maximum_number_of_frames(&self) -> usize {
        MAXIMUM_NUMBER_OF_FRAMES
    }

    fn update_position(&mut self, num_samples: usize) {
        self.sample_position += num_samples;

        let seconds =
            Timestamp::from_seconds(self.sample_position as f64 / self.sample_rate as f64);
        self.current_time
            .store(seconds.as_raw_i64(), Ordering::Release);
    }
}
