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

/// Options to control the engine
pub struct EngineOptions {
    sample_rate: usize,
    maximum_channel_count: usize,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            maximum_channel_count: 2,
        }
    }
}

impl EngineOptions {
    /// Specify the sample rate of the engine
    pub fn with_sample_rate(mut self, sample_rate: usize) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Specify the maximum channel count between any two nodes in the engine
    ///
    /// A bigger channel count will use more memory
    pub fn with_maximum_channel_count(mut self, maximum_channel_count: usize) -> Self {
        self.maximum_channel_count = maximum_channel_count;
        self
    }
}

/// Create an engine with the default options
///
/// This returns a pair:
///
/// * The `Context` is the root context. This will be required to create most
///   nodes and should be kept in scope for the lifetime of the application
///
/// * The `AudioProcess` is used to generate audio. This might be passed to a
///   different thread if used in a realtime context, or it might be kept in
///   the main thread if used offline.
pub fn create_engine() -> (Box<dyn Context>, Box<dyn AudioProcess + Send>) {
    create_engine_with_options(EngineOptions::default())
}

/// Create an audio engine with custom options
///
/// This returns a pair:
///
/// * The `Context` is the root context. This will be required to create most
///   nodes and should be kept in scope for the lifetime of the application
///
/// * The `AudioProcess` is used to generate audio. This might be passed to a
///   different thread if used in a realtime context, or it might be kept in
///   the main thread if used offline.
pub fn create_engine_with_options(
    options: EngineOptions,
) -> (Box<dyn Context>, Box<dyn AudioProcess + Send>) {
    let (command_transmitter, command_receiver) = CommandTransmitter::new();

    let timestamp = Arc::new(AtomicI64::new(0));

    let processor = Box::new(Processor::new(
        options.sample_rate,
        options.maximum_channel_count,
        command_receiver,
        Arc::clone(&timestamp),
    ));

    let engine = Box::new(Root {
        sample_rate: options.sample_rate,
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
