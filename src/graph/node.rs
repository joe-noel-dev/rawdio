use crate::commands::{command::Command, id::Id};
use lockfree::channel::mpsc::Sender;

use super::connection::{Connection, OutputConnection};

pub trait Node {
    fn get_id(&self) -> Id;

    fn get_command_queue(&self) -> Sender<Command>;

    fn connect_to_output(&self) {
        let _ = self
            .get_command_queue()
            .send(Command::ConnectToOutput(OutputConnection {
                from_id: self.get_id(),
                output_index: 0,
            }));
    }

    fn connect_to(&self, id: Id) {
        let _ = self
            .get_command_queue()
            .send(Command::AddConnection(Connection {
                from_id: self.get_id(),
                output_index: 0,
                to_id: id,
                input_index: 0,
            }));
    }

    fn disconnect_from(&self, id: Id) {
        let _ = self
            .get_command_queue()
            .send(Command::AddConnection(Connection {
                from_id: self.get_id(),
                output_index: 0,
                to_id: id,
                input_index: 0,
            }));
    }
}
