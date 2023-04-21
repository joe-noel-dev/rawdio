use crate::{
    buffer::BufferPool, effects::utility::EventProcessor, graph::DspProcessor, AudioBuffer,
    OwnedAudioBuffer, ProcessContext, SampleLocation,
};

use super::{
    recorder_event::{RecorderEvent, RecorderEventReceiver, RecorderEventType},
    recorder_notification::{RecorderNotification, RecorderNotificationTransmitter},
};

pub struct RecorderProcessor {
    sample_rate: usize,
    notification_transmitter: RecorderNotificationTransmitter,
    event_processor: EventProcessor<RecorderEvent>,
    buffer_pool: BufferPool,
    current_buffer: OwnedAudioBuffer,
    current_position_in_buffer: usize,
    recording: bool,
}

const MAX_PENDING_EVENTS: usize = 16;
const BUFFER_COUNT: usize = 32;
const BUFFER_SIZE: usize = 4096;

impl RecorderProcessor {
    pub fn new(
        sample_rate: usize,
        channel_count: usize,
        event_receiver: RecorderEventReceiver,
        notification_transmitter: RecorderNotificationTransmitter,
    ) -> Self {
        Self {
            sample_rate,
            event_processor: EventProcessor::with_capacity(
                MAX_PENDING_EVENTS,
                event_receiver,
                sample_rate,
            ),
            notification_transmitter,
            buffer_pool: BufferPool::new(BUFFER_COUNT, BUFFER_SIZE, channel_count, sample_rate),
            current_buffer: OwnedAudioBuffer::new(BUFFER_SIZE, channel_count, sample_rate),
            current_position_in_buffer: 0,
            recording: false,
        }
    }

    fn next_buffer(&mut self) {
        let mut next_buffer = self
            .buffer_pool
            .remove()
            .expect("Ran out of record buffers");

        let samples_used = self.current_position_in_buffer;
        self.current_position_in_buffer = 0;

        std::mem::swap(&mut next_buffer, &mut self.current_buffer);

        let _ = self
            .notification_transmitter
            .send(RecorderNotification::Data(next_buffer, samples_used));
    }

    fn process(&mut self, input_buffer: &dyn AudioBuffer, start_frame: usize, frame_count: usize) {
        if !self.recording {
            return;
        }

        let channel_count = std::cmp::min(
            input_buffer.channel_count(),
            self.current_buffer.channel_count(),
        );

        let mut position = start_frame;

        while position < (start_frame + frame_count) {
            let frames_this_time = std::cmp::min(
                frame_count - position,
                self.current_buffer.frame_count() - self.current_position_in_buffer,
            );

            let source_location = SampleLocation::frame(position);
            let destination_location = SampleLocation::frame(self.current_position_in_buffer);

            self.current_buffer.copy_from(
                input_buffer,
                source_location,
                destination_location,
                channel_count,
                frames_this_time,
            );

            position += frames_this_time;
            self.current_position_in_buffer += frames_this_time;

            if self.current_position_in_buffer >= self.current_buffer.frame_count() {
                self.next_buffer();
            }
        }
    }

    fn start(&mut self) {
        if self.recording {
            return;
        }

        self.recording = true;

        let _ = self
            .notification_transmitter
            .send(RecorderNotification::Start);
    }

    fn stop(&mut self) {
        if !self.recording {
            return;
        }

        self.next_buffer();
        self.recording = false;

        let _ = self
            .notification_transmitter
            .send(RecorderNotification::Stop);
    }

    fn return_buffer(&mut self, buffer: OwnedAudioBuffer) {
        self.buffer_pool.add(buffer);
    }

    fn process_event(&mut self, event: RecorderEvent) {
        match event.event_type {
            RecorderEventType::Start => self.start(),
            RecorderEventType::Stop => self.stop(),
            RecorderEventType::ReturnBuffer(buffer) => self.return_buffer(buffer),
        }
    }
}

impl DspProcessor for RecorderProcessor {
    fn process_audio(&mut self, context: &mut ProcessContext) {
        self.event_processor.receive_events();

        let mut current_time = *context.start_time;
        let mut position = 0;

        while position < context.input_buffer.frame_count() {
            let (end_frame, event) = self.event_processor.next_event(
                context.start_time,
                &current_time,
                context.input_buffer.frame_count(),
            );

            debug_assert!(end_frame <= context.input_buffer.frame_count());

            let frame_count = end_frame - position;

            self.process(context.input_buffer, position, frame_count);

            position += frame_count;
            current_time = current_time.incremented_by_samples(frame_count, self.sample_rate);

            if let Some(event) = event {
                self.process_event(event);
            }
        }
    }
}
