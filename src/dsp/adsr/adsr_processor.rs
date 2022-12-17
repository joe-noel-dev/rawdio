use super::{
    adsr_envelope::AdsrEnvelope,
    adsr_event::{AdsrEvent, AdsrEventType},
};
use crate::{dsp::Channel, graph::DspProcessor, Level, SampleLocation, Timestamp};
use std::time::Duration;

pub struct AdsrProcessor {
    event_receiver: Channel::Receiver<AdsrEvent>,
    pending_events: Vec<AdsrEvent>,
    envelope: AdsrEnvelope,
}

const MAX_PENDING_EVENTS: usize = 64;

impl AdsrProcessor {
    pub fn new(event_receiver: Channel::Receiver<AdsrEvent>, sample_rate: usize) -> Self {
        Self {
            event_receiver,
            pending_events: Vec::with_capacity(MAX_PENDING_EVENTS),
            envelope: AdsrEnvelope::new(
                sample_rate,
                Duration::ZERO,
                Duration::ZERO,
                Level::from_db(0.0),
                Duration::ZERO,
            ),
        }
    }

    fn read_events(&mut self) {
        let mut sort_required = false;

        while let Ok(event) = self.event_receiver.recv() {
            self.pending_events.push(event);
            sort_required = true;
        }

        if sort_required {
            self.pending_events
                .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
        }
    }

    fn process_event(&mut self, event: AdsrEvent) {
        match event.event_type {
            AdsrEventType::NoteOn => self.envelope.open(),
            AdsrEventType::NoteOff => self.envelope.close(),
            AdsrEventType::SetAttack(attack_time) => self.envelope.set_attack_time(attack_time),
            AdsrEventType::SetDecay(decay_time) => self.envelope.set_decay_time(decay_time),
            AdsrEventType::SetSustain(sustain_level) => {
                self.envelope.set_sustain_level(sustain_level)
            }
            AdsrEventType::SetRelease(release_time) => self.envelope.set_release_time(release_time),
        }
    }

    fn process_events(&mut self, time: Timestamp) {
        while let Some(event) = self.next_event_before(time) {
            self.process_event(event);
        }
    }

    fn next_event_before(&mut self, time: Timestamp) -> Option<AdsrEvent> {
        if let Some(next_event) = self.pending_events.first() {
            if next_event.time <= time {
                return Some(self.pending_events.remove(0));
            }
        }

        None
    }
}

impl DspProcessor for AdsrProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn crate::AudioBuffer,
        output_buffer: &mut dyn crate::AudioBuffer,
        start_time: &crate::timestamp::Timestamp,
        _parameters: &crate::graph::DspParameters,
    ) {
        self.read_events();

        for frame in 0..output_buffer.num_frames() {
            self.process_events(
                start_time.incremented_by_samples(frame, output_buffer.sample_rate()),
            );

            let envelope = self.envelope.process();

            for channel in 0..output_buffer.num_channels() {
                let location = SampleLocation::new(channel, frame);
                let sample = input_buffer.get_sample(location);
                let sample = sample * envelope as f32;
                output_buffer.set_sample(location, sample);
            }
        }
    }
}
