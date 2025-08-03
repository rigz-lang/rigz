use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::rc::Rc;

// todo figure this out later
// struct VM {}
//
// derive_module!(
//     r#"trait VM
//         fn mut VM.get_register(register: Number) -> Any!
//         fn mut VM.first -> Any!
//         fn mut VM.last -> Any!
//         fn mut VM.remove_register(register: Number) -> Any!
//     end"#
// );
// #[allow(unused_variables)]
// impl RigzVM for VMModule<'_> {
//     fn mut_vm_get_register(&self, vm: &mut VM, register: Number) -> Result<ObjectValue, VMError> {
//         todo!()
//     }
//
//     fn mut_vm_first(&self, vm: &mut VM) -> Result<ObjectValue, VMError> {
//         todo!()
//     }
//
//     fn mut_vm_last(&self, vm: &mut VM) -> Result<ObjectValue, VMError> {
//         todo!()
//     }
//
//     fn mut_vm_remove_register(
//         &self,
//         vm: &mut VM,
//         register: Number,
//     ) -> Result<ObjectValue, VMError> {
//         todo!()
//     }
// }

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
    fn any_clone(&self, this: ObjectValue) -> ObjectValue {
        todo!()
    }

    fn any_is_err(&self, this: ObjectValue) -> bool {
        todo!()
    }

    fn any_is_none(&self, this: ObjectValue) -> bool {
        todo!()
    }

    fn any_is_some(&self, this: ObjectValue) -> bool {
        todo!()
    }

    fn any_to_b(&self, this: ObjectValue) -> bool {
        todo!()
    }

    fn any_to_i(&self, this: ObjectValue) -> Result<i64, VMError> {
        todo!()
    }

    fn any_to_f(&self, this: ObjectValue) -> Result<f64, VMError> {
        todo!()
    }

    fn any_to_n(&self, this: ObjectValue) -> Result<Number, VMError> {
        todo!()
    }

    fn any_to_s(&self, this: ObjectValue) -> String {
        todo!()
    }

    fn any_to_list(&self, this: ObjectValue) -> Vec<ObjectValue> {
        todo!()
    }

    fn any_to_map(&self, this: ObjectValue) -> IndexMap<ObjectValue, ObjectValue> {
        todo!()
    }

    fn any_type(&self, this: ObjectValue) -> String {
        todo!()
    }

    fn mut_list_extend(&self, this: &mut Vec<ObjectValue>, value: Vec<ObjectValue>) {
        todo!()
    }

    fn list_first(&self, this: Vec<ObjectValue>) -> Option<ObjectValue> {
        todo!()
    }

    fn list_last(&self, this: Vec<ObjectValue>) -> Option<ObjectValue> {
        todo!()
    }

    fn mut_list_push(&self, this: &mut Vec<ObjectValue>, value: Vec<ObjectValue>) {
        todo!()
    }

    fn list_concat(&self, this: Vec<ObjectValue>, value: Vec<ObjectValue>) -> Vec<ObjectValue> {
        todo!()
    }

    fn list_with(&self, this: Vec<ObjectValue>, value: Vec<ObjectValue>) -> Vec<ObjectValue> {
        todo!()
    }

    fn mut_map_extend(
        &self,
        this: &mut IndexMap<ObjectValue, ObjectValue>,
        value: IndexMap<ObjectValue, ObjectValue>,
    ) {
        todo!()
    }

    fn map_first(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Option<ObjectValue> {
        todo!()
    }

    fn map_last(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Option<ObjectValue> {
        todo!()
    }

    fn mut_map_insert(
        &self,
        this: &mut IndexMap<ObjectValue, ObjectValue>,
        key: ObjectValue,
        value: ObjectValue,
    ) {
        todo!()
    }

    fn map_with(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
        key: Vec<ObjectValue>,
        value: Vec<ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        todo!()
    }

    fn map_concat(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
        value: IndexMap<ObjectValue, ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        todo!()
    }

    fn map_entries(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Vec<ObjectValue> {
        todo!()
    }

    fn map_keys(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Vec<ObjectValue> {
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

    fn mut_string_push(&self, this: &mut String, value: ObjectValue) {
        todo!()
    }

    fn string_concat(&self, this: String, value: String) -> String {
        todo!()
    }

    fn string_with(&self, this: String, value: Vec<ObjectValue>) -> String {
        todo!()
    }

    fn string_trim(&self, this: String) -> String {
        todo!()
    }

    fn format(&self, template: String, args: Vec<ObjectValue>) -> String {
        todo!()
    }

    fn printf(&self, template: String, args: Vec<ObjectValue>) {
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
    fn any_to_json(&self, value: ObjectValue) -> Result<String, VMError> {
        match serde_json::to_string(&value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::runtime(format!("Failed to write json - {e}"))),
        }
    }

    fn parse(&self, input: String) -> Result<ObjectValue, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::runtime(format!("Failed to parse json - {e}"))),
        }
    }
}

use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn blah() {
    let json = JSONModule;
    assert_eq!(
        "5",
        json.call(
            "parse",
            vec![Rc::new(RefCell::new(5.into()))].into()
        )
        .expect("json parse failed")
        .to_string()
        .as_str()
    )
}
