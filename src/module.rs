use crate::{VMError, Value, VM};
use dyn_clone::DynClone;

/// modules will be cloned when used, until DynClone can be removed, ideally they're Copy + Clone
pub trait Module<'vm>: DynClone {
    fn name(&self) -> &'static str;

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError>;

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError>;

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError>;

    // todo create proc_macro that uses tree-sitter-rigz for syntax highlighting and compile time syntax validation
    fn trait_definition(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(Module<'_>);
