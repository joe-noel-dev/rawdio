use crate::{realtime_context::RealtimeContext, timestamp::Timestamp};
use lockfree::channel::spsc::{Receiver, Sender};

use super::{command::Command, notification::Notification};

pub struct Processor {
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
    sample_position: usize,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        command_rx: Receiver<Command>,
        notification_tx: Sender<Notification>,
    ) -> Self {
        Self {
            sample_rate,
            command_rx,
            notification_tx,
            sample_position: 0,
        }
    }
}

impl RealtimeContext for Processor {
    fn process(&mut self, data: &mut [f32], num_channels: usize) {
        self.process_commands();
        Self::clear_output(data);
        self.sample_position += data.len() / num_channels;
        self.notify_position();
    }
}

impl Processor {
    fn process_commands(&mut self) {
        while let Ok(command) = self.command_rx.recv() {
            match command {
                Command::Start => println!("Received start"),
                Command::Stop => println!("Received stop"),
            }
        }
    }

    fn clear_output(data: &mut [f32]) {
        data.fill(0.0);
    }

    fn notify_position(&mut self) {
        let timestamp =
            Timestamp::with_seconds(self.sample_position as f64 / self.sample_rate as f64);
        let _ = self.notification_tx.send(Notification::Position(timestamp));
    }
}
