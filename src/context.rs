use crate::{
    audio_process::AudioProcess,
    commands::{command::Command, notification::Notification},
    graph::node::Node,
    realtime::processor::Processor,
    timestamp::Timestamp,
};

use lockfree::channel::{
    mpsc::{self, Sender},
    spsc::{self, Receiver},
};

pub struct Context {
    sample_rate: usize,
    timestamp: Timestamp,
    command_tx: Sender<Command>,
    notification_rx: Receiver<Notification>,
    realtime_processor: Option<Processor>,
}

impl Context {
    pub fn new(sample_rate: usize) -> Self {
        let (command_tx, command_rx) = mpsc::create();
        let (notification_tx, notification_rx) = spsc::create();

        Self {
            sample_rate,
            timestamp: Timestamp::default(),
            command_tx,
            notification_rx,
            realtime_processor: Some(Processor::new(sample_rate, command_rx, notification_tx)),
        }
    }

    pub fn start(&mut self) {
        let _ = self.command_tx.send(Command::Start);
    }

    pub fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
    }

    pub fn current_time(&self) -> Timestamp {
        self.timestamp
    }

    pub fn get_audio_process(&mut self) -> Box<dyn AudioProcess + Send> {
        let mut other = None;
        std::mem::swap(&mut self.realtime_processor, &mut other);
        assert!(other.is_some());
        Box::new(other.unwrap())
    }

    pub fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn process_notifications(&mut self) {
        while let Ok(notification) = self.notification_rx.recv() {
            match notification {
                Notification::Position(timestamp) => self.timestamp = timestamp,
            }
        }
    }

    pub fn get_command_queue(&self) -> Sender<Command> {
        self.command_tx.clone()
    }

    pub fn connect_to_output(&mut self, _source_node: &dyn Node) {}
}
