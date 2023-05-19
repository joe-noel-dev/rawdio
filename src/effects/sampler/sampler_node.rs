use crate::{
    commands::Id, effects::Channel, graph::DspParameters, AudioBuffer, Context, GraphNode,
    OwnedAudioBuffer, Timestamp,
};

use super::{
    sampler_event::SamplerEvent,
    sampler_processor::{EventTransmitter, SamplerDspProcess},
};

/// A node that can play or loop a sample
pub struct Sampler {
    /// The node to connect to the audio graph
    pub node: GraphNode,
    event_transmitter: EventTransmitter,
}

static EVENT_CHANNEL_CAPACITY: usize = 32;

impl Sampler {
    /// Create a Sampler with a specified capacity of events in the event queue
    pub fn new_with_event_capacity(
        context: &dyn Context,
        sample: OwnedAudioBuffer,
        capacity: usize,
    ) -> Self {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = Channel::bounded(capacity);

        let input_count = 0;
        let output_count = sample.channel_count();
        let sample_rate = sample.sample_rate();
        let processor = Box::new(SamplerDspProcess::new(sample_rate, sample, event_receiver));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            input_count,
            output_count,
            processor,
            DspParameters::empty(),
        );

        Self {
            node,
            event_transmitter,
        }
    }

    /// Create a new sampler with the given sample
    pub fn new(context: &dyn Context, sample: OwnedAudioBuffer) -> Self {
        Self::new_with_event_capacity(context, sample, EVENT_CHANNEL_CAPACITY)
    }

    /// Start the sampler playing
    pub fn start_now(&mut self) {
        self.send_event(SamplerEvent::start_now());
    }

    /// Stop the sampler playing
    pub fn stop_now(&mut self) {
        self.send_event(SamplerEvent::stop_now());
    }

    /// Start from the specified time, at the specified position in the sample
    pub fn start_from_position_at_time(
        &mut self,
        start_time: Timestamp,
        position_in_sample: Timestamp,
    ) {
        self.send_event(SamplerEvent::start(start_time, position_in_sample));
    }

    /// Stop at the specified time
    pub fn stop_at_time(&mut self, stop_time: Timestamp) {
        self.send_event(SamplerEvent::stop(stop_time));
    }

    /// Enable looping
    ///
    /// # Arguments
    ///
    /// * `loop_start` - The position in the sample to start the loop
    /// * `loop_end` - The position in the sample to go back to `loop_start`
    pub fn enable_loop(&mut self, loop_start: Timestamp, loop_end: Timestamp) {
        self.send_event(SamplerEvent::enable_loop(loop_start, loop_end));
    }

    /// Cancel a loop
    ///
    /// This will clear the loop points and finish when it reaches the end of the sample
    pub fn cancel_loop(&mut self) {
        self.send_event(SamplerEvent::cancel_loop());
    }

    /// Cancel all scheduled events that haven't occurred yet
    pub fn cancel_all(&mut self) {
        self.send_event(SamplerEvent::cancel_all());
    }

    /// Enable loop at a time
    ///
    /// # Arguments
    ///
    /// * `enable_at_time` - The time to enable loop points
    /// * `loop_start` - The position in the loop to start looping
    /// * `loop_end` - The position in the loop to go back to the start
    pub fn enable_loop_at_time(
        &mut self,
        enable_at_time: Timestamp,
        loop_start: Timestamp,
        loop_end: Timestamp,
    ) {
        self.send_event(SamplerEvent::enable_loop_at_time(
            enable_at_time,
            loop_start,
            loop_end,
        ));
    }

    /// Cancel the loop at a time
    pub fn cancel_loop_at_time(&mut self, cancel_time: Timestamp) {
        self.send_event(SamplerEvent::cancel_loop_at_time(cancel_time));
    }

    fn send_event(&mut self, event: SamplerEvent) {
        println!("Sending {event:?}");
        debug_assert!(!self.event_transmitter.is_full());
        let _ = self.event_transmitter.send(event);
    }
}
