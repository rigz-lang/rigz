use std::error::Error;
use crate::program::{Element, Program};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationError {
    MissingExpression(String),
    ModuleError(String),
    InvalidSelf(String),
    InvalidFunction(String),
    InvalidImport(String),
    InvalidExport(String),
    NotImplemented(String),
    InvalidType(String),
}

impl Error for ValidationError {

}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingExpression(e) => write!(f, "Missing Expression: {e}"),
            ValidationError::ModuleError(e) => write!(f, "Module Error: {e}"),
            ValidationError::InvalidSelf(e) => write!(f, "Invalid Self: {e}"),
            ValidationError::InvalidFunction(e) => write!(f, "Invalid Function: {e}"),
            ValidationError::InvalidImport(e) => write!(f, "Invalid Import: {e}"),
            ValidationError::InvalidExport(e) => write!(f, "Invalid Export: {e}"),
            ValidationError::NotImplemented(e) => write!(f, "Not Implemented: {e}"),
            ValidationError::InvalidType(e) => write!(f, "Invalid Type: {e}"),
        }
    }
}

impl Program<'_> {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self.elements.last() {
            None => Err(ValidationError::MissingExpression(
                "Invalid Program, no elements".to_string(),
            )),
            Some(e) => match e {
                Element::Statement(_) => Err(ValidationError::MissingExpression(
                    "Invalid Program, must end with expression".to_string(),
                )),
                Element::Expression(_) => Ok(()),
            },
        }
    }
}
