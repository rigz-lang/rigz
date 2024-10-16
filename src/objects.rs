use crate::{VMError, Value};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::hash::{Hash, Hasher};

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