use crate::{graph::dsp::Dsp, timestamp};

pub enum Notification {
    Position(timestamp::Timestamp),
    DisposeDsp(Dsp),
}
