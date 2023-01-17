use crate::{
    buffer::BufferPool,
    graph::{DspParameters, DspProcessor},
    AudioBuffer, OwnedAudioBuffer, SampleLocation, Timestamp,
};

use super::{
    recorder_event::{RecorderEvent, RecorderEventReceiver, RecorderEventType},
    recorder_notification::{RecorderNotification, RecorderNotificationTransmitter},
};

pub struct RecorderProcessor {
    sample_rate: usize,
    event_receiver: RecorderEventReceiver,
    notification_transmitter: RecorderNotificationTransmitter,
    pending_events: Vec<RecorderEvent>,
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
            event_receiver,
            notification_transmitter,
            pending_events: Vec::with_capacity(MAX_PENDING_EVENTS),
            buffer_pool: BufferPool::new(BUFFER_COUNT, BUFFER_SIZE, channel_count, sample_rate),
            current_buffer: OwnedAudioBuffer::new(BUFFER_SIZE, channel_count, sample_rate),
            current_position_in_buffer: 0,
            recording: false,
        }
    }

    fn process_events(&mut self) {
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

    fn next_event_before(&mut self, end_time: &Timestamp) -> Option<RecorderEvent> {
        if let Some(next_event) = self.pending_events.first() {
            if next_event.time < *end_time {
                return Some(self.pending_events.remove(0));
            }
        }

        None
    }

    fn next_event_position(
        &mut self,
        frame_start_time: &Timestamp,
        current_frame_position: &Timestamp,
        frame_count: usize,
    ) -> (usize, Option<RecorderEvent>) {
        let frame_end_time = frame_start_time.incremented_by_samples(frame_count, self.sample_rate);

        if let Some(next_event) = self.next_event_before(&frame_end_time) {
            let event_time = std::cmp::max(next_event.time, *current_frame_position);
            let position_in_frame = event_time - *frame_start_time;

            (
                position_in_frame.as_samples(self.sample_rate).floor() as usize,
                Some(next_event),
            )
        } else {
            (frame_count, None)
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
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        _output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        _parameters: &DspParameters,
    ) {
        self.process_events();

        let mut current_time = *start_time;
        let mut position = 0;

        while position < input_buffer.frame_count() {
            let (end_frame, event) =
                self.next_event_position(start_time, &current_time, input_buffer.frame_count());

            debug_assert!(end_frame <= input_buffer.frame_count());

            let frame_count = end_frame - position;

            self.process(input_buffer, position, frame_count);

            position += frame_count;
            current_time = current_time.incremented_by_samples(frame_count, self.sample_rate);

            if let Some(event) = event {
                self.process_event(event);
            }
        }
    }
}
