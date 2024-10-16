use crate::{VMError, Value, VM};
use dyn_clone::DynClone;

/// modules will be cloned when used, until DynClone can be removed
pub trait Module<'vm>: DynClone {
    fn name(&self) -> &'vm str;

    fn call(&self, function: &'vm str, args: Vec<Value<'vm>>) -> Result<Value<'vm>, VMError>;

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value<'vm>>,
    ) -> Result<Value<'vm>, VMError>;

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value<'vm>>,
    ) -> Result<Value<'vm>, VMError>;

    fn extensions(&self) -> &[&str];

    fn functions(&self) -> &[&str];

    fn vm_extensions(&self) -> &[&str];
}

dyn_clone::clone_trait_object!(Module<'_>);
