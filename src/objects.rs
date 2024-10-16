use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RigzType {
    None,
    Any,
    Bool,
    Int,
    Float,
    Number,
    String,
    List,
    Map,
    Error,
    This,
    VM,
    Range,
    Type(Box<RigzType>),
    Function(Vec<RigzType>, Box<RigzType>),
    Custom(CustomType),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

// todo create an object/class type and store type definitions in scope
