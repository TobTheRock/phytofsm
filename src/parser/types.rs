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

impl From<&str> for Event {
    fn from(s: &str) -> Self {
        Event(s.to_string())
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action(s.to_string())
    }
}
