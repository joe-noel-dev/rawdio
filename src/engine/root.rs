use std::sync::{atomic::AtomicI64, Arc};

use crate::{realtime::Processor, AudioProcess, Command, CommandQueue, Context, Timestamp};

use super::context::NotifierStatus;

pub struct Root {
    sample_rate: usize,
    timestamp: Arc<AtomicI64>,
    command_transmitter: CommandTransmitter,
    notifiers: Vec<Box<dyn Fn() -> NotifierStatus>>,
}

impl Context for Root {
    fn start(&mut self) {
        self.command_transmitter.send(Command::Start);
    }

    fn stop(&mut self) {
        self.command_transmitter.send(Command::Stop);
    }

    fn current_time(&self) -> Timestamp {
        Timestamp::from_raw_i64(self.timestamp.load(std::sync::atomic::Ordering::Acquire))
    }

    fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn get_command_queue(&self) -> Box<dyn CommandQueue> {
        Box::new(self.command_transmitter.clone())
    }

    fn add_notifier(&mut self, notifier: Box<dyn Fn() -> NotifierStatus>) {
        self.notifiers.push(notifier);
    }

    fn process_notifications(&mut self) {
        self.notifiers
            .retain(|notifier| (notifier)() == NotifierStatus::Continue);
    }
}

pub fn create_engine(sample_rate: usize) -> (Box<dyn Context>, Box<dyn AudioProcess + Send>) {
    let (command_transmitter, command_receiver) = CommandTransmitter::new();
    let timestamp = Arc::new(AtomicI64::new(0));
    let processor = Box::new(Processor::new(
        sample_rate,
        command_receiver,
        Arc::clone(&timestamp),
    ));

    let engine = Box::new(Root {
        sample_rate,
        timestamp,
        command_transmitter,
        notifiers: Vec::new(),
    });

    (engine, processor)
}

#[derive(Clone)]
struct CommandTransmitter {
    command_tx: crossbeam::channel::Sender<Command>,
}

impl CommandTransmitter {
    fn new() -> (Self, crossbeam::channel::Receiver<Command>) {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        (Self { command_tx }, command_rx)
    }
}

impl CommandQueue for CommandTransmitter {
    fn send(&self, command: Command) {
        let _ = self.command_tx.send(command);
    }
}
