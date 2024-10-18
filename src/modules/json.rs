use rigz_vm::{Module, VMError, Value};

#[derive(Copy, Clone)]
pub struct JsonModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for JsonModule {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
        match function {
            "parse" => {
                let len = args.len();
                if len != 1 {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid args for parse, expected 1 argument, received {len}"
                    )));
                }
                let value = args[0].to_string();
                match serde_json::from_str(value.as_str()) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(VMError::RuntimeError(format!(
                        "Unable to deserialize {value} - {e}"
                    ))),
                }
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
            "to_json" => match serde_json::to_string(&value) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(VMError::RuntimeError(format!(
                    "Unable to serialize {value} - {e}"
                ))),
            },
            _ => Err(VMError::InvalidModuleFunction(format!(
                "Function {function} does not exist"
            ))),
        }
    }

    fn trait_definition(&self) -> &'static str {
        r#"import trait JSON
            fn Any.to_json -> String!

            fn parse(input: String) -> Any!
        end"#
    }
}
