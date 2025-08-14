use itertools::Itertools;
use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;

derive_module! {
    r#"import trait Collections
        fn Set.each(func: |Any| -> Any)
            for v in self = func v
        end

        fn Set.filter(func: |Any| -> Bool) -> Set
            Set[for v in self: v if func v]
        end

        fn Set.map(func: |Any| -> Any) -> Set
            Set[for v in self: func v]
        end

        fn List.each(func: |Any| -> Any)
            for v in self = func v
        end

        fn List.filter(func: |Any| -> Bool) -> List
            [for v in self: v if func v]
        end

        fn List.map(func: |Any| -> Any) -> List
            [for v in self: func v]
        end

        fn Map.each(func: |Any, Any| -> (Any, Any))
            for k, v in self = func k, v
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
            mut result = init
            for v in self
                result = func result, v
            end
            result
        end

        fn Set.sum -> Number
            self.reduce(0, |prev, res| prev + res)
        end

        fn List.to_tuple -> Any
        fn List.reduce(init: Any, func: |Any, Any| -> Any) -> Any
            mut result = init
            for v in self
                result = func result, v
            end
            result
        end

        fn List.sum -> Number
            self.reduce(0, |prev, res| prev + res)
        end

        fn Map.reduce(init: Any, func: |Any, Any, Any| -> Any) -> Any
            mut result = init
            for k, v in self
                result = func result, k, v
            end
            result
        end

        fn Map.sum -> Number
            self.reduce(0, |res, _, nxt| res + nxt)
        end

        fn List.len -> Int
        fn List.empty = !self.to_b
        fn List.first -> Any?
        fn List.last -> Any?
        fn List.nth(number: Number) -> Any?!
        fn mut List.shift -> Any?
        fn mut List.push(var value)
        fn List.concat(value: List) -> List
        fn List.with(var value) -> List
        fn List.has(value) -> Bool

        fn Set.len -> Int
        fn Set.empty = !self.to_b
        fn Set.first -> Any?
        fn Set.last -> Any?
        fn Set.nth(number: Number) -> Any?!
        fn mut Set.insert(var value)
        fn mut Set.remove(value) -> Bool
        fn Set.concat(value: Set) -> Set
        fn Set.with(var value) -> Set
        fn Set.has(value) -> Bool

        fn Map.len -> Int
        fn mut Map.extend(value: Map)
        fn mut Map.clear -> None
        fn Map.empty = !self.to_b
        fn Map.first -> Any?
        fn Map.last -> Any?
        fn Map.nth(number: Number) -> (Any, Any)?!
        fn mut Map.insert(key, value)
        fn mut Map.remove(key) -> Any?
        fn Map.with(var key, value) -> Map
        fn Map.concat(value: Map) -> Map
        fn Map.entries -> List = self.to_list
        fn Map.keys -> List
        fn Map.values -> List
        fn Map.has(key) -> Bool
        fn Map.has_value(value) -> Bool

        fn List.group_by(func: |Any| -> Any)
            mut result = {}
            # todo support using reduce with lambda here
            for v in self
                val = func v
                if result.has val
                    # todo support += and .push
                    result[val] = result[val] + v
                else
                    result.insert val, [v]
                end
            end
            result
        end
    end"#
}

impl RigzCollections for CollectionsModule {
    fn mut_set_extend(&self, this: &mut IndexSet<ObjectValue>, value: IndexSet<ObjectValue>) {
        this.extend(value)
    }

    fn mut_set_clear(&self, this: &mut IndexSet<ObjectValue>) {
        this.clear()
    }

    fn mut_list_extend(&self, this: &mut Vec<Rc<RefCell<ObjectValue>>>, value: Vec<Rc<RefCell<ObjectValue>>>) {
        this.extend(value)
    }

    fn mut_list_clear(&self, this: &mut Vec<Rc<RefCell<ObjectValue>>>) {
        this.clear()
    }

    fn mut_set_sort(&self, this: &mut IndexSet<ObjectValue>) {
        this.sort_unstable()
    }

    fn mut_list_sort(&self, this: &mut Vec<Rc<RefCell<ObjectValue>>>) {
        this.sort_unstable()
    }

    fn mut_map_sort(&self, this: &mut IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) {
        this.sort_unstable_keys()
    }

    fn set_split_first(
        &self,
        this: &IndexSet<ObjectValue>,
    ) -> (Option<ObjectValue>, IndexSet<ObjectValue>) {
        if this.is_empty() {
            (None, IndexSet::new())
        } else {
            let first = this.first().unwrap();
            (Some(first.clone()), this.iter().skip(1).cloned().collect())
        }
    }

