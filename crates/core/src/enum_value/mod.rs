use crate::RigzType;

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDeclaration {
    pub name: String,
    pub variants: Vec<(String, RigzType)>,
}
