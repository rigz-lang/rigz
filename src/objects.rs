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
    Function(Vec<RigzType>, Box<RigzType>),
}
