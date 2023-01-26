use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::effects::Channel;
use crate::engine::NotifierStatus;
use crate::graph::DspParameters;
use crate::{commands::Id, Context, GraphNode};

use super::envelope_notification::{EnvelopeNotification, EnvelopeNotificationReceiver};
use super::envelope_processor::EnvelopeProcessor;

pub struct Envelope {
    pub node: GraphNode,
    notifications: Vec<EnvelopeNotification>,
    notification_receiver: EnvelopeNotificationReceiver,
}

static NOTIFICATION_CHANNEL_CAPACITY: usize = 64;

impl Envelope {
    pub fn new(
        context: &mut dyn Context,
        channel_count: usize,
        attack_time: Duration,
        release_time: Duration,
        notification_frequency: f64,
    ) -> Rc<RefCell<Self>> {
        let id = Id::generate();

        let (notification_transmitter, notification_receiver) =
            Channel::bounded(NOTIFICATION_CHANNEL_CAPACITY);

        let processor = Box::new(EnvelopeProcessor::new(
            context.get_sample_rate(),
            channel_count,
            attack_time,
            release_time,
            notification_frequency,
            notification_transmitter,
        ));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            channel_count,
            channel_count,
            processor,
            DspParameters::empty(),
        );

        let envelope_node = Rc::new(RefCell::new(Envelope {
            node,
            notifications: Vec::new(),
            notification_receiver,
        }));

        let weak_envelope = Rc::downgrade(&envelope_node);

        context.add_notifier(Box::new(move || {
            if let Some(envelope) = weak_envelope.upgrade() {
                envelope.borrow_mut().process_notifications();
                return NotifierStatus::Continue;
            }

            NotifierStatus::Remove
        }));

        envelope_node
    }

    fn process_notifications(&mut self) {
        while let Ok(notification) = self.notification_receiver.try_recv() {
            self.notifications.push(notification);
        }
    }

    pub fn take_notifications(&mut self) -> Vec<EnvelopeNotification> {
        let mut notifications = Vec::new();
        std::mem::swap(&mut notifications, &mut self.notifications);
        notifications
    }
}
