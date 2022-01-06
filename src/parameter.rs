use lockfree::channel::mpsc::Sender;

use crate::commands::{command::Command, id::Id, parameter_command::ParameterCommand};

#[derive(Clone, Copy)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
}

#[derive(Clone)]
pub struct AudioParameter {
    id: Id,
    value: ParameterValue,
    command_queue: Sender<Command>,
}

pub fn create(
    initial_value: ParameterValue,
    command_queue: Sender<Command>,
) -> (AudioParameter, RealtimeAudioParameter) {
    let id = Id::generate();
    let audio_param = AudioParameter::new(id, initial_value, command_queue);
    let realtime_audio_param = RealtimeAudioParameter::new(id, initial_value);
    (audio_param, realtime_audio_param)
}

impl AudioParameter {
    pub fn new(id: Id, initial_value: ParameterValue, command_queue: Sender<Command>) -> Self {
        Self {
            id,
            value: initial_value,
            command_queue,
        }
    }
}

pub struct RealtimeAudioParameter {
    id: Id,
    value: ParameterValue,
}

impl RealtimeAudioParameter {
    pub fn new(id: Id, initial_value: ParameterValue) -> Self {
        Self {
            id,
            value: initial_value,
        }
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn process_command(&mut self, command: ParameterCommand) {}

    pub fn float_value(&self) -> f32 {
        if let ParameterValue::Float(value) = self.value {
            return value;
        }

        panic!("Invalid parameter type");
    }
}
