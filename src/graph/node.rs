use crate::commands::id::Id;

pub trait Node {
    fn get_id(&self) -> Id;
}
