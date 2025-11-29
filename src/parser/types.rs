use derive_more::{From, Into};

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Event(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Action(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    Simple,
    Enter,
}
