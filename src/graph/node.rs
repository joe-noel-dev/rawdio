use crate::realtime::id::Id;

pub trait Node {
    fn get_id(&self) -> Id;
}
