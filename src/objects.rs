use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RigzType {
    None,
    Any,
    Bool,
    Number,
    String,
    List,
    Map,
    Error,
    This,
    Function(Vec<RigzType>, Box<RigzType>),
}

// todo create an object/class type and store type definitions in scope
