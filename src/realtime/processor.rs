use crate::{
    audio_process::AudioProcess,
    buffer::audio_buffer::AudioBuffer,
    commands::{command::Command, notification::Notification},
    timestamp::Timestamp,
};
use lockfree::channel::{mpsc::Receiver, spsc::Sender};

use super::dsp_graph::DspGraph;

const MAXIMUM_NUMBER_OF_FRAMES: usize = 1024;
const MAXIMUM_NUMBER_OF_CHANNELS: usize = 2;

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,

    sample_position: usize,
    graph: DspGraph,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        command_rx: Receiver<Command>,
        notification_tx: Sender<Notification>,
    ) -> Self {
        Self {
            started: false,
            sample_rate,
            command_rx,
            notification_tx,
            sample_position: 0,
            graph: DspGraph::default(),
        }
    }
}

impl AudioProcess for Processor {
    fn process(&mut self, output_buffer: &mut dyn AudioBuffer) {
        assert!(output_buffer.num_frames() <= self.get_maximum_number_of_frames());

        output_buffer.clear();

        self.process_commands();

        if !self.started {
            return;
        }

        let current_time = self.current_time();

        self.graph.process(output_buffer, &current_time);

        self.update_position(output_buffer.num_frames());
        self.notify_position();
    }

    fn get_maximum_number_of_frames(&self) -> usize {
        MAXIMUM_NUMBER_OF_FRAMES
    }

    fn get_maximum_number_of_channel(&self) -> usize {
        MAXIMUM_NUMBER_OF_CHANNELS
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

                Command::AddConnection(connection) => self.graph.add_connection(connection),
                Command::RemoveConnection(connection) => self.graph.remove_connection(connection),
                Command::ConnectToOutput(output_connection) => {
                    self.graph.connect_to_output(output_connection)
                }
            }
        }
    }

    fn send_notficiation(&mut self, notification: Notification) {
        let _ = self.notification_tx.send(notification);
    }

    fn update_position(&mut self, num_samples: usize) {
        self.sample_position += num_samples;
    }

    fn current_time(&self) -> Timestamp {
        Timestamp::from_seconds(self.sample_position as f64 / self.sample_rate as f64)
    }

    fn notify_position(&mut self) {
        self.send_notficiation(Notification::Position(self.current_time()));
    }
}
