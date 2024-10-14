use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct StdLibModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for StdLibModule {
    fn name(&self) -> &'static str {
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

    fn trait_definition(&self) -> &'static str {
        r#"import trait STD
            fn Any.is_err -> Bool
            fn Any.is_none -> Bool
            fn Any.to_b -> Bool
            fn Any.to_i -> Int!
            fn Any.to_f -> Float!
            fn Any.to_n -> Number!
            fn Any.to_s -> String
            fn Any.type -> String

            fn mut List.extend(value: List)
            fn List.first -> Any?
            fn List.last -> Any?
            fn mut List.push(var value)
            fn List.concat(value: List) -> List
            fn List.with(var value) -> List

            fn mut Map.extend(value: Map)
            fn Map.first -> Any?
            fn Map.last -> Any?
            fn mut Map.insert(key, value)
            fn Map.with(var key, value) -> Map
            fn Map.concat(value: Map) -> Map
            fn Map.entries -> List
            fn Map.keys -> List

            fn Number.ceil -> Number
            fn Number.round -> Number
            fn Number.trunc -> Number

            fn mut String.push(value)
            fn String.concat(value: String) -> String
            fn String.with(value) -> String
            fn String.trim -> String

            fn format(template: String, var args) -> String
            fn puts(var args)
            fn printf(template: String, var args)
        end"#
    }
}
