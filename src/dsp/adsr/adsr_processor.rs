use super::{
    adsr_envelope::AdsrEnvelope,
    adsr_event::{AdsrEvent, AdsrEventType},
};
use crate::{
    dsp::{utility::EventProcessor, Channel},
    graph::DspProcessor,
    Level, SampleLocation,
};
use std::time::Duration;

pub struct AdsrProcessor {
    event_processor: EventProcessor<AdsrEvent>,
    envelope: AdsrEnvelope,
}

const MAX_PENDING_EVENTS: usize = 64;

impl AdsrProcessor {
    pub fn new(event_receiver: Channel::Receiver<AdsrEvent>, sample_rate: usize) -> Self {
        Self {
            event_processor: EventProcessor::with_capacity(
                MAX_PENDING_EVENTS,
                event_receiver,
                sample_rate,
                |event| event.time,
            ),
            envelope: AdsrEnvelope::new(
                sample_rate,
                Duration::ZERO,
                Duration::ZERO,
                Level::from_db(0.0),
                Duration::ZERO,
            ),
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
}

impl DspProcessor for AdsrProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn crate::AudioBuffer,
        output_buffer: &mut dyn crate::AudioBuffer,
        start_time: &crate::timestamp::Timestamp,
        _parameters: &crate::graph::DspParameters,
    ) {
        self.event_processor.process_events();

        let mut current_time = *start_time;
        let mut position = 0;

        while position < output_buffer.frame_count() {
            let (end_frame, event) = self.event_processor.next_event(
                start_time,
                &current_time,
                output_buffer.frame_count(),
            );

            let frame_count = end_frame - position;

            for frame in 0..frame_count {
                let envelope = self.envelope.process();

                for channel in 0..output_buffer.channel_count() {
                    let location = SampleLocation::new(channel, frame);
                    let sample = input_buffer.get_sample(location);
                    let sample = sample * envelope as f32;
                    output_buffer.set_sample(location, sample);
                }
            }

            current_time =
                current_time.incremented_by_samples(frame_count, output_buffer.sample_rate());

            position += frame_count;

            if let Some(event) = event {
                self.process_event(event);
            }
        }
    }
}
