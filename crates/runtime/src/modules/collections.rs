use itertools::Itertools;
use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"import trait Collections
        fn Set.each(func: |Any| -> Any)
            for v in self = func v
            none
        end

        fn Set.filter(func: |Any| -> Bool) -> Set
            Set.new [for v in self: v if func v]
        end

        fn Set.map(func: |Any| -> Any) -> Set
            Set.new [for v in self: func v]
        end

        fn List.each(func: |Any| -> Any)
            for v in self = func v
            none
        end

        fn List.filter(func: |Any| -> Bool) -> List
            [for v in self: v if func v]
        end

        fn List.map(func: |Any| -> Any) -> List
            [for v in self: func v]
        end

        fn Map.each(func: |Any, Any| -> (Any, Any))
            for k, v in self = func k, v
            none
        end

        fn Map.filter(func: |Any, Any| -> Bool) -> Map
            {for k, v in self: k, v if func k, v}
        end

        fn Map.map(func: |Any, Any| -> (Any, Any)) -> Map
            {for k, v in self: func k, v}
        end

        fn mut Set.extend(value: Set)
        fn mut Set.clear -> None
        fn mut List.extend(value: List)
        fn mut List.clear -> None

        fn mut Set.sort
        fn mut List.sort
        fn mut Map.sort

        fn Set.split_first -> (Any?, Set)
        fn Set.split_last -> (Any?, Set)
        fn Set.zip(other: Set) -> Map
        fn List.split_first -> (Any?, List)
        fn List.split_last -> (Any?, List)
        fn List.zip(other: List) -> Map

        fn Map.split_first -> ((Any, Any)?, Map)
        fn Map.split_last -> ((Any, Any)?, Map)

        fn Set.to_tuple -> Any
        fn Set.reduce(init: Any, func: |Any, Any| -> Any) -> Any
            if !self
                init
            else
                (first, rest) = self.split_first
                res = func init, first
                rest.reduce res, func
            end
        end

        fn Set.sum -> Number
            self.reduce(0, |prev, res| prev + res)
        end

        fn List.to_tuple -> Any
        fn List.reduce(init: Any, func: |Any, Any| -> Any) -> Any
            if !self
                init
            else
                (first, rest) = self.split_first
                res = func init, first
                rest.reduce res, func
            end
        end

        fn List.sum -> Number
            self.reduce(0, |prev, res| prev + res)
        end

        fn Map.reduce(init: Any, func: |Any, Any, Any| -> Any) -> Any
            if !self
                init
            else
                (k, rest) = self.split_first
                (key, first) = k
                nxt = func init, key, first
                rest.reduce nxt, func
            end
        end

        fn Map.sum -> Number
            self.reduce(0, |res, _, nxt| res + nxt)
        end

        fn List.empty = !self.to_b
        fn List.first -> Any?
        fn List.last -> Any?
        fn mut List.push(var value)
        fn List.concat(value: List) -> List
        fn List.with(var value) -> List

        fn Set.empty = !self.to_b
        fn Set.first -> Any?
        fn Set.last -> Any?
        fn mut Set.insert(var value)
        fn Set.concat(value: Set) -> Set
        fn Set.with(var value) -> Set

        fn mut Map.extend(value: Map)
        fn mut Map.clear -> None
        fn Map.empty = !self.to_b
        fn Map.first -> Any?
        fn Map.last -> Any?
        fn Map.get_index(number: Number) -> (Any, Any)?!
        fn mut Map.insert(key, value)
        fn Map.with(var key, value) -> Map
        fn Map.concat(value: Map) -> Map
        fn Map.entries = self.to_list
        fn Map.keys -> List
        fn Map.values -> List
    end"#
}

impl RigzCollections for CollectionsModule {
    fn mut_set_extend(&self, this: &mut IndexSet<ObjectValue>, value: IndexSet<ObjectValue>) {
        this.extend(value)
    }

    fn mut_set_clear(&self, this: &mut IndexSet<ObjectValue>) {
        this.clear()
    }

    fn mut_list_extend(&self, this: &mut Vec<ObjectValue>, value: Vec<ObjectValue>) {
        this.extend(value)
    }

    fn mut_list_clear(&self, this: &mut Vec<ObjectValue>) {
        this.clear()
    }

    fn mut_set_sort(&self, this: &mut IndexSet<ObjectValue>) {
        this.sort()
    }

    fn mut_list_sort(&self, this: &mut Vec<ObjectValue>) {
        this.sort()
    }

    fn mut_map_sort(&self, this: &mut IndexMap<ObjectValue, ObjectValue>) {
        *this = this
            .into_iter()
            .sorted()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }

    fn set_split_first(&self, this: IndexSet<ObjectValue>) -> (Option<ObjectValue>, IndexSet<ObjectValue>) {
        if this.is_empty() {
            (None, IndexSet::new())
        } else {
            let first = this.first().unwrap();
            (Some(first.clone()), this.iter()
                .skip(1)
                .cloned()
                .collect())
        }
    }

