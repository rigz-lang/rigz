use rigz_ast::{Element, ParsedModule};
use crate::prepare::ProgramParser;
use crate::{Runtime, RuntimeError};

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

impl <'lex> Into<Program<'lex>> for rigz_ast::Program<'lex> {
    fn into(self) -> Program<'lex> {
        Program {
            elements: self.elements,
        }
    }
}

impl<'lex> Program<'lex> {
    #[inline]
    /// skips program validation (used for REPL to support statements as last line)
    pub fn create_runtime(self) -> Result<Runtime<'lex>, RuntimeError> {
        let mut builder = ProgramParser::new();
        builder.parse_program(self).map_err(|e| e.into())?;
        Ok(builder.create().into())
    }

    #[inline]
    /// skips program validation (used for REPL to support statements as last line)
    pub fn create_vm_with_modules(
        self,
        modules: Vec<impl ParsedModule<'lex> + 'static>,
    ) -> Result<Runtime<'lex>, RuntimeError> {
        let mut builder = ProgramParser::with_modules(modules).map_err(|e| e.into())?;
        builder.parse_program(self).map_err(|e| e.into())?;
        Ok(builder.create().into())
    }
}