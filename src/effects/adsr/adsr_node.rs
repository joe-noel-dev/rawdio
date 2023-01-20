use super::{
    adsr_event::{AdsrEvent, AdsrEventType},
    adsr_processor::AdsrProcessor,
};
use crate::{commands::Id, effects::Channel, CommandQueue, Level, Node, Timestamp};
use std::{collections::HashMap, time::Duration};

pub struct AdsrNode {
    pub node: Node,
    event_transmitter: Channel::Sender<AdsrEvent>,
}

impl AdsrNode {
    pub fn new(command_queue: CommandQueue, channel_count: usize, sample_rate: usize) -> Self {
        let id = Id::generate();

        let parameters = HashMap::new();

        let (event_transmitter, event_receiver) = Channel::create();

        let processor = Box::new(AdsrProcessor::new(event_receiver, sample_rate));

        let node = Node::new(
            id,
            command_queue,
            channel_count,
            channel_count,
            processor,
            parameters,
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
