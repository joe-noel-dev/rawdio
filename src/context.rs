use crate::{
    node::Node,
    realtime::{command::Command, notification::Notification, processor::Processor},
    realtime_context::RealtimeContext,
    timestamp::{self, Timestamp},
};

use lockfree::channel::spsc::{self, Receiver, Sender};

pub struct Context {
    sample_rate: usize,
    timestamp: Timestamp,
    command_tx: Sender<Command>,
    notification_rx: Receiver<Notification>,
    realtime_processor: Option<Box<dyn RealtimeContext + Send>>,
}

impl Context {
    pub fn new(sample_rate: usize) -> Self {
        let (command_tx, command_rx) = spsc::create();
        let (notification_tx, notification_rx) = spsc::create();

        Self {
            sample_rate,
            timestamp: Timestamp::default(),
            command_tx,
            notification_rx,
            realtime_processor: Some(Box::new(Processor::new(
                sample_rate,
                command_rx,
                notification_tx,
            ))),
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

    pub fn get_realtime_context(&mut self) -> Box<dyn RealtimeContext + Send> {
        let mut other = None;
        std::mem::swap(&mut self.realtime_processor, &mut other);
        assert!(other.is_some());
        other.unwrap()
    }

    pub fn process_notifications(&mut self) {
        while let Ok(notification) = self.notification_rx.recv() {
            match notification {
                Notification::Position(timestamp) => self.timestamp = timestamp,
            }
        }
    }

    pub fn connect_to_output(&mut self, _source_node: &dyn Node) {}
}
