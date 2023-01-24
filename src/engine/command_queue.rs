use crate::commands::Command;

pub trait CommandQueue {
    fn send(&self, command: Command);
}
