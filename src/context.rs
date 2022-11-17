use crate::{CommandQueue, Timestamp};

pub trait Context {
    fn start(&mut self);

    fn stop(&mut self);

    fn current_time(&self) -> Timestamp;

    fn get_sample_rate(&self) -> usize;

    fn get_command_queue(&self) -> CommandQueue;
}
