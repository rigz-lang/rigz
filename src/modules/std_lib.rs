use rigz_ast::*;
use rigz_ast_derive::derive_module;

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
        fn Map.values -> List

        fn Number.ceil -> Number
        fn Number.round -> Number
        fn Number.trunc -> Number

        fn mut String.push(value)
        fn String.concat(value: String) -> String
        fn String.with(var value) -> String
        fn String.trim -> String

        fn assert(condition: Bool, message = '') -> None!
        fn assert_eq(lhs, rhs, message = '') -> None!
        fn assert_neq(lhs, rhs, message = '') -> None!
        fn format(template: String, var args) -> String
        fn printf(template: String, var args) -> None
    end"#
);

impl RigzStd for StdModule {
    fn any_clone(&self, this: Value) -> Value {
        this.clone()
    }

    fn any_is_err(&self, this: Value) -> bool {
        matches!(this, Value::Error(_))
    }

    fn any_is_none(&self, this: Value) -> bool {
        // todo should error be counted as none?
        matches!(this, Value::None)
    }

    fn any_is_some(&self, this: Value) -> bool {
        !matches!(this, Value::None | Value::Error(_))
    }

    fn any_to_b(&self, this: Value) -> bool {
        this.to_bool()
    }

    fn any_to_i(&self, this: Value) -> Result<i64, VMError> {
        this.to_int()
    }

    fn any_to_f(&self, this: Value) -> Result<f64, VMError> {
        this.to_float()
    }

    fn any_to_n(&self, this: Value) -> Result<Number, VMError> {
        this.to_number()
    }

    fn any_to_s(&self, this: Value) -> String {
        this.to_string()
    }

    fn any_to_list(&self, this: Value) -> Vec<Value> {
        this.to_list()
    }

    fn any_to_map(&self, this: Value) -> IndexMap<Value, Value> {
        this.to_map()
    }

    fn any_type(&self, this: Value) -> String {
        this.rigz_type().to_string()
    }

    fn mut_list_extend(&self, this: &mut Vec<Value>, value: Vec<Value>) {
        this.extend(value)
    }

    fn list_first(&self, this: Vec<Value>) -> Option<Value> {
        this.first().map(|v| v.clone())
    }

    fn list_last(&self, this: Vec<Value>) -> Option<Value> {
        this.last().map(|v| v.clone())
    }

    fn mut_list_push(&self, this: &mut Vec<Value>, value: Vec<Value>) {
        this.extend(value)
    }

    fn list_concat(&self, this: Vec<Value>, value: Vec<Value>) -> Vec<Value> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn list_with(&self, this: Vec<Value>, value: Vec<Value>) -> Vec<Value> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn mut_map_extend(&self, this: &mut IndexMap<Value, Value>, value: IndexMap<Value, Value>) {
        this.extend(value)
    }

    fn map_first(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        this.first().map(|(_, v)| v.clone())
    }

    fn map_last(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        this.last().map(|(_, v)| v.clone())
    }

    fn mut_map_insert(&self, this: &mut IndexMap<Value, Value>, key: Value, value: Value) {
        this.insert(key, value);
    }

    fn map_with(&self, this: IndexMap<Value, Value>, key: Vec<Value>, value: Vec<Value>) -> IndexMap<Value, Value> {
        let mut this = this;
        for (k, v) in key.into_iter().zip(value.into_iter()) {
            this.insert(k, v);
        }
        this
    }

    fn map_concat(&self, this: IndexMap<Value, Value>, value: IndexMap<Value, Value>) -> IndexMap<Value, Value> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn map_entries(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        let mut entries = Vec::new();
        for (k, v) in this {
            entries.push(vec![k, v].into())
        }
        entries
    }

    fn map_keys(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        this.keys().map(|v| v.clone()).collect()
    }

    fn map_values(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        this.values().map(|v| v.clone()).collect()
    }

    fn number_ceil(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => {
                (f.ceil() as i64).into()
            }
        }
    }

    fn number_round(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => {
                (f.round() as i64).into()
            }
        }
    }

    fn number_trunc(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => {
                (f.trunc() as i64).into()
            }
        }
    }

    fn mut_string_push(&self, this: &mut String, value: Value) {
        this.push_str(value.to_string().as_str())
    }

    fn string_concat(&self, this: String, value: String) -> String {
        let mut this = this;
        this.push_str(value.to_string().as_str());
        this
    }

    fn string_with(&self, this: String, value: Vec<Value>) -> String {
        let mut this = this;
        for v in value {
            this.push_str(v.to_string().as_str())
        }
        this
    }

    fn string_trim(&self, this: String) -> String {
        this.trim().to_string()
    }

    // todo support formatting message
    fn assert(&self, condition: bool, message: String) -> Result<(), VMError> {
        if !condition {
            let message = if message.is_empty() {
                "Assertion Failed".to_string()
            } else {
                format!("Assertion Failed: {message}")
            };
            return Err(VMError::RuntimeError(message))
        }
        Ok(())
    }


    fn assert_eq(&self, lhs: Value, rhs: Value, message: String) -> Result<(), VMError> {
        if lhs == rhs {
            return Ok(())
        }

        let base = format!("\tLeft: {lhs}\n\t\tRight: {rhs}");
        let message = if message.is_empty() {
            format!("Assertion Failed\n\t{base}")
        } else {
            format!("Assertion Failed: {message}\n\t{base}")
        };

        Err(VMError::RuntimeError(message))
    }

    fn assert_neq(&self, lhs: Value, rhs: Value, message: String) -> Result<(), VMError> {
        if lhs != rhs {
            return Ok(())
        }

        let base = format!("\tLeft: {lhs}\n\t\tRight: {rhs}");
        let message = if message.is_empty() {
            format!("Assertion Failed\n\t{base}")
        } else {
            format!("Assertion Failed: {message}\n\t{base}")
        };

        Err(VMError::RuntimeError(message))
    }

    fn format(&self, template: String, args: Vec<Value>) -> String {
        let mut res = template;
        for arg in args {
            let l = arg.to_string();
            res = res.replacen("{}", l.as_str(), 1);
        }
        res
    }

    fn printf(&self, template: String, args: Vec<Value>) {
        println!("{}", self.format(template, args))
    }
}