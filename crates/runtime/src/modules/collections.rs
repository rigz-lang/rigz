use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"import trait Collections
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

        fn Map.split_first -> ((Any, Any)?, Map)
        fn Map.split_last -> ((Any, Any)?, Map)

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

        fn Map.reduce(init: Any, func: |Any, Any, Any| -> Any) -> Any
            if !self
                init
            else
                (k, rest) = self.split_first
                (key, first) = k
                next = func key, init, first
                rest.reduce next, func
            end
        end

        fn Map.sum -> Number
            self.reduce(0, |res, _, next| res + next)
        end

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
    end"#
}

impl RigzCollections for CollectionsModule {
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
    ) -> (Option<(Value, Value)>, IndexMap<Value, Value>) {
        if this.is_empty() {
            (None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some((k.clone(), v.clone())),
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
    ) -> (Option<(Value, Value)>, IndexMap<Value, Value>) {
        if this.is_empty() {
            (None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some((k.clone(), v.clone())),
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
}
