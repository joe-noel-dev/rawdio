use crate::{CommandQueue, Timestamp};

#[derive(PartialEq)]
pub enum NotifierStatus {
    Continue,
    Remove,
}

pub trait Context {
    fn start(&mut self);

    fn stop(&mut self);

    fn current_time(&self) -> Timestamp;

    fn get_sample_rate(&self) -> usize;

    fn get_command_queue(&self) -> Box<dyn CommandQueue>;

    fn add_notifier(&mut self, notifier: Box<dyn Fn() -> NotifierStatus>);

    fn process_notifications(&mut self);
}
