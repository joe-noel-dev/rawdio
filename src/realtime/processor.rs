use crate::{
    audio_process::AudioProcess,
    commands::{command::Command, id::Id, notification::Notification},
    graph::dsp::Dsp,
    sources::realtime_oscillator::RealtimeOscillator,
    timestamp::Timestamp,
    utility::{audio_buffer::AudioBuffer, pool::Pool},
};
use lockfree::channel::spsc::{Receiver, Sender};

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
    sample_position: usize,
    oscillators: Pool<Id, RealtimeOscillator>,
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

            oscillators: Pool::new(64),
        }
    }
}

impl AudioProcess for Processor {
    fn process(&mut self, data: &mut dyn AudioBuffer) {
        data.clear();

        self.process_commands();

        if !self.started {
            return;
        }

        if let Some(osc) = self.oscillators.get_mut(&Id::with_value(0)) {
            osc.process(data);
        }

        self.update_position(data.num_frames());
        self.notify_position();
    }
}

impl Processor {
    fn process_commands(&mut self) {
        while let Ok(command) = self.command_rx.recv() {
            match command {
                Command::Start => self.started = true,
                Command::Stop => self.started = false,
                Command::AddOscillator(osc) => self.add_oscillator(osc),
                Command::RemoveOscillator(id) => self.remove_oscillator(id),
            }
        }
    }

    fn send_notficiation(&mut self, notification: Notification) {
        let _ = self.notification_tx.send(notification);
    }

    fn update_position(&mut self, num_samples: usize) {
        self.sample_position += num_samples;
    }

    fn notify_position(&mut self) {
        let timestamp =
            Timestamp::with_seconds(self.sample_position as f64 / self.sample_rate as f64);
        self.send_notficiation(Notification::Position(timestamp));
    }

    fn add_oscillator(&mut self, osc: RealtimeOscillator) {
        self.oscillators.add(osc.get_id(), Box::new(osc));
    }

    fn remove_oscillator(&mut self, id: Id) {
        if let Some(osc) = self.oscillators.remove(&id) {
            self.send_notficiation(Notification::DisposeOscillator(*osc));
        }
    }
}
