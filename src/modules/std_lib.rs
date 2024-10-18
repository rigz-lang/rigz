use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct StdLibModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for StdLibModule {
    fn name(&self) -> &'static str {
        "STD"
    }

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
        match function {
            "format" | "printf" => Err(VMError::RuntimeError(format!(
                "Function {function} is not implemented"
            ))),
            "puts" => {
                println!(
                    "{}",
                    args.into_iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Ok(Value::None)
            }
            _ => Err(VMError::InvalidModuleFunction(format!(
                "Function {function} does not exist"
            ))),
        }
    }

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        match function {
            "first" => match value {
                Value::List(v) => match v.first() {
                    None => Ok(Value::None),
                    Some(v) => Ok(v.clone()),
                },
                Value::Map(v) => match v.first() {
                    None => Ok(Value::None),
                    Some((_, v)) => Ok(v.clone()),
                },
                v => Err(VMError::RuntimeError(format!("Cannot call first on {v}"))),
            },
            "last" => match value {
                Value::List(v) => match v.last() {
                    None => Ok(Value::None),
                    Some(v) => Ok(v.clone()),
                },
                Value::Map(v) => match v.last() {
                    None => Ok(Value::None),
                    Some((_, v)) => Ok(v.clone()),
                },
                v => Err(VMError::RuntimeError(format!("Cannot call last on {v}"))),
            },
            "keys" => match value {
                Value::Map(v) => Ok(Value::List(
                    v.keys().into_iter().map(|k| k.clone()).collect(),
                )),
                v => Err(VMError::RuntimeError(format!("Cannot call keys on {v}"))),
            },
            "values" => match value {
                Value::Map(v) => Ok(Value::List(
                    v.values().into_iter().map(|k| k.clone()).collect(),
                )),
                v => Err(VMError::RuntimeError(format!("Cannot call keys on {v}"))),
            },
            "is_err" => {
                let none = match value {
                    Value::Error(_) => true,
                    _ => false,
                };
                Ok(none.into())
            }
            "is_none" => {
                let none = match value {
                    Value::None => true, // should Error return true here too?
                    _ => false,
                };
                Ok(none.into())
            }
            "is_some" => {
                let none = match value {
                    Value::None | Value::Error(_) => false,
                    _ => true,
                };
                Ok(none.into())
            }
            "to_b" => Ok(value.to_bool().into()),
            "to_f" => match value.to_float() {
                None => Err(VMError::InvalidModuleFunction(format!(
                    "Failed to convert {value} to float"
                ))),
                Some(i) => Ok(i.into()),
            },
            "to_i" => match value.to_int() {
                None => Err(VMError::InvalidModuleFunction(format!(
                    "Failed to convert {value} to int"
                ))),
                Some(i) => Ok(i.into()),
            },
            "to_n" => match value.to_number() {
                None => Err(VMError::InvalidModuleFunction(format!(
                    "Failed to convert {value} to number"
                ))),
                Some(i) => Ok(i.into()),
            },
            "to_s" => Ok(value.to_string().into()),
            "type" => Ok(value.rigz_type().to_string().into()),
            _ => Err(VMError::InvalidModuleFunction(format!(
                "Extension Function {function} does not exist"
            ))),
        }
    }

    fn call_mutable_extension(&self, value: &mut Value, function: &'vm str, args: Vec<Value>) {
        match function {
            "extend" => {
                let len = args.len();
                if len != 1 {
                    *value = VMError::RuntimeError(format!(
                        "Invalid args for parse, expected 1 argument, received {len}"
                    ))
                    .into();
                }
                let mut arg = Value::None;
                for a in args {
                    arg = a;
                    break;
                }
                match (value, arg) {
                    (Value::List(v), Value::List(o)) => v.extend(o),
                    (Value::Map(v), Value::Map(o)) => v.extend(o),
                    (v, o) => {
                        *v = VMError::RuntimeError(format!(
                            "Invalid args for parse, cannot extend {v} with {o}"
                        ))
                        .into();
                    }
                }
            }
            "push" => match value {
                Value::List(v) => v.extend(args),
                Value::String(v) => v.extend(args.into_iter().map(|v| v.to_string())),
                v => {
                    *v = VMError::RuntimeError(format!(
                        "Invalid args for push, cannot push elements to {v}"
                    ))
                    .into();
                }
            },
            _ => {
                *value = VMError::InvalidModuleFunction(format!(
                    "Extension Function {function} does not exist"
                ))
                .into()
            }
        }
    }

    fn trait_definition(&self) -> &'static str {
        r#"import trait STD
            fn Any.is_err -> Bool
            fn Any.is_none -> Bool
            fn Any.is_some -> Bool
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
            fn String.with(var value) -> String
            fn String.trim -> String

            fn format(template: String, var args) -> String
            fn puts(var args)
            fn printf(template: String, var args)
        end"#
    }
}
