pub use rigz_vm::{Module, Number, RigzType, VMBuilder, Value, VM};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition<'vm> {
    pub arguments: Vec<FunctionArgument<'vm>>,
    pub return_type: RigzType,
    pub self_type: Option<FunctionType>,
    pub positional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    rigz_type: RigzType,
    mutable: bool,
}

impl Into<FunctionType> for RigzType {
    fn into(self) -> FunctionType {
        FunctionType::new(self)
    }
}

impl FunctionType {
    fn new(rigz_type: RigzType) -> Self {
        Self {
            rigz_type,
            mutable: false,
        }
    }

    fn mutable(rigz_type: RigzType) -> Self {
        Self {
            rigz_type,
            mutable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument<'vm> {
    pub name: Option<&'vm str>,
    pub default: Option<Value>,
    pub function_type: FunctionType,
}

pub mod ast;
pub mod modules;
pub mod prepare;
pub mod runtime;
pub mod token;
