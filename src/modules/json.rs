use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct JsonModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for JsonModule {
    fn name(&self) -> &'static str {
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

    fn trait_definition(&self) -> &'static str {
        r#"import trait JSON
            fn Any.to_json -> String!

            fn parse(input: String) -> Any!
        end"#
    }
}
