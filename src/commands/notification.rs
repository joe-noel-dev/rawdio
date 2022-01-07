use crate::timestamp;

pub enum Notification {
    Position(timestamp::Timestamp),
}
