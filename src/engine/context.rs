use crate::{CommandQueue, Timestamp};

#[derive(PartialEq)]
pub enum NotifierStatus {
    Continue,
    Remove,
}

/// The root audio context
pub trait Context {
    /// Start the audio context
    ///
    /// Before this is called, audio will not be process and time will not
    /// advance
    fn start(&mut self);

    /// Stop the audio context
    fn stop(&mut self);

    /// Get the current time in the audio context
    ///
    /// If running the audio process in a different thread, the time may advance
    /// between asking for the time and getting the time. If this is the case,
    /// some 'lookahead' may be required.
    fn current_time(&self) -> Timestamp;

    /// Get the sample rate of the audio context
    fn get_sample_rate(&self) -> usize;

    /// Get the command queue to send commands to the context
    fn get_command_queue(&self) -> Box<dyn CommandQueue>;

    /// Add a notifier that will be given an opportunity to get notifications
    /// after every audio block
    fn add_notifier(&mut self, notifier: Box<dyn Fn() -> NotifierStatus>);

    /// Generate all notifications
    fn process_notifications(&mut self);
}
