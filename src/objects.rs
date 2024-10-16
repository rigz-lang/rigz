use std::hash::{Hash, Hasher};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use crate::{VMError, Value};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RigzType {
    None,
    Any,
    Bool,
    Int,
    Float,
    Number,
    UInt,
    String,
    List,
    Map,
    Error,
    Function(Vec<RigzType>, Box<RigzType>),
    Object(RigzObjectDefinition),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigzObjectDefinition {
    pub name: String,
    pub fields: IndexMap<String, RigzType>,
}

impl Hash for RigzObjectDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in &self.fields {
            k.hash(state);
            v.hash(state);
        }
    }
}

pub(crate) static NONE: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "None".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::None)]),
});

pub(crate) static BOOL: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Bool".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Bool)]),
});

pub(crate) static NUMBER: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Number".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Number)]),
});

pub(crate) static STRING: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "String".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::String)]),
});

pub(crate) static ERROR: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Error".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Error)]),
});

pub(crate) static LIST: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "List".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::List)]),
});

pub(crate) static MAP: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Map".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Map)]),
});

#[derive(Clone, Debug, PartialEq)]
pub struct RigzObject<'vm> {
    pub fields: IndexMap<String, Value<'vm>>,
    pub definition_index: &'vm RigzObjectDefinition,
}

impl<'vm> RigzObject<'vm> {
    pub fn cast(&self, def: RigzObjectDefinition) -> Result<RigzObject<'vm>, VMError> {
        if self.definition_index == &def {
            return Ok(self.clone());
        }

        if self.definition_index.fields == def.fields {
            return Ok(self.clone());
        }

        Err(VMError::ConversionError(format!(
            "Cannot convert {} to {}",
            self.definition_index.name, def.name
        )))
    }
}

impl<'vm> RigzObject<'vm> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn equivalent(&self, other: &IndexMap<Value<'vm>, Value<'vm>>) -> bool {
        for (k, v) in other {
            let key = k.to_string();
            if !self.fields.contains_key(&key) {
                return false;
            }
            match self.fields.get(&key) {
                None => return false,
                Some(o) => {
                    if !o.eq(v) {
                        return false;
                    }
                }
            };
        }
        true
    }
}

impl<'vm> Hash for RigzObject<'vm> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.definition_index.name.hash(state);
        for (k, v) in &self.fields {
            k.hash(state);
            v.hash(state);
        }
    }
}