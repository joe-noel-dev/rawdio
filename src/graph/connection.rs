use crate::commands::id::Id;

#[derive(Clone, PartialEq)]
pub struct Connection {
    pub source_id: Id,
    pub destination_id: Id,
}

#[derive(Clone, PartialEq)]
pub struct OutputConnection {
    pub source_id: Id,
}
