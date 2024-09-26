pub use rigz_vm::{Module, Number, RigzType, VMBuilder, Value, VM};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition<'vm> {
    pub arguments: Vec<FunctionArgument<'vm>>,
    pub return_type: RigzType,
    pub positional: bool
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument<'vm> {
    pub name: Option<&'vm str>,
    pub default: Option<Value<'vm>>,
    pub rigz_type: RigzType,
}

mod ast;
mod token;
