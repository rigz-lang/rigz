pub(crate) mod expression;

use crate::prepare::ProgramParser;
use crate::{Runtime, RuntimeError};
use rigz_ast::Element;

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

impl<'lex> Into<Program<'lex>> for rigz_ast::Program<'lex> {
    fn into(self) -> Program<'lex> {
        Program {
            elements: self.elements,
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
