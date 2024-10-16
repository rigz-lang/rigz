use std::fmt::{Debug, Formatter};
use indexmap::IndexMap;
use crate::{RigzType, Value};

pub type Function<'vm> = IndexMap<&'vm str, &'vm dyn Fn(Vec<Value<'vm>>) -> Value<'vm>>;
pub type ExtensionFunction<'vm> =
IndexMap<&'vm str, &'vm dyn Fn(Value<'vm>, Vec<Value<'vm>>) -> Value<'vm>>;

#[derive(Clone)]
pub struct Module<'vm> {
    pub name: &'vm str,
    pub functions: Function<'vm>,
    pub extension_functions: IndexMap<RigzType, ExtensionFunction<'vm>>,
}

impl<'vm> Debug for Module<'vm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut extension_debug = String::new();
        for (k, v) in &self.extension_functions {
            extension_debug
                .push_str(format!("type={:?}, functions={:?}", k.clone(), v.keys()).as_str());
            extension_debug.push(';');
        }
        write!(
            f,
            "Module {{name={}, functions={:?}, extension_functions={}}}",
            self.name,
            self.functions.keys(),
            extension_debug
        )
    }
}