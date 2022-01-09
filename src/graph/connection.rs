use crate::commands::id::Id;

#[derive(Clone, PartialEq)]
pub struct Connection {
    pub from_id: Id,
    pub output_index: usize,
    pub to_id: Id,
    pub input_index: usize,
}

#[derive(Clone, PartialEq)]
pub struct OutputConnection {
    pub from_id: Id,
    pub output_index: usize,
}
