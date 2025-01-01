use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"import trait Std
        fn Any.clone -> Any
        fn Any.is_err -> Bool
        fn Any.is_none -> Bool
        fn Any.is_some -> Bool
        fn Any.is(type: Type) -> Bool
        fn Any.is_int -> Bool
        fn Any.is_float -> Bool
        fn Any.is_num -> Bool
        fn Any.to_b -> Bool
        fn Any.to_i -> Int!
        fn Any.to_f -> Float!
        fn Any.to_n -> Number!
        fn Any.to_s -> String
        fn Any.to_list -> List
        fn Any.to_map -> Map
        fn Any.type -> String
        fn Any.get(index) -> Any!?

        fn List.filter(func: |Any| -> Bool) -> List
            [for v in self: v if func v]
        end

        fn List.map(func: |Any| -> Any) -> List
            [for v in self: func v]
        end

        fn Map.filter(func: |Any, Any| -> Bool) -> Map
            {for k, v in self: k, v if func k, v}
        end

        fn Map.map(func: |Any, Any| -> (Any, Any)) -> Map
            {for k, v in self: func k, v}
        end

        fn mut List.extend(value: List)
        fn mut List.clear -> None

        fn List.split_first -> (Any?, List)
        fn List.split_last -> (Any?, List)
        fn List.zip(other: List) -> Map

        fn Map.split_first -> ((Any?, Any?), Map)
        fn Map.split_last -> ((Any?, Any?), Map)

        fn List.to_tuple -> Any
        fn List.reduce(init: Any, func: |Any, Any| -> Any) -> Any
            if !self
                init
            else
                (first, rest) = self.split_first
                next = func init, first
                puts first, init, next, self
                rest.reduce next, func
            end
        end

        fn List.sum -> Number
            self.reduce(0, |res, next| res + next)
        end

        /*
        fn Map.reduce(init: Any, func: |Any, (Any, Any)| -> Any) -> Any
            if !self
                init
            else
                (first, rest) = self.split_first
                next = func init, first
                rest.reduce next, func
            end
        end

        fn Map.sum -> Number
            self.reduce(0, |res, (_, next)| res + next)
        end
        */

        fn List.empty = self.to_bool
        fn List.first -> Any?
        fn List.last -> Any?
        fn mut List.push(var value)
        fn List.concat(value: List) -> List
        fn List.with(var value) -> List

        fn mut Map.extend(value: Map)
        fn mut Map.clear -> None
        fn Map.empty = self.to_bool
        fn Map.first -> Any?
        fn Map.last -> Any?
        fn Map.get_index(number: Number) -> (Any, Any)?!
        fn mut Map.insert(key, value)
        fn Map.with(var key, value) -> Map
        fn Map.concat(value: Map) -> Map
        fn Map.entries -> List
        fn Map.keys -> List
        fn Map.values -> List

        fn assert(condition: Bool, message = '') -> None!
        fn assert_eq(lhs, rhs, message = '') -> None!
        fn assert_neq(lhs, rhs, message = '') -> None!
        fn format(template: String, var args) -> String
        fn printf(template: String, var args) -> None
    end"#
);

