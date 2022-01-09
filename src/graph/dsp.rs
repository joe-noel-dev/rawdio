use std::collections::HashMap;

use crate::{
    buffer::audio_buffer::AudioBuffer,
    commands::{
        command::{Command, ParameterChangeRequest},
        id::Id,
    },
    parameter::RealtimeAudioParameter,
    timestamp::Timestamp,
};

use super::connection::Connection;

use lockfree::channel::mpsc::Sender;

pub type DspParameterMap = HashMap<Id, RealtimeAudioParameter>;

pub struct Dsp {
    id: Id,
    processor: Box<dyn DspProcessor + Send + Sync>,
    parameters: DspParameterMap,
    connections: Vec<Option<Connection>>,
}

pub trait DspProcessor {
    fn process_audio(
        &mut self,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameterMap,
    );
}

const EMPTY_CONNECTION: Option<Connection> = None;

impl Dsp {
    pub fn new(
        id: Id,
        processor: Box<dyn DspProcessor + Send + Sync>,
        parameters: DspParameterMap,
        number_of_outputs: usize,
    ) -> Self {
        Self {
            id,
            processor,
            parameters,
            connections: vec![EMPTY_CONNECTION; number_of_outputs],
        }
    }

    pub fn add_to_audio_process(dsp: Self, command_queue: &Sender<Command>) {
        let _ = command_queue.send(Command::AddDsp(Box::new(dsp)));
    }

    pub fn remove_from_audio_process(id: Id, command_queue: &Sender<Command>) {
        let _ = command_queue.send(Command::RemoveDsp(id));
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn process_audio(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        for (_, parameter) in self.parameters.iter_mut() {
            parameter.set_current_time(*start_time);
        }

        self.processor
            .process_audio(output_buffer, start_time, &self.parameters);
    }

    pub fn request_parameter_change(&mut self, parameter_change: ParameterChangeRequest) {
        if let Some(parameter) = self.parameters.get_mut(&parameter_change.parameter_id) {
            parameter.add_parameter_change(parameter_change.change)
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
        assert!(connection.output_index < self.connections.len());
        assert!(connection.from_id == self.id);

        let _ = std::mem::replace(
            &mut self.connections[connection.output_index],
            Some(connection),
        );
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        assert!(connection.output_index < self.connections.len());
        assert!(connection.from_id == self.id);

        let existing_connection = &self.connections[connection.output_index];
        if let Some(existing_connection) = existing_connection {
            if *existing_connection == connection {
                let _ = std::mem::replace(&mut self.connections[connection.output_index], None);
            }
        }
    }
}
