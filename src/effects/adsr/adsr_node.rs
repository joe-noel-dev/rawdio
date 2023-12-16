use super::{
    adsr_event::{AdsrEvent, AdsrEventType},
    adsr_processor::AdsrProcessor,
};
use crate::{
    commands::Id, effects::Channel, graph::DspParameters, Context, GraphNode, Level, Timestamp,
};
use std::time::Duration;

/// An ADSR node that will create an envelope to modulate the amplitude of its input
pub struct Adsr {
    /// The node that can be routed into the audio graph
    pub node: GraphNode,
    event_transmitter: Channel::Sender<AdsrEvent>,
}

static EVENT_CHANNEL_CAPACITY: usize = 32;

impl Adsr {
    /// Create a new ADSR node
    pub fn new(context: &dyn Context, channel_count: usize, sample_rate: usize) -> Self {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = Channel::bounded(EVENT_CHANNEL_CAPACITY);

        let processor = Box::new(AdsrProcessor::new(
            event_receiver,
            sample_rate,
            context.maximum_frame_count(),
        ));

        Self {
            node: GraphNode::new(
                id,
                context,
                channel_count,
                channel_count,
                processor,
                DspParameters::empty(),
            ),
            event_transmitter,
        }
    }

    /// Trigger a note on at a particular time
    pub fn note_on_at_time(&mut self, time: Timestamp) {
        let _ = self
            .event_transmitter
            .send(AdsrEvent::new(time, AdsrEventType::NoteOn));
    }

    /// Trigger a note off at a particular time
    pub fn note_off_at_time(&mut self, time: Timestamp) {
        let _ = self
            .event_transmitter
            .send(AdsrEvent::new(time, AdsrEventType::NoteOff));
    }

    /// Set the attack time
    ///
    /// This is the time from the note on until the gain reaches unity
    pub fn set_attack_time(&mut self, attack_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent::new(
            Timestamp::zero(),
            AdsrEventType::SetAttack(attack_time),
        ));
    }

    /// Set the decay time
    ///
    /// This is the time after the end of the attack time to reach the steady-state sustain level
    pub fn set_decay_time(&mut self, decay_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent::new(
            Timestamp::zero(),
            AdsrEventType::SetDecay(decay_time),
        ));
    }

    /// Set the sustain level
    ///
    /// This is the level that will be sustained after the attack and decay phase
    /// until a note off
    pub fn set_sustain_level(&mut self, sustain_level: Level) {
        let _ = self.event_transmitter.send(AdsrEvent::new(
            Timestamp::zero(),
            AdsrEventType::SetSustain(sustain_level),
        ));
    }

    /// Set the release time
    ///
    /// This is the time after a note off until gain of 0 is reached
    pub fn set_release_time(&mut self, release_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent::new(
            Timestamp::zero(),
            AdsrEventType::SetRelease(release_time),
        ));
    }

    /// Convenience method to set the attack, decay, sustain, and release in one go
    pub fn set_adsr(
        &mut self,
        attack_time: Duration,
        decay_time: Duration,
        sustain_level: Level,
        release_time: Duration,
    ) {
        self.set_attack_time(attack_time);
        self.set_decay_time(decay_time);
        self.set_sustain_level(sustain_level);
        self.set_release_time(release_time);
    }
}
