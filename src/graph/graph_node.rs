use crate::{commands::Id, Command, CommandQueue, Context};

use super::{
    connection::Connection,
    endpoint::{Endpoint, EndpointType},
    Dsp, DspParameters, DspProcessor,
};

/// A node the connects into the audio graph
pub struct GraphNode {
    id: Id,
    command_queue: Box<dyn CommandQueue>,
}

impl GraphNode {
    /// Create a new audio graph
    pub fn new(
        id: Id,
        context: &dyn Context,
        input_count: usize,
        output_count: usize,
        processor: Box<dyn DspProcessor + Send + Sync>,
        parameters: DspParameters,
    ) -> Self {
        let dsp = Dsp::new(id, input_count, output_count, processor, parameters);

        let command_queue = context.get_command_queue();

        dsp.add_to_audio_process(command_queue.as_ref());

        Self { id, command_queue }
    }

    fn get_id(&self) -> Id {
        self.id
    }

    /// Connect this node to the system input
    ///
    /// Only one node can be connected to the system input.
    /// If this is called on more than one node, the second connection will
    /// replace the first.
    pub fn connect_to_input(&self) {
        self.command_queue
            .send(Command::ConnectToInput(Endpoint::new(
                self.get_id(),
                EndpointType::Input,
            )));
    }

    /// Connect this node to the system output
    ///
    /// Only one node can be connected to the system output.
    /// If this is called on more than one node, the second connection will
    /// replace the first.
    ///
    /// If you need to connect more than one node to the output, you can
    /// connect all nodes to another node (e.g. gain), and connect that node to
    /// the output.
    pub fn connect_to_output(&self) {
        self.command_queue
            .send(Command::ConnectToOutput(Endpoint::new(
                self.get_id(),
                EndpointType::Output,
            )));
    }

    fn connect_to_id(&self, id: Id) {
        self.command_queue
            .send(Command::AddConnection(Connection::new(self.get_id(), id)));
    }

    /// Connect the output of this node to the input of another node
    pub fn connect_to(&self, node: &GraphNode) {
        self.connect_to_id(node.get_id());
    }

    fn disconnect_from_id(&self, id: Id) {
        self.command_queue
            .send(Command::RemoveConnection(Connection::new(
                self.get_id(),
                id,
            )));
    }

    /// Disconnect the output of this node from the input of another node
    pub fn disconnect_from_node(&self, node: &GraphNode) {
        self.disconnect_from_id(node.get_id());
    }
}

impl Drop for GraphNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, self.command_queue.as_ref());
    }
}
