use crate::commands::{command::Command, id::Id};
use lockfree::channel::mpsc::Sender;

use super::{connection::Connection, endpoint::Endpoint};

pub trait Node {
    fn get_id(&self) -> Id;

    fn get_command_queue(&self) -> Sender<Command>;

    fn connect_to_output(&self) {
        let _ = self
            .get_command_queue()
            .send(Command::ConnectToOutput(Endpoint::new(self.get_id())));
    }

    fn connect_to(&self, id: Id) {
        let _ = self
            .get_command_queue()
            .send(Command::AddConnection(Connection::new(self.get_id(), id)));
    }

    fn disconnect_from(&self, id: Id) {
        let _ = self
            .get_command_queue()
            .send(Command::RemoveConnection(Connection::new(
                self.get_id(),
                id,
            )));
    }
}
