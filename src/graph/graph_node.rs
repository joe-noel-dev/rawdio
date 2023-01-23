use crate::{commands::Id, Command, CommandQueue};

use super::{
    connection::Connection,
    endpoint::{Endpoint, EndpointType},
    Dsp, DspParameters, DspProcessor,
};

pub struct GraphNode {
    id: Id,
    command_queue: CommandQueue,
}

impl GraphNode {
    pub fn new(
        id: Id,
        command_queue: CommandQueue,
        input_count: usize,
        output_count: usize,
        processor: Box<dyn DspProcessor + Send + Sync>,
        parameters: DspParameters,
    ) -> Self {
        let dsp = Dsp::new(id, input_count, output_count, processor, parameters);

        dsp.add_to_audio_process(&command_queue);

        Self { id, command_queue }
    }

    fn get_id(&self) -> Id {
        self.id
    }

    pub fn connect_to_input(&self) {
        let _ = self
            .command_queue
            .send(Command::ConnectToInput(Endpoint::new(
                self.get_id(),
                EndpointType::Input,
            )));
    }

    pub fn connect_to_output(&self) {
        let _ = self
            .command_queue
            .send(Command::ConnectToOutput(Endpoint::new(
                self.get_id(),
                EndpointType::Output,
            )));
    }

    fn connect_to_id(&self, id: Id) {
        let _ = self
            .command_queue
            .send(Command::AddConnection(Connection::new(self.get_id(), id)));
    }

    pub fn connect_to(&self, node: &GraphNode) {
        self.connect_to_id(node.get_id());
    }

    fn disconnect_from_id(&self, id: Id) {
        let _ = self
            .command_queue
            .send(Command::RemoveConnection(Connection::new(
                self.get_id(),
                id,
            )));
    }

    pub fn disconnect_from_node(&self, node: &GraphNode) {
        self.disconnect_from_id(node.get_id());
    }
}

impl Drop for GraphNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue);
    }
}
