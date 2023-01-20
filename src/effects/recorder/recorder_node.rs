use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    commands::Id, effects::Channel, engine::NotifierStatus, BorrowedAudioBuffer, Context, Node,
    OwnedAudioBuffer, Timestamp,
};

use super::{
    recorder_event::{RecorderEvent, RecorderEventTransmitter},
    recorder_notification::{RecorderNotification, RecorderNotificationReceiver},
    recorder_processor::RecorderProcessor,
};

pub struct RecorderNode {
    pub node: Node,
    event_transmitter: RecorderEventTransmitter,
    notification_receiver: RecorderNotificationReceiver,
    current_recording: Option<OwnedAudioBuffer>,
    is_recording: bool,
}

impl RecorderNode {
    pub fn new(
        context: &mut dyn Context,
        channel_count: usize,
        sample_rate: usize,
    ) -> Rc<RefCell<Self>> {
        let id = Id::generate();

        let parameters = HashMap::new();

        let (event_transmitter, event_receiver) = Channel::create();
        let (notification_transmitter, notification_receiver) = Channel::create();

        let processor = Box::new(RecorderProcessor::new(
            sample_rate,
            channel_count,
            event_receiver,
            notification_transmitter,
        ));

        let output_count = 0;

        let node = Node::new(
            id,
            context.get_command_queue(),
            channel_count,
            output_count,
            processor,
            parameters,
        );

        let recorder = Rc::new(RefCell::new(Self {
            node,
            event_transmitter,
            notification_receiver,
            current_recording: None,
            is_recording: false,
        }));

        let weak_recorder = Rc::downgrade(&recorder);

        context.add_notifier(Box::new(move || {
            if let Some(recorder) = weak_recorder.upgrade() {
                recorder.borrow_mut().process_notifications();
                return NotifierStatus::Continue;
            }

            NotifierStatus::Remove
        }));

        recorder
    }

    pub fn record_now(&mut self) {
        let _ = self.event_transmitter.send(RecorderEvent::start_now());
    }

    pub fn stop_record_now(&mut self) {
        let _ = self.event_transmitter.send(RecorderEvent::stop_now());
    }

    pub fn stop_record_at_time(&mut self, time: Timestamp) {
        let _ = self
            .event_transmitter
            .send(RecorderEvent::stop_at_time(time));
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn take_recording(&mut self) -> Option<OwnedAudioBuffer> {
        self.current_recording.take()
    }

    fn append_buffer(&mut self, buffer: &OwnedAudioBuffer, samples_used: usize) {
        let slice = BorrowedAudioBuffer::slice_frames(buffer, 0, samples_used);
        self.current_recording = match &self.current_recording {
            Some(current_recording) => Some(current_recording.extended_with_buffer(&slice)),
            None => Some(OwnedAudioBuffer::from_buffer(&slice)),
        };
    }

    fn return_buffer(&mut self, buffer: OwnedAudioBuffer) {
        let _ = self
            .event_transmitter
            .send(RecorderEvent::return_buffer(buffer));
    }

    fn process_notifications(&mut self) {
        while let Ok(event) = self.notification_receiver.recv() {
            match event {
                RecorderNotification::Start => self.is_recording = true,
                RecorderNotification::Data(buffer, samples_used) => {
                    self.append_buffer(&buffer, samples_used);
                    self.return_buffer(buffer);
                }
                RecorderNotification::Stop => self.is_recording = false,
            }
        }
    }
}
