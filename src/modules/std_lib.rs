use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct StdLibModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for StdLibModule {
    fn name(&self) -> &'vm str {
        "STD"
    }

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
        todo!()
    }

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        todo!()
    }

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        todo!()
    }

    fn extensions(&self) -> &'vm [&'vm str] {
        todo!()
    }

    fn functions(&self) -> &'vm [&'vm str] {
        todo!()
    }

    fn vm_extensions(&self) -> &'vm [&'vm str] {
        &[]
    }

    fn trait_definition(&self) -> &'vm str {
        r#"trait STD
            fn Any.is_err -> Bool
            fn Any.is_none -> Bool
            fn Any.to_n -> Number!
            fn Any.to_s -> String

            fn format(template: String, var args) -> String
            fn puts(var args)
            fn printf(template: String, var args)
        end"#
    }
}
