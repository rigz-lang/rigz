use crate::prepare::{Program, ProgramParser};
use rigz_ast::{ParsedModule, Parser, ParserOptions, ParsingError, ValidationError};
use rigz_core::{ObjectValue, PrimitiveValue, TestResults, VMError};
use rigz_vm::{VMOptions, VM};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct RuntimeOptions {
    vm: VMOptions,
    parser: ParserOptions,
}

pub struct Runtime<'vm> {
    parser: ProgramParser<'vm, VM>,
    runtime_options: RuntimeOptions,
}

impl<'vm> From<ProgramParser<'vm, VM>> for Runtime<'vm> {
    #[inline]
    fn from(value: ProgramParser<'vm, VM>) -> Self {
        Runtime {
            parser: value,
            runtime_options: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    Parse(ParsingError),
    Validation(ValidationError),
    Run(VMError),
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Parse(e) => write!(f, "Parse Error - {e}"),
            RuntimeError::Validation(e) => write!(f, "Validation Error - {e}"),
            RuntimeError::Run(e) => write!(f, "Runtime Error - {e}"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<RuntimeError> for ParsingError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Parse(self)
    }
}

#[allow(clippy::from_over_into)]
impl Into<RuntimeError> for VMError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Run(self)
    }
}

#[allow(clippy::from_over_into)]
impl Into<RuntimeError> for ValidationError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Validation(self)
    }
}

impl Default for Runtime<'_> {
    /// Does not include default modules, use Runtime::new() instead
    fn default() -> Self {
        Runtime {
            parser: ProgramParser::default(),
            runtime_options: Default::default(),
        }
    }
}

impl Runtime<'_> {
    pub fn vm(&self) -> &VM {
        &self.parser.builder
    }

    pub fn vm_mut(&mut self) -> &mut VM {
        &mut self.parser.builder
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, RuntimeError> {
        self.vm().snapshot().map_err(|e| e.into())
    }

    pub fn from_snapshot(bytes: Vec<u8>) -> Result<Runtime<'static>, RuntimeError> {
        let mut runtime = Runtime::new();
        runtime
            .vm_mut()
            .load_snapshot(bytes)
            .map_err(|e| e.into())?;
        Ok(runtime)
    }

    pub fn new() -> Self {
        Runtime {
            parser: ProgramParser::new(),
            runtime_options: Default::default(),
        }
    }

    pub fn with_options(&mut self, options: RuntimeOptions) {
        self.runtime_options = options;
    }

    pub fn create(input: String) -> Result<Self, RuntimeError> {
        let parser = Parser::prepare(&input, false).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        program.validate().map_err(|e| e.into())?;
        let program: Program = program.into();
        program.create_runtime()
    }

    pub fn create_with_options(
        input: String,
        runtime_options: RuntimeOptions,
    ) -> Result<Self, RuntimeError> {
        let parser = Parser::prepare(&input, false).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        program.validate().map_err(|e| e.into())?;
        let program: Program = program.into();
        let mut runtime = program.create_runtime()?;
        runtime.runtime_options = runtime_options;
        Ok(runtime)
    }

    /// Use register_module to add modules
    pub fn create_without_modules(input: String) -> Result<Self, RuntimeError> {
        let parser = Parser::prepare(&input, false).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        program.validate().map_err(|e| e.into())?;
        let program: Program = program.into();
        program.create_runtime_without_modules()
    }

    /// Meant for REPL, skips requirement that programs must end in expression
    pub fn create_unverified(input: String) -> Result<Self, RuntimeError> {
        let parser = Parser::prepare(&input, false).map_err(|e| e.into())?;
        let program: Program = parser.parse().map_err(|e| e.into())?.into();
        program.create_runtime()
    }

    /// Use register_module to add modules, meant for repl
    pub fn create_unverified_without_modules(input: String) -> Result<Self, RuntimeError> {
        let parser = Parser::prepare(&input, false).map_err(|e| e.into())?;
        let program: Program = parser.parse().map_err(|e| e.into())?.into();
        program.create_runtime_without_modules()
    }

    pub fn register_module(
        &mut self,
        module: impl ParsedModule + 'static,
    ) -> Result<(), ValidationError> {
        self.parser.register_module(module)
    }

    pub fn run(&mut self) -> Result<ObjectValue, RuntimeError> {
        self.parser.builder.eval().map_err(|e| e.into())
    }

    pub fn run_within(&mut self, duration: Duration) -> Result<ObjectValue, RuntimeError> {
        self.parser
            .builder
            .run_within(duration)
            .map_err(|e| e.into())
    }

    pub fn test(&mut self) -> TestResults {
        self.parser.builder.test()
    }

    pub fn eval(&mut self, input: String) -> Result<ObjectValue, RuntimeError> {
        self.parser.repl(input)?;
        self.run()
    }

    pub fn eval_within(
        &mut self,
        input: String,
        duration: Duration,
    ) -> Result<ObjectValue, RuntimeError> {
        self.parser.repl(input)?;
        self.run_within(duration)
    }
}

pub fn eval(input: String) -> Result<ObjectValue, RuntimeError> {
    let mut runtime = Runtime::create(input)?;
    runtime.run()
}

pub fn test(input: String) -> Result<TestResults, RuntimeError> {
    let mut runtime = Runtime::create(input)?;
    Ok(runtime.test())
}

pub fn eval_print_vm(input: String) -> Result<ObjectValue, RuntimeError> {
    let mut runtime = Runtime::create(input)?;
    println!("VM (before) - {:#?}", runtime.vm());
    runtime.run()
}