    fn set_split_last(
        &self,
        this: &IndexSet<ObjectValue>,
    ) -> (Option<ObjectValue>, IndexSet<ObjectValue>) {
        if this.is_empty() {
            (None, IndexSet::new())
        } else {
            let last = this.last().unwrap();
            (
                Some(last.clone()),
                this.iter().rev().skip(1).cloned().rev().collect(),
            )
        }
    }

    fn set_zip(
        &self,
        this: &IndexSet<ObjectValue>,
        other: IndexSet<ObjectValue>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        this.iter().cloned().zip(other).collect()
    }

    fn list_split_first(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> (Option<ObjectValue>, Vec<ObjectValue>) {
        match this.split_first() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.borrow().clone()), rest.iter().map(|v| v.borrow().clone()).collect()),
        }
    }

    fn list_split_last(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> (Option<ObjectValue>, Vec<ObjectValue>) {
        match this.split_last() {
            None => (None, vec![]),
            Some((s, rest)) => (Some(s.borrow().clone()), rest.iter().map(|v| v.borrow().clone()).collect()),
        }
    }

    fn list_zip(
        &self,
        this: &Vec<Rc<RefCell<ObjectValue>>>,
        other: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        this.iter().cloned().zip(other).map(|(k, v)| (k.borrow().clone(), v.borrow().clone())).collect()
    }

    fn map_split_first(
        &self,
        this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
    ) -> (
        Option<(ObjectValue, ObjectValue)>,
        IndexMap<ObjectValue, ObjectValue>,
    ) {
        if this.is_empty() {
            (None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some((k.clone().into(), v.borrow().clone())),
                this.iter()
                    .skip(1)
                    .map(|(k, v)| (k.clone(), v.borrow().clone()))
                    .collect(),
            )
        }
    }

    fn map_split_last(
        &self,
        this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
    ) -> (
        Option<(ObjectValue, ObjectValue)>,
        IndexMap<ObjectValue, ObjectValue>,
    ) {
        if this.is_empty() {
            (None, IndexMap::new())
        } else {
            let (k, v) = this.first().unwrap();
            (
                Some((k.clone().into(), v.borrow().clone())),
                this.iter()
                    .rev()
                    .skip(1)
                    .map(|(k, v)| (k.clone(), v.borrow().clone()))
                    .rev()
                    .collect(),
            )
        }
    }

    fn set_to_tuple(&self, this: &IndexSet<ObjectValue>) -> ObjectValue {
        ObjectValue::Tuple(this.iter().cloned().map(|v| v.into()).collect())
    }

    fn list_to_tuple(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> ObjectValue {
        ObjectValue::Tuple(this.clone())
    }

    fn list_len(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> i64 {
        this.len() as i64
    }

    fn list_first(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> Option<ObjectValue> {
        this.first().map(|b| b.borrow().clone())
    }

    fn list_last(&self, this: &Vec<Rc<RefCell<ObjectValue>>>) -> Option<ObjectValue> {
        this.last().map(|b| b.borrow().clone())
    }

    fn list_nth(
        &self,
        this: &Vec<Rc<RefCell<ObjectValue>>>,
        number: Number,
    ) -> Result<Option<ObjectValue>, VMError> {
        let index = number.to_usize()?;
        Ok(this.get(index).map(|b| b.borrow().clone()))
    }

    fn mut_list_shift(&self, this: &mut Vec<Rc<RefCell<ObjectValue>>>) -> Option<ObjectValue> {
        if this.is_empty() {
            None
        } else {
            Some(this.remove(0).borrow().clone())
        }
    }

    fn mut_list_push(&self, this: &mut Vec<Rc<RefCell<ObjectValue>>>, value: Vec<Rc<RefCell<ObjectValue>>>) {
        this.extend(value)
    }

    fn list_concat(&self, this: &Vec<Rc<RefCell<ObjectValue>>>, value: Vec<Rc<RefCell<ObjectValue>>>) -> Vec<ObjectValue> {
        let mut this = this.clone();
        this.extend(value);
        this.into_iter().map(|v| v.borrow().clone()).collect()
    }

    fn list_with(&self, this: &Vec<Rc<RefCell<ObjectValue>>>, value: Vec<Rc<RefCell<ObjectValue>>>) -> Vec<ObjectValue> {
        let mut this = this.clone();
        this.extend(value);
        this.into_iter().map(|v| v.borrow().clone()).collect()
    }

    fn list_has(&self, this: &Vec<Rc<RefCell<ObjectValue>>>, value: Rc<RefCell<ObjectValue>>) -> bool {
        this.contains(&value)
    }

    fn set_len(&self, this: &IndexSet<ObjectValue>) -> i64 {
        this.len() as i64
    }

    fn set_first(&self, this: &IndexSet<ObjectValue>) -> Option<ObjectValue> {
        this.first().cloned()
    }

    fn set_last(&self, this: &IndexSet<ObjectValue>) -> Option<ObjectValue> {
        this.last().cloned()
    }

    fn set_nth(
        &self,
        this: &IndexSet<ObjectValue>,
        number: Number,
    ) -> Result<Option<ObjectValue>, VMError> {
        let index = number.to_usize()?;
        Ok(this.get_index(index).map(|v| v.clone().into()))
    }

    fn mut_set_insert(&self, this: &mut IndexSet<ObjectValue>, value: Vec<Rc<RefCell<ObjectValue>>>) {
        this.extend(value.into_iter().map(|v| v.borrow().clone()) )
    }

    fn mut_set_remove(&self, this: &mut IndexSet<ObjectValue>, value: Rc<RefCell<ObjectValue>>) -> bool {
        this.shift_remove(value.borrow().deref())
    }

    fn set_concat(
        &self,
        this: &IndexSet<ObjectValue>,
        value: IndexSet<ObjectValue>,
    ) -> IndexSet<ObjectValue> {
        let mut this = this.clone();
        this.extend(value);
        this
    }

    fn set_with(
        &self,
        this: &IndexSet<ObjectValue>,
        value: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> IndexSet<ObjectValue> {
        let mut this = this.clone();
        this.extend(value.into_iter().map(|v| v.borrow().clone()));
        this
    }

    fn set_has(&self, this: &IndexSet<ObjectValue>, value: Rc<RefCell<ObjectValue>>) -> bool {
        this.contains(value.borrow().deref())
    }

    fn map_len(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) -> i64 {
        this.len() as i64
    }

    fn mut_map_extend(
        &self,
        this: &mut IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
        value: IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
    ) {
        this.extend(value)
    }

    fn mut_map_clear(&self, this: &mut IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) {
        this.clear()
    }

    fn map_first(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) -> Option<ObjectValue> {
        this.first().map(|(_, v)| v.borrow().clone())
    }

    fn map_last(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) -> Option<ObjectValue> {
        this.last().map(|(_, v)| v.borrow().clone())
    }

    fn map_nth(
        &self,
        this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
        number: Number,
    ) -> Result<Option<(ObjectValue, ObjectValue)>, VMError> {
        let index = number.to_usize()?;
        Ok(this.get_index(index).map(|(k, v)| (k.clone(), v.borrow().clone())))
    }

    fn mut_map_insert(
        &self,
        this: &mut IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
        key: Rc<RefCell<ObjectValue>>,
        value: Rc<RefCell<ObjectValue>>,
    ) {
        this.insert(key.borrow().clone(), value);
    }

    fn mut_map_remove(&self, this: &mut IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>, key: Rc<RefCell<ObjectValue>>) -> Option<ObjectValue> {
        this.shift_remove(key.borrow().deref()).map(|b| b.borrow().clone())
    }

    fn map_with(
        &self,
        this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
        key: Vec<Rc<RefCell<ObjectValue>>>,
        value: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        let mut this: IndexMap<_, _> = this.into_iter().map(|(k, v)| (k.clone(), v.borrow().clone())).collect();
        for (k, v) in key.into_iter().zip(value.into_iter()) {
            this.insert(k.borrow().clone(), v.borrow().clone());
        }
        this
    }

    fn map_concat(
        &self,
        this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
        value: IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        let mut this: IndexMap<_, _> = this.into_iter().map(|(k, v)| (k.clone(), v.borrow().clone())).collect();
        this.extend(value.into_iter().map(|(k, v)| (k, v.borrow().clone())));
        this
    }

    fn map_keys(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) -> Vec<ObjectValue> {
        this.keys().map(|k| k.clone().into()).collect()
    }

    fn map_values(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>) -> Vec<ObjectValue> {
        this.values().map(|v| v.borrow().clone()).collect()
    }

    fn map_has(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>, value: Rc<RefCell<ObjectValue>>) -> bool {
        this.contains_key(value.borrow().deref())
    }

    fn map_has_value(&self, this: &IndexMap<ObjectValue, Rc<RefCell<ObjectValue>>>, value: Rc<RefCell<ObjectValue>>) -> bool {
        this.values().contains(&value)
    }
}
