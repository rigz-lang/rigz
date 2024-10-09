use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct JsonModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for JsonModule {
    fn name(&self) -> &'vm str {
        "JSON"
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
        r#"trait JSON
            fn Any.to_json -> String!

            fn parse(input: String) -> Any!
        end"#
    }
}
