use crate::{effects::Channel, Timestamp};

pub trait EventProcessorEvent {
    fn get_time(&self) -> Timestamp;
    fn should_clear_queue(&self) -> bool;
}

pub struct EventProcessor<Event>
where
    Event: EventProcessorEvent,
{
    pending_events: Vec<Event>,
    receive_channel: Channel::Receiver<Event>,
    sample_rate: usize,
}

impl<Event> EventProcessor<Event>
where
    Event: EventProcessorEvent,
{
    pub fn with_capacity(
        capacity: usize,
        receive_channel: Channel::Receiver<Event>,
        sample_rate: usize,
    ) -> Self {
        Self {
            pending_events: Vec::with_capacity(capacity),
            receive_channel,
            sample_rate,
        }
    }

    pub fn receive_events(&mut self) {
        let mut sort_required = false;

        while let Ok(event) = self.receive_channel.try_recv() {
            if event.should_clear_queue() {
                self.pending_events.clear();
            }

            self.pending_events.push(event);
            sort_required = true;
        }

        if sort_required {
            self.pending_events.sort_by(|a, b| {
                let a_time = a.get_time();
                let b_time = b.get_time();
                a_time.partial_cmp(&b_time).unwrap()
            });
        }
    }

    fn next_event_before(&mut self, end_time: &Timestamp) -> Option<Event> {
        if let Some(next_event) = self.pending_events.first() {
            if next_event.get_time() < *end_time {
                return Some(self.pending_events.remove(0));
            }
        }

        None
    }

    pub fn next_event(
        &mut self,
        frame_start_time: &Timestamp,
        current_frame_position: &Timestamp,
        frame_count: usize,
    ) -> (usize, Option<Event>) {
        let frame_end_time = frame_start_time.incremented_by_samples(frame_count, self.sample_rate);

        if let Some(next_event) = self.next_event_before(&frame_end_time) {
            let event_time = std::cmp::max(next_event.get_time(), *current_frame_position);
            let position_in_frame = event_time - *frame_start_time;

            (
                position_in_frame.as_samples(self.sample_rate).floor() as usize,
                Some(next_event),
            )
        } else {
            (frame_count, None)
        }
    }
}
