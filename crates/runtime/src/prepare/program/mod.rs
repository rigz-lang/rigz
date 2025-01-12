pub(crate) mod expression;

use crate::prepare::ProgramParser;
use crate::{Runtime, RuntimeError};
use rigz_ast::Element;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub elements: Vec<Element>,
}

impl From<rigz_ast::Program> for Program {
    fn from(value: rigz_ast::Program) -> Self {
        Program {
            elements: value.elements,
        }
    }
}

impl<'lex> Program {
    #[inline]
    pub fn create_runtime(self) -> Result<Runtime<'lex>, RuntimeError> {
        let mut builder = ProgramParser::new();
        builder.parse_program(self).map_err(|e| e.into())?;
        Ok(builder.create().into())
    }

    #[inline]
    pub fn create_runtime_without_modules(self) -> Result<Runtime<'lex>, RuntimeError> {
        let mut builder = ProgramParser::default();
        builder.parse_program(self).map_err(|e| e.into())?;
        Ok(builder.create().into())
    }
}
