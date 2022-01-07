use std::sync::{atomic::Ordering, Arc};

use lockfree::channel::mpsc::Sender;

use crate::commands::{command::Command, id::Id, parameter_command::ParameterCommand};
use atomic_float::AtomicF32;

pub type ParameterValue = Arc<AtomicF32>;

#[derive(Clone)]
pub struct AudioParameter {
    id: Id,
    value: ParameterValue,
    command_queue: Sender<Command>,
}

pub struct RealtimeAudioParameter {
    id: Id,
    value: ParameterValue,
}

impl AudioParameter {
    pub fn new(initial_value: f32, command_queue: Sender<Command>) -> Self {
        let id = Id::generate();
        let param_value = ParameterValue::new(AtomicF32::new(initial_value));
        let realtime_audio_param = RealtimeAudioParameter::new(id, param_value.clone());

        let _ = command_queue.send(Command::ParameterCommand(ParameterCommand::Add(
            realtime_audio_param,
        )));

        Self {
            id,
            value: param_value,
            command_queue,
        }
    }

    pub fn get_value(&self) -> ParameterValue {
        self.value.clone()
    }

    pub fn set_value_immediate(&mut self, value: f32) {
        let _ = self.command_queue.send(Command::ParameterCommand(
            ParameterCommand::SetValueImmediate((self.id, value)),
        ));
    }
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

    pub fn get_value(&self) -> f32 {
        self.value.load(Ordering::Acquire)
    }

    pub fn set_value(&mut self, value: f32) {
        self.value.store(value, Ordering::Release)
    }
}
