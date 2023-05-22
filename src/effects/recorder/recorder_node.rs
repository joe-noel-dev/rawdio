use std::{cell::RefCell, rc::Rc};

use crate::{
    commands::Id, effects::Channel, engine::NotifierStatus, graph::DspParameters,
    BorrowedAudioBuffer, Context, GraphNode, OwnedAudioBuffer, Timestamp,
};

use super::{
    recorder_event::{RecorderEvent, RecorderEventTransmitter},
    recorder_notification::{RecorderNotification, RecorderNotificationReceiver},
    recorder_processor::RecorderProcessor,
};

/// A node that records its input
///
/// The recorder node doesn't produce output so should be at the end of the
/// chain
///
/// Call `take_recording()` to get the recording from the node
pub struct Recorder {
    /// The node to connect to the audio graph
    pub node: GraphNode,
    event_transmitter: RecorderEventTransmitter,
    notification_receiver: RecorderNotificationReceiver,
    current_recording: Option<OwnedAudioBuffer>,
    is_recording: bool,
}

static EVENT_CHANNEL_CAPACITY: usize = 32;
static NOTIFICATION_CHANNEL_CAPACITY: usize = 32;

impl Recorder {
    /// Create a new recorder node
    pub fn new(
        context: &mut dyn Context,
        channel_count: usize,
        sample_rate: usize,
    ) -> Rc<RefCell<Self>> {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = Channel::bounded(EVENT_CHANNEL_CAPACITY);
        let (notification_transmitter, notification_receiver) =
            Channel::bounded(NOTIFICATION_CHANNEL_CAPACITY);

        let processor = Box::new(RecorderProcessor::new(
            sample_rate,
            channel_count,
            event_receiver,
            notification_transmitter,
        ));

        let output_count = 0;

        let node = GraphNode::new(
            id,
            context,
            channel_count,
            output_count,
            processor,
            DspParameters::empty(),
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

    /// Start recording
    pub fn record_now(&mut self) {
        let _ = self.event_transmitter.send(RecorderEvent::start_now());
    }

    /// Stop recording
    pub fn stop_record_now(&mut self) {
        let _ = self.event_transmitter.send(RecorderEvent::stop_now());
    }

    /// Stop recording at a particular time
    pub fn stop_record_at_time(&mut self, time: Timestamp) {
        let _ = self
            .event_transmitter
            .send(RecorderEvent::stop_at_time(time));
    }

    /// Query if the recorder is currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Take the current recording from the node
    ///
    /// This will clear the recording
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
        while let Ok(event) = self.notification_receiver.try_recv() {
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
