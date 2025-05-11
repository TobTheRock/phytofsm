use std::iter::Iterator;

use crate::error::Result;

mod plantuml;

trait IFsmParser {
    fn from_file(&self, file_path: &std::path::Path) -> Result<impl IFsmRepr>;
    fn parse(&self) -> Result<impl IFsmRepr>;
}


trait IFsmRepr {
    fn name(&self) -> Option<&str>;
    fn states(&self) -> impl Iterator<Item = impl IStateRepr>;
    fn transitions(&self) -> impl Iterator<Item = impl ITransitionRepr>;

    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.states().eq(other.states())
            && self.transitions().eq(other.transitions())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateType {
    Simple,
    Enter,
    Exit,
}

trait IStateRepr: PartialEq {
    fn name(&self) -> &str;
    fn descriptions(&self) -> impl Iterator<Item = &str>;
    fn state_type(&self) -> StateType;
    fn parent(&self) -> Option<&str>;

    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.descriptions().eq(other.descriptions())
            && self.state_type() == other.state_type()
            && self.parent() == other.parent()
    }
}

trait ITransitionRepr: PartialEq {
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn description(&self) -> Option<&str>;

    fn eq(&self, other: &Self) -> bool {
        self.from() == other.from()
            && self.to() == other.to()
            && self.description() == other.description()
    }
}

