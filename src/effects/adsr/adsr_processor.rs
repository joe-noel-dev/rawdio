use itertools::izip;

use super::{
    adsr_envelope::AdsrEnvelope,
    adsr_event::{AdsrEvent, AdsrEventType},
};
use crate::{
    effects::{utility::EventProcessor, Channel},
    graph::DspProcessor,
    prelude::*,
};
use std::time::Duration;

pub struct AdsrProcessor {
    event_processor: EventProcessor<AdsrEvent>,
    envelope: AdsrEnvelope,
    envelope_buffer: Vec<f32>,
}

const MAX_PENDING_EVENTS: usize = 64;

impl AdsrProcessor {
    pub fn new(
        event_receiver: Channel::Receiver<AdsrEvent>,
        sample_rate: usize,
        maximum_frame_count: usize,
    ) -> Self {
        Self {
            event_processor: EventProcessor::with_capacity(
                MAX_PENDING_EVENTS,
                event_receiver,
                sample_rate,
            ),
            envelope: AdsrEnvelope::new(
                sample_rate,
                Duration::ZERO,
                Duration::ZERO,
                Level::from_db(0.0),
                Duration::ZERO,
            ),
            envelope_buffer: vec![0.0_f32; maximum_frame_count],
        }
    }

    fn process_event(&mut self, event: AdsrEvent) {
        match event.get_event_type() {
            AdsrEventType::NoteOn => self.envelope.open(),
            AdsrEventType::NoteOff => self.envelope.close(),
            AdsrEventType::SetAttack(attack_time) => self.envelope.set_attack_time(*attack_time),
            AdsrEventType::SetDecay(decay_time) => self.envelope.set_decay_time(*decay_time),
            AdsrEventType::SetSustain(sustain_level) => {
                self.envelope.set_sustain_level(*sustain_level)
            }
            AdsrEventType::SetRelease(release_time) => {
                self.envelope.set_release_time(*release_time)
            }
        }
    }

    fn prepare_envelope(&mut self, frame_count: usize, start_time: &Timestamp, sample_rate: usize) {
        self.event_processor.receive_events();

        let mut current_time = *start_time;
        let mut position = 0;

        while position < frame_count {
            let (end_frame, event) =
                self.event_processor
                    .next_event(start_time, &current_time, frame_count);

            let frame_count = end_frame - position;

            for frame in 0..frame_count {
                let envelope = self.envelope.process();
                self.envelope_buffer[frame + position] = envelope as f32;
            }

            current_time = current_time.incremented_by_samples(frame_count, sample_rate);

            position += frame_count;

            if let Some(event) = event {
                self.process_event(event);
            }
        }
    }
}

impl DspProcessor for AdsrProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        let channel_count = std::cmp::min(
            context.input_buffer.channel_count(),
            context.output_buffer.channel_count(),
        );

        let frame_count = std::cmp::min(
            context.input_buffer.frame_count(),
            context.output_buffer.frame_count(),
        );

        self.prepare_envelope(
            frame_count,
            context.start_time,
            context.output_buffer.sample_rate(),
        );

        for channel in 0..channel_count {
            let location = SampleLocation::channel(channel);
            let output_channel = context.output_buffer.get_channel_data_mut(location);
            let input_channel = context.input_buffer.get_channel_data(location);

            for (output_sample, input_sample, envelope) in izip!(
                output_channel.iter_mut(),
                input_channel.iter(),
                self.envelope_buffer.iter()
            ) {
                *output_sample = *input_sample * *envelope;
            }
        }
    }
}
