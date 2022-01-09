use crate::commands::{command::Command, id::Id};

use super::dsp::{Dsp, DspParameterMap, DspProcessor};
use lockfree::channel::mpsc::Sender;

pub trait Node {
    fn get_id(&self) -> Id;
}

pub fn add_dsp(
    id: Id,
    processor: Box<dyn DspProcessor + Send + Sync>,
    parameters: DspParameterMap,
    command_queue: Sender<Command>,
) {
    let dsp = Dsp::new(id, processor, parameters);
    let _ = command_queue.send(Command::AddDsp(Box::new(dsp)));
}

pub fn remove_dsp(id: Id, command_queue: Sender<Command>) {
    let _ = command_queue.send(Command::RemoveDsp(id));
}
