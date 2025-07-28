use std::fmt::{Display, Formatter};
use crate::RigzType;

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDeclaration {
    pub name: String,
    pub variants: Vec<(String, RigzType)>,
}

impl Display for EnumDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}