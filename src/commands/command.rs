use super::id::Id;

pub enum Command {
    Start,
    Stop,

    AddOscillator(Id),

    RemoveNode(Id),
}
