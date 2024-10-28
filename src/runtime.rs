use std::fmt::{Display, Formatter};
use crate::prepare::{Program, ProgramParser};
use rigz_ast::{VMError, Value, ValidationError, VM, Parser, ParsedModule, ParsingError, TestResults};

pub struct Runtime<'vm> {
    parser: ProgramParser<'vm, VM<'vm>>,
}

impl<'vm> From<ProgramParser<'vm, VM<'vm>>> for Runtime<'vm> {
    #[inline]
    fn from(value: ProgramParser<'vm, VM<'vm>>) -> Self {
        Runtime { parser: value }
    }
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    Parse(ParsingError),
    Validation(ValidationError),
    Run(VMError),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Parse(e) => write!(f, "Parse Error - {e}"),
            RuntimeError::Validation(e) => write!(f, "Validation Error - {e}"),
            RuntimeError::Run(e) => write!(f, "Runtime Error - {e}"),
        }
    }
}

impl Into<RuntimeError> for ParsingError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Parse(self)
    }
}

impl Into<RuntimeError> for VMError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Run(self)
    }
}

impl Into<RuntimeError> for ValidationError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Validation(self)
    }
}

impl<'vm> Runtime<'vm> {
    pub fn vm(&self) -> &VM<'vm> {
        &self.parser.builder
    }

    pub fn vm_mut(&mut self) -> &mut VM<'vm> {
        &mut self.parser.builder
    }

    pub fn new() -> Self {
        Runtime {
            parser: ProgramParser::new(),
        }
    }

    pub fn create(input: &'vm str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        program.validate().map_err(|e| e.into())?;
        let program: Program = program.into();
        program.create_runtime()
    }

    /// Does not include default modules
    pub fn create_with_modules(
        input: &'vm str,
        modules: Vec<impl ParsedModule<'vm> + 'static>,
    ) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        program.validate().map_err(|e| e.into())?;
        let program: Program = program.into();
        program.create_vm_with_modules(modules)
    }

    /// Meant for REPL, skips requirement that programs must end in expression
    pub fn create_unverified(input: &'vm str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program: Program = parser.parse().map_err(|e| e.into())?.into();
        program.create_runtime()
    }

    /// Does not include default modules
    pub fn create_unverified_with_modules(
        input: &'vm str,
        modules: Vec<impl ParsedModule<'vm> + 'static>,
    ) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program: Program = parser.parse().map_err(|e| e.into())?.into();
        program.create_vm_with_modules(modules)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        self.parser.builder.eval().map_err(|e| e.into())
    }

    pub fn test(&mut self) -> TestResults {
        self.parser.builder.test()
    }

    pub fn eval(&mut self, input: String) -> Result<Value, RuntimeError> {
        self.parser.repl(input)?;
        self.run()
    }
}

pub fn eval(input: &str) -> Result<Value, RuntimeError> {
    let mut runtime = Runtime::create(input)?;
    runtime.run()
}
