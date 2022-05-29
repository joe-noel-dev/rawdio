use std::sync::{atomic::AtomicI64, Arc};

use crate::{
    audio_process::AudioProcess, commands::command::Command, realtime::processor::Processor, timestamp::Timestamp,
};

use lockfree::channel::mpsc::{self, Sender};

pub struct Context {
    sample_rate: usize,
    timestamp: Arc<AtomicI64>,
    command_tx: Sender<Command>,
    realtime_processor: Option<Processor>,
}

impl Context {
    pub fn new(sample_rate: usize) -> Self {
        let (command_tx, command_rx) = mpsc::create();
        let timestamp = Arc::new(AtomicI64::new(0));
        Self {
            sample_rate,
            timestamp: timestamp.clone(),
            command_tx,
            realtime_processor: Some(Processor::new(sample_rate, command_rx, Arc::clone(&timestamp))),
        }
    }

    pub fn start(&mut self) {
        let _ = self.command_tx.send(Command::Start);
    }

    pub fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
    }

    pub fn current_time(&self) -> Timestamp {
        Timestamp::from_raw_i64(self.timestamp.load(std::sync::atomic::Ordering::Acquire))
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

    pub fn get_command_queue(&self) -> Sender<Command> {
        self.command_tx.clone()
    }
}
