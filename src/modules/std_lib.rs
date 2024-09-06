use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct StdLibModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for StdLibModule {
    fn name(&self) -> &'vm str {
        "std"
    }

    fn call(&self, function: &'vm str, args: Vec<Value<'vm>>) -> Result<Value<'vm>, VMError> {
        todo!()
    }

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value<'vm>>,
    ) -> Result<Value<'vm>, VMError> {
        todo!()
    }

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value<'vm>>,
    ) -> Result<Value<'vm>, VMError> {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        todo!()
    }

    fn functions(&self) -> &[&str] {
        todo!()
    }

    fn vm_extensions(&self) -> &[&str] {
        todo!()
    }
}
