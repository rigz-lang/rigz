use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

// need to borrow this for ext
derive_module!(
    r#"trait JSON
        fn Any.to_json -> String!
        fn parse(input: String) -> Any!
    end"#
);

derive_module!(
    r#"trait File
        fn read(path: String, encoding = "utf-8") -> String!
        fn write(path: String, contents: String, encoding = "utf-8") -> None!
    end"#
);

derive_module!(
    r#"trait VM
        fn mut VM.get_register(register: Number) -> Any!
        fn mut VM.first -> Any!
        fn mut VM.last -> Any!
        fn mut VM.remove_register(register: Number) -> Any!
    end"#
);

derive_module!(
    r#"import trait Std
        fn Any.clone -> Any
        fn Any.is_err -> Bool
        fn Any.is_none -> Bool
        fn Any.is_some -> Bool
        fn Any.to_b -> Bool
        fn Any.to_i -> Int!
        fn Any.to_f -> Float!
        fn Any.to_n -> Number!
        fn Any.to_s -> String
        fn Any.to_list -> List
        fn Any.to_map -> Map
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
        fn String.with(var value) -> String
        fn String.trim -> String

        fn format(template: String, var args) -> String
        fn printf(template: String, var args) -> None
    end"#
);

#[allow(unused_variables)]
impl RigzStd for StdModule {
    fn any_clone(&self, this: Value) -> Value {
        todo!()
    }

    fn any_is_err(&self, this: Value) -> bool {
        todo!()
    }

    fn any_is_none(&self, this: Value) -> bool {
        todo!()
    }

    fn any_is_some(&self, this: Value) -> bool {
        todo!()
    }

    fn any_to_b(&self, this: Value) -> bool {
        todo!()
    }

    fn any_to_i(&self, this: Value) -> Result<i64, VMError> {
        todo!()
    }

    fn any_to_f(&self, this: Value) -> Result<f64, VMError> {
        todo!()
    }

    fn any_to_n(&self, this: Value) -> Result<Number, VMError> {
        todo!()
    }

    fn any_to_s(&self, this: Value) -> String {
        todo!()
    }

    fn any_to_list(&self, this: Value) -> Vec<Value> {
        todo!()
    }

    fn any_to_map(&self, this: Value) -> IndexMap<Value, Value> {
        todo!()
    }

    fn any_type(&self, this: Value) -> String {
        todo!()
    }

    fn mut_list_extend(&self, this: &mut Vec<Value>, value: Vec<Value>) {
        todo!()
    }

    fn list_first(&self, this: Vec<Value>) -> Option<Value> {
        todo!()
    }

    fn list_last(&self, this: Vec<Value>) -> Option<Value> {
        todo!()
    }

    fn mut_list_push(&self, this: &mut Vec<Value>, value: Vec<Value>) {
        todo!()
    }

    fn list_concat(&self, this: Vec<Value>, value: Vec<Value>) -> Vec<Value> {
        todo!()
    }

    fn list_with(&self, this: Vec<Value>, value: Vec<Value>) -> Vec<Value> {
        todo!()
    }

    fn mut_map_extend(&self, this: &mut IndexMap<Value, Value>, value: IndexMap<Value, Value>) {
        todo!()
    }

    fn map_first(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        todo!()
    }

    fn map_last(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        todo!()
    }

    fn mut_map_insert(&self, this: &mut IndexMap<Value, Value>, key: Value, value: Value) {
        todo!()
    }

    fn map_with(
        &self,
        this: IndexMap<Value, Value>,
        key: Vec<Value>,
        value: Vec<Value>,
    ) -> IndexMap<Value, Value> {
        todo!()
    }

    fn map_concat(
        &self,
        this: IndexMap<Value, Value>,
        value: IndexMap<Value, Value>,
    ) -> IndexMap<Value, Value> {
        todo!()
    }

    fn map_entries(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        todo!()
    }

    fn map_keys(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        todo!()
    }

    fn number_ceil(&self, this: Number) -> Number {
        todo!()
    }

    fn number_round(&self, this: Number) -> Number {
        todo!()
    }

    fn number_trunc(&self, this: Number) -> Number {
        todo!()
    }

    fn mut_string_push(&self, this: &mut String, value: Value) {
        todo!()
    }

    fn string_concat(&self, this: String, value: String) -> String {
        todo!()
    }

    fn string_with(&self, this: String, value: Vec<Value>) -> String {
        todo!()
    }

    fn string_trim(&self, this: String) -> String {
        todo!()
    }

    fn format(&self, template: String, args: Vec<Value>) -> String {
        todo!()
    }

    fn printf(&self, template: String, args: Vec<Value>) {
        todo!()
    }
}

#[allow(unused_variables)]
impl<'vm> RigzVM<'vm> for VMModule {
    fn mut_vm_get_register(&self, vm: &mut VM<'vm>, register: Number) -> Result<Value, VMError> {
        todo!()
    }

    fn mut_vm_first(&self, vm: &mut VM<'vm>) -> Result<Value, VMError> {
        todo!()
    }

    fn mut_vm_last(&self, vm: &mut VM<'vm>) -> Result<Value, VMError> {
        todo!()
    }

    fn mut_vm_remove_register(&self, vm: &mut VM<'vm>, register: Number) -> Result<Value, VMError> {
        todo!()
    }
}
//
#[allow(unused_variables)]
impl RigzFile for FileModule {
    fn read(&self, path: String, encoding: String) -> Result<String, VMError> {
        todo!()
    }

    fn write(&self, path: String, contents: String, encoding: String) -> Result<(), VMError> {
        todo!()
    }
}

#[allow(unused_variables)]
impl RigzJSON for JSONModule {
    fn any_to_json(&self, value: Value) -> Result<String, VMError> {
        match serde_json::to_string(&value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to write json - {e}"))),
        }
    }

    fn parse(&self, input: String) -> Result<Value, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to parse json - {e}"))),
        }
    }
}

#[test]
fn blah() {
    let json = JSONModule;
    assert_eq!(
        "5",
        json.call("parse", vec![Rc::new(RefCell::new(5.into()))].into())
            .expect("json parse failed")
            .to_string()
            .as_str()
    )
}
