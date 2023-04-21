use crate::{effects::Channel, Timestamp};

type GetEventTime<Event> = fn(&Event) -> Timestamp;

pub struct EventProcessor<Event> {
    pending_events: Vec<Event>,
    receive_channel: Channel::Receiver<Event>,
    sample_rate: usize,
    get_event_time: GetEventTime<Event>,
}

impl<Event> EventProcessor<Event> {
    pub fn with_capacity(
        capacity: usize,
        receive_channel: Channel::Receiver<Event>,
        sample_rate: usize,
        get_event_time: GetEventTime<Event>,
    ) -> Self {
        Self {
            pending_events: Vec::with_capacity(capacity),
            receive_channel,
            sample_rate,
            get_event_time,
        }
    }

    pub fn receive_events(&mut self) {
        let mut sort_required = false;

        while let Ok(event) = self.receive_channel.try_recv() {
            self.pending_events.push(event);
            sort_required = true;
        }

        if sort_required {
            self.pending_events.sort_by(|a, b| {
                let a_time = (self.get_event_time)(a);
                let b_time = (self.get_event_time)(b);
                a_time.partial_cmp(&b_time).unwrap()
            });
        }
    }

    fn next_event_before(&mut self, end_time: &Timestamp) -> Option<Event> {
        if let Some(next_event) = self.pending_events.first() {
            if (self.get_event_time)(next_event) < *end_time {
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
            let event_time =
                std::cmp::max((self.get_event_time)(&next_event), *current_frame_position);
            let position_in_frame = event_time - *frame_start_time;

            (
                position_in_frame.as_samples(self.sample_rate).floor() as usize,
                Some(next_event),
            )
        } else {
            (frame_count, None)
        }
    }

    pub fn cancel_all_pending_events(&mut self) {
        self.pending_events.clear();
    }
}
