use crate::{RigzType, VMError, Value};
use indexmap::IndexMap;
use std::fmt::{Debug, Formatter};

pub type Function<'vm> =
    IndexMap<&'vm str, &'vm dyn Fn(Vec<Value<'vm>>) -> Result<Value<'vm>, VMError>>;
pub type ExtensionFunction<'vm> =
    IndexMap<&'vm str, &'vm dyn Fn(Value<'vm>, Vec<Value<'vm>>) -> Result<Value<'vm>, VMError>>;

pub type MutableFunction<'vm> =
    IndexMap<&'vm str, &'vm dyn FnMut(Vec<Value<'vm>>) -> Result<Value<'vm>, VMError>>;
pub type MutableExtensionFunction<'vm> =
    IndexMap<&'vm str, &'vm dyn FnMut(Value<'vm>, Vec<Value<'vm>>) -> Result<Value<'vm>, VMError>>;

#[derive(Clone, Default)]
pub struct Module<'vm> {
    pub name: &'vm str,
    pub functions: Function<'vm>,
    pub extension_functions: IndexMap<RigzType, ExtensionFunction<'vm>>,
    pub mutable_functions: Function<'vm>,
    pub mutable_extension_functions: IndexMap<RigzType, ExtensionFunction<'vm>>,
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
