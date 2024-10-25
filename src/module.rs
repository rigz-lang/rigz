use crate::{VMError, Value, VM};
use dyn_clone::DynClone;
use std::fmt::Debug;

/// modules will be cloned when used, until DynClone can be removed, ideally they're Copy + Clone
#[allow(unused_variables)]
pub trait Module<'vm>: Debug + DynClone {
    fn name(&self) -> &'static str;

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call`",
            self.name()
        )))
    }

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_extension`",
            self.name()
        )))
    }

    fn call_mutable_extension(
        &self,
        value: &mut Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Option<Value>, VMError> {
        Ok(Some(
            VMError::UnsupportedOperation(format!(
                "{} does not implement `call_mutable_extension`",
                self.name()
            ))
            .to_value(),
        ))
    }

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `vm_extension`",
            self.name()
        )))
    }

    // todo create proc_macro that uses tree-sitter-rigz for syntax highlighting and compile time syntax validation
    fn trait_definition(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(Module<'_>);
