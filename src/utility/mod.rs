mod level;
mod scoped_time_measure;
mod timestamp;

pub type Level = level::Level;
pub type Timestamp = timestamp::Timestamp;

pub mod macros {
    macro_rules! unwrap_or_return {
        ( $e:expr ) => {
            match $e {
                Some(x) => x,
                None => return,
            }
        };
    }

    pub(crate) use unwrap_or_return;
}