    fn set_split_last(&self, this: IndexSet<ObjectValue>) -> (Option<ObjectValue>, IndexSet<ObjectValue>) {
        if this.is_empty() {
            (None, IndexSet::new())
        } else {
            let last = this.last().unwrap();
            (Some(last.clone()), this.iter()
                .rev()
                .skip(1)
                .cloned()
                .rev()
                .collect())
        }
    }

    fn set_zip(&self, this: IndexSet<ObjectValue>, other: IndexSet<ObjectValue>) -> IndexMap<ObjectValue, ObjectValue> {
        this.into_iter().zip(other).collect()
    }

    fn list_split_first(&self, this: Vec<ObjectValue>) -> (Option<ObjectValue>, Vec<ObjectValue>) {
        match this.split_first() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.clone()), rest.to_vec()),
        }
    }

    fn list_split_last(&self, this: Vec<ObjectValue>) -> (Option<ObjectValue>, Vec<ObjectValue>) {
        match this.split_last() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.clone()), rest.to_vec()),
        }
    }

    fn list_zip(
        &self,
        this: Vec<ObjectValue>,
        other: Vec<ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        this.into_iter().zip(other).collect()
    }

    fn map_split_first(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
    ) -> (
        Option<(ObjectValue, ObjectValue)>,
        IndexMap<ObjectValue, ObjectValue>,
    ) {
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
        this: IndexMap<ObjectValue, ObjectValue>,
    ) -> (
        Option<(ObjectValue, ObjectValue)>,
        IndexMap<ObjectValue, ObjectValue>,
    ) {
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

    fn set_to_tuple(&self, this: IndexSet<ObjectValue>) -> ObjectValue {
        ObjectValue::Tuple(this.into_iter().collect())
    }

    fn list_to_tuple(&self, this: Vec<ObjectValue>) -> ObjectValue {
        ObjectValue::Tuple(this)
    }

    fn list_first(&self, this: Vec<ObjectValue>) -> Option<ObjectValue> {
        this.first().cloned()
    }

    fn list_last(&self, this: Vec<ObjectValue>) -> Option<ObjectValue> {
        this.last().cloned()
    }

    fn mut_list_push(&self, this: &mut Vec<ObjectValue>, value: Vec<ObjectValue>) {
        this.extend(value)
    }

    fn list_concat(&self, this: Vec<ObjectValue>, value: Vec<ObjectValue>) -> Vec<ObjectValue> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn list_with(&self, this: Vec<ObjectValue>, value: Vec<ObjectValue>) -> Vec<ObjectValue> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn set_first(&self, this: IndexSet<ObjectValue>) -> Option<ObjectValue> {
        this.first().cloned()
    }

    fn set_last(&self, this: IndexSet<ObjectValue>) -> Option<ObjectValue> {
        this.last().cloned()
    }

    fn mut_set_insert(&self, this: &mut IndexSet<ObjectValue>, value: Vec<ObjectValue>) {
        this.extend(value)
    }

    fn set_concat(&self, this: IndexSet<ObjectValue>, value: IndexSet<ObjectValue>) -> IndexSet<ObjectValue> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn set_with(&self, this: IndexSet<ObjectValue>, value: Vec<ObjectValue>) -> IndexSet<ObjectValue> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn mut_map_extend(
        &self,
        this: &mut IndexMap<ObjectValue, ObjectValue>,
        value: IndexMap<ObjectValue, ObjectValue>,
    ) {
        this.extend(value)
    }

    fn mut_map_clear(&self, this: &mut IndexMap<ObjectValue, ObjectValue>) {
        this.clear()
    }

    fn map_first(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Option<ObjectValue> {
        this.first().map(|(_, v)| v.clone())
    }

    fn map_last(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Option<ObjectValue> {
        this.last().map(|(_, v)| v.clone())
    }

    fn map_get_index(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
        number: Number,
    ) -> Result<Option<(ObjectValue, ObjectValue)>, VMError> {
        let index = number.to_usize()?;
        Ok(this.get_index(index).map(|(k, v)| (k.clone(), v.clone())))
    }

    fn mut_map_insert(
        &self,
        this: &mut IndexMap<ObjectValue, ObjectValue>,
        key: ObjectValue,
        value: ObjectValue,
    ) {
        this.insert(key, value);
    }

    fn map_with(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
        key: Vec<ObjectValue>,
        value: Vec<ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        let mut this = this;
        for (k, v) in key.into_iter().zip(value.into_iter()) {
            this.insert(k, v);
        }
        this
    }

    fn map_concat(
        &self,
        this: IndexMap<ObjectValue, ObjectValue>,
        value: IndexMap<ObjectValue, ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        let mut this = this;
        this.extend(value);
        this
    }

    fn map_keys(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Vec<ObjectValue> {
        this.keys().cloned().collect()
    }

    fn map_values(&self, this: IndexMap<ObjectValue, ObjectValue>) -> Vec<ObjectValue> {
        this.values().cloned().collect()
    }
}
