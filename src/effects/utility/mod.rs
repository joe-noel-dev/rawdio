mod event_processor;
mod periodic_notification;

pub type EventProcessor<Event> = event_processor::EventProcessor<Event>;
pub type PeriodicNotification = periodic_notification::PeriodicNotification;
