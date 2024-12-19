pub(crate) mod expression;

use crate::prepare::ProgramParser;
use crate::{Runtime, RuntimeError};
use rigz_ast::Element;

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

impl<'l> From<rigz_ast::Program<'l>> for Program<'l> {
    fn from(value: rigz_ast::Program<'l>) -> Self {
        Program {
            elements: value.elements,
        }
    }
}

impl<'lex> Program<'lex> {
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