fn is_float(s: &str) -> bool {
    let mut float = false;
    for c in s.chars() {
        if c == '.' {
            if float {
                float = false;
                break;
            }
            float = true;
        } else if !c.is_ascii_digit() {
            break;
        }
    }
    float
}

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
        // todo should error count as some?
        !matches!(this, Value::None)
    }

    fn any_is(&self, this: Value, rigz_type: RigzType) -> bool {
        this.rigz_type() == rigz_type
    }

    fn any_is_int(&self, this: Value) -> bool {
        match this {
            Value::Number(Number::Int(_)) => true,
            Value::String(s) => s.trim().chars().all(|c| c.is_ascii_digit()),
            _ => false,
        }
    }

    fn any_is_float(&self, this: Value) -> bool {
        match this {
            Value::Number(Number::Float(_)) => true,
            Value::String(s) => is_float(s.trim()),
            _ => false,
        }
    }

    fn any_is_num(&self, this: Value) -> bool {
        match this {
            Value::Number(_) => true,
            Value::String(s) => {
                let s = s.trim();
                s.chars().all(|c| c.is_ascii_digit()) || is_float(s)
            }
            _ => false,
        }
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

    fn any_get(&self, this: Value, index: Value) -> Result<Option<Value>, VMError> {
        this.get(index)
    }

    fn mut_list_extend(&self, this: &mut Vec<Value>, value: Vec<Value>) {
        this.extend(value)
    }

    fn mut_list_clear(&self, this: &mut Vec<Value>) {
        this.clear()
    }

    fn list_split_first(&self, this: Vec<Value>) -> (Option<Value>, Vec<Value>) {
        match this.split_first() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.clone()), rest.to_vec()),
        }
    }

    fn list_split_last(&self, this: Vec<Value>) -> (Option<Value>, Vec<Value>) {
        match this.split_last() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.clone()), rest.to_vec()),
        }
    }

    fn list_zip(&self, this: Vec<Value>, other: Vec<Value>) -> IndexMap<Value, Value> {
        this.into_iter().zip(other).collect()
    }

    fn map_split_first(
        &self,
        this: IndexMap<Value, Value>,
    ) -> (Option<Value>, Option<Value>, IndexMap<Value, Value>) {
        if this.is_empty() {
            (None, None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some(k.clone()),
                Some(v.clone()),
                this.iter()
                    .skip(1)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        }
    }

    fn map_split_last(
        &self,
        this: IndexMap<Value, Value>,
    ) -> (Option<Value>, Option<Value>, IndexMap<Value, Value>) {
        if this.is_empty() {
            (None, None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some(k.clone()),
                Some(v.clone()),
                this.iter()
                    .rev()
                    .skip(1)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .rev()
                    .collect(),
            )
        }
    }

    fn list_to_tuple(&self, this: Vec<Value>) -> Value {
        Value::Tuple(this)
    }

    fn list_first(&self, this: Vec<Value>) -> Option<Value> {
        this.first().cloned()
    }

    fn list_last(&self, this: Vec<Value>) -> Option<Value> {
        this.last().cloned()
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

    fn mut_map_clear(&self, this: &mut IndexMap<Value, Value>) {
        this.clear()
    }

    fn map_first(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        this.first().map(|(_, v)| v.clone())
    }

    fn map_last(&self, this: IndexMap<Value, Value>) -> Option<Value> {
        this.last().map(|(_, v)| v.clone())
    }

    fn map_get_index(
        &self,
        this: IndexMap<Value, Value>,
        number: Number,
    ) -> Result<Option<(Value, Value)>, VMError> {
        let index = number.to_usize()?;
        Ok(this.get_index(index).map(|(k, v)| (k.clone(), v.clone())))
    }

    fn mut_map_insert(&self, this: &mut IndexMap<Value, Value>, key: Value, value: Value) {
        this.insert(key, value);
    }

    fn map_with(
        &self,
        this: IndexMap<Value, Value>,
        key: Vec<Value>,
        value: Vec<Value>,
    ) -> IndexMap<Value, Value> {
        let mut this = this;
        for (k, v) in key.into_iter().zip(value.into_iter()) {
            this.insert(k, v);
        }
        this
    }

    fn map_concat(
        &self,
        this: IndexMap<Value, Value>,
        value: IndexMap<Value, Value>,
    ) -> IndexMap<Value, Value> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn map_entries(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        this.into_iter()
            .map(|(k, v)| Value::Tuple(vec![k, v]))
            .collect()
    }

    fn map_keys(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        this.keys().cloned().collect()
    }

    fn map_values(&self, this: IndexMap<Value, Value>) -> Vec<Value> {
        this.values().cloned().collect()
    }

    // todo support formatting message
    fn assert(&self, condition: bool, message: String) -> Result<(), VMError> {
        if !condition {
            let message = if message.is_empty() {
                "Assertion Failed".to_string()
            } else {
                format!("Assertion Failed: {message}")
            };
            return Err(VMError::RuntimeError(message));
        }
        Ok(())
    }

    fn assert_eq(&self, lhs: Value, rhs: Value, message: String) -> Result<(), VMError> {
        if lhs == rhs {
            return Ok(());
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
            return Ok(());
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
