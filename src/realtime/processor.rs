use crate::{
    audio_process::AudioProcess,
    commands::{
        command::Command, id::Id, notification::Notification, parameter_command::ParameterCommand,
    },
    graph::dsp::Dsp,
    parameter::RealtimeAudioParameter,
    timestamp::Timestamp,
    utility::{audio_buffer::AudioBuffer, pool::Pool},
};
use lockfree::channel::{mpsc::Receiver, spsc::Sender};

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
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
        Self {
            started: false,
            sample_rate,
            command_rx,
            notification_tx,
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

        if let Some(dsp) = self.dsps.get_mut(&Id::with_value(0)) {
            (dsp.process)(data);
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
                Command::ParameterCommand(parameter_command) => {
                    self.handle_parameter_command(parameter_command)
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

    fn notify_position(&mut self) {
        let timestamp =
            Timestamp::with_seconds(self.sample_position as f64 / self.sample_rate as f64);
        self.send_notficiation(Notification::Position(timestamp));
    }

    fn add_dsp(&mut self, dsp: Dsp) {
        self.dsps.add(dsp.get_id(), Box::new(dsp));
    }

    fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.dsps.remove(&id) {
            self.send_notficiation(Notification::DisposeDsp(*dsp));
        }
    }

    fn handle_parameter_command(&mut self, parameter_command: ParameterCommand) {
        match parameter_command {
            ParameterCommand::Add(parameter) => self.add_parameter(parameter),
            ParameterCommand::SetValueImmediate((id, value)) => {
                self.set_parameter_value_immediate(id, value)
            }
        }
    }

    fn add_parameter(&mut self, audio_parameter: RealtimeAudioParameter) {
        self.parameters
            .add(audio_parameter.get_id(), Box::new(audio_parameter));
    }

    fn set_parameter_value_immediate(&mut self, id: Id, value: f32) {
        if let Some(parameter) = self.parameters.get_mut(&id) {
            parameter.set_value(value);
        }
    }
}
