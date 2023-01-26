use super::{
    adsr_event::{AdsrEvent, AdsrEventType},
    adsr_processor::AdsrProcessor,
};
use crate::{
    commands::Id, effects::Channel, graph::DspParameters, Context, GraphNode, Level, Timestamp,
};
use std::time::Duration;

pub struct Adsr {
    pub node: GraphNode,
    event_transmitter: Channel::Sender<AdsrEvent>,
}

static EVENT_CHANNEL_CAPACITY: usize = 32;

impl Adsr {
    pub fn new(context: &dyn Context, channel_count: usize, sample_rate: usize) -> Self {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = Channel::bounded(EVENT_CHANNEL_CAPACITY);

        let processor = Box::new(AdsrProcessor::new(event_receiver, sample_rate));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            channel_count,
            channel_count,
            processor,
            DspParameters::empty(),
        );

        Self {
            node,
            event_transmitter,
        }
    }

    pub fn note_on_at_time(&mut self, time: Timestamp) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time,
            event_type: AdsrEventType::NoteOn,
        });
    }

    pub fn note_off_at_time(&mut self, time: Timestamp) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time,
            event_type: AdsrEventType::NoteOff,
        });
    }

    pub fn set_attack_time(&mut self, attack_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time: Timestamp::zero(),
            event_type: AdsrEventType::SetAttack(attack_time),
        });
    }

    pub fn set_decay_time(&mut self, decay_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time: Timestamp::zero(),
            event_type: AdsrEventType::SetDecay(decay_time),
        });
    }

    pub fn set_sustain_level(&mut self, sustain_level: Level) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time: Timestamp::zero(),
            event_type: AdsrEventType::SetSustain(sustain_level),
        });
    }

    pub fn set_release_time(&mut self, release_time: Duration) {
        let _ = self.event_transmitter.send(AdsrEvent {
            time: Timestamp::zero(),
            event_type: AdsrEventType::SetRelease(release_time),
        });
    }
}
