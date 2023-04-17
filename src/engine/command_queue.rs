use crate::commands::Command;

/// Something that can process commands
pub trait CommandQueue {
    /// Send a command
    fn send(&self, command: Command);
}
