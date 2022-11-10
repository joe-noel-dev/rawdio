use std::sync::{atomic::AtomicI64, Arc};

use crate::{
    audio_process::AudioProcess, realtime::Processor, Command, CommandQueue, Context, Timestamp,
};

use lockfree::channel::mpsc;

pub struct Engine {
    sample_rate: usize,
    timestamp: Arc<AtomicI64>,
    command_tx: CommandQueue,
    realtime_processor: Option<Processor>,
}

impl Engine {
    pub fn new(sample_rate: usize) -> Self {
        let (command_tx, command_rx) = mpsc::create();
        let timestamp = Arc::new(AtomicI64::new(0));
        Self {
            sample_rate,
            timestamp: timestamp.clone(),
            command_tx,
            realtime_processor: Some(Processor::new(
                sample_rate,
                command_rx,
                Arc::clone(&timestamp),
            )),
        }
    }
}

impl Context for Engine {
    fn start(&mut self) {
        let _ = self.command_tx.send(Command::Start);
    }

    fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
    }

    fn current_time(&self) -> Timestamp {
        Timestamp::from_raw_i64(self.timestamp.load(std::sync::atomic::Ordering::Acquire))
    }

    fn get_audio_process(&mut self) -> Box<dyn AudioProcess + Send> {
        let mut other = None;
        std::mem::swap(&mut self.realtime_processor, &mut other);
        assert!(other.is_some());
        Box::new(other.unwrap())
    }

    fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_tx.clone()
    }
}

pub fn create_context(sample_rate: usize) -> Box<dyn Context> {
    Box::new(Engine::new(sample_rate))
}
