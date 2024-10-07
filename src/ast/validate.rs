use crate::ast::{Element, Program};

#[derive(Debug)]
pub enum ValidationError {
    MissingExpression(String),
}

impl<'vm> Program<'vm> {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self.elements.last() {
            None => Err(ValidationError::MissingExpression(
                "Invalid Program, no elements".to_string(),
            )),
            Some(e) => match e {
                Element::Statement(_) => Err(ValidationError::MissingExpression(
                    "Invalid Program, file must end with expression".to_string(),
                )),
                Element::Expression(_) => Ok(()),
            },
        }
    }
}
