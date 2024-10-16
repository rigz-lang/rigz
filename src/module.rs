use crate::{VMError, Value, VM};
use dyn_clone::DynClone;

/// modules will be cloned when used, until DynClone can be removed
pub trait Module<'vm>: DynClone {
    fn name(&self) -> &'vm str; // todo should this be static?

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

    // todo get rid of extensions, functions, and vm_extensions. use trait definition
    fn extensions(&self) -> &'vm [&'vm str];

    fn functions(&self) -> &'vm [&'vm str];

    fn vm_extensions(&self) -> &'vm [&'vm str];

    // todo create proc_macro that uses tree-sitter-rigz for syntax highlighting and compile time validation
    fn trait_definition(&self) -> &'vm str; // todo should this be static?
}

dyn_clone::clone_trait_object!(Module<'_>);
