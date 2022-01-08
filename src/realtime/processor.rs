use crate::{
    audio_process::AudioProcess,
    commands::{command::Command, id::Id, notification::Notification},
    graph::dsp::Dsp,
    parameter::RealtimeAudioParameter,
    timestamp::Timestamp,
    utility::{audio_buffer::AudioBuffer, pool::Pool},
};
use lockfree::channel::{
    mpsc::Receiver,
    spsc::{self, Sender},
};

use super::garbage_collector::{run_garbage_collector, GarbageCollectionCommand};

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
    sample_position: usize,
    dsps: Pool<Id, Dsp>,
    parameters: Pool<Id, RealtimeAudioParameter>,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        command_rx: Receiver<Command>,
        notification_tx: Sender<Notification>,
    ) -> Self {
        let (garbase_collection_tx, garbage_collection_rx) = spsc::create();

        run_garbage_collector(garbage_collection_rx);

        Self {
            started: false,
            sample_rate,
            command_rx,
            notification_tx,
            garbase_collection_tx,
            sample_position: 0,

            dsps: Pool::new(64),
            parameters: Pool::new(8192),
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

        let current_time = self.current_time();
        for (_, parameter) in self.parameters.all_mut() {
            parameter.jump_to_time(&current_time);
        }

        for (_, dsp) in self.dsps.all_mut() {
            (dsp.process)(data, &current_time);
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
                Command::AddDsp(dsp) => self.add_dsp(dsp),
                Command::RemoveDsp(id) => self.remove_dsp(id),
                Command::AddParameter(audio_parameter) => self.add_parameter(audio_parameter),
                Command::RemoveParameter(id) => self.remove_parameter(id),
                Command::SetValueImmediate((id, value)) => {
                    self.set_parameter_value_immediate(id, value)
                }
                Command::LinearRampToValue((id, value, time)) => {
                    self.linear_ramp_to_value(id, value, time)
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

    fn add_dsp(&mut self, dsp: Box<Dsp>) {
        self.dsps.add(dsp.get_id(), dsp);
    }

    fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.dsps.remove(&id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }
    }

    fn add_parameter(&mut self, mut audio_parameter: Box<RealtimeAudioParameter>) {
        audio_parameter.jump_to_time(&self.current_time());
        self.parameters
            .add(audio_parameter.get_id(), audio_parameter);
    }

    fn remove_parameter(&mut self, id: Id) {
        if let Some(parameter) = self.parameters.remove(&id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeParameter(parameter));
        }
    }

    fn set_parameter_value_immediate(&mut self, id: Id, value: f64) {
        if let Some(parameter) = self.parameters.get_mut(&id) {
            parameter.set_value(value);
        }
    }

    fn linear_ramp_to_value(&mut self, id: Id, value: f64, time: Timestamp) {
        if let Some(parameter) = self.parameters.get_mut(&id) {
            parameter.add_parameter_change(value, time);
        }
    }
}
