use crate::{
    AsPrimitive, CreateObject, Definition, Object, ObjectValue, PrimitiveValue, RigzArgs, RigzType,
    VMError, WithTypeInfo,
};
use log::warn;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Hash, PartialOrd, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RigzObject {
    #[serde(skip)]
    pub rigz_type: Arc<RigzType>,
    pub values: Vec<ObjectValue>,
}

impl Default for RigzObject {
    fn default() -> Self {
        Self {
            rigz_type: Arc::new(Default::default()),
            values: vec![],
        }
    }
}

impl RigzObject {
    pub fn new(rigz_type: Arc<RigzType>) -> Self {
        let values = match rigz_type.deref() {
            RigzType::Custom(c) => c.fields.iter().map(|_| ObjectValue::default()).collect(),
            _ => vec![ObjectValue::default()],
        };
        Self { rigz_type, values }
    }
}

impl Debug for RigzObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{{{:?}}}", self.rigz_type, self.values)
    }
}

impl Display for RigzObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl WithTypeInfo for RigzObject {
    fn rigz_type(&self) -> RigzType {
        self.rigz_type.as_ref().clone()
    }
}

impl AsPrimitive<ObjectValue> for RigzObject {
    fn get(&self, attr: &ObjectValue) -> Result<ObjectValue, VMError> {
        if let ObjectValue::Primitive(s) = attr {
            match s {
                PrimitiveValue::Number(n) => {
                    let index = n.to_int();
                    let index = if index.is_negative() {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Negative indexes are not supported yet {index}, {self:?}"
                        )));
                    } else {
                        index as usize
                    };
                    Ok(self.values[index].clone())
                }
                PrimitiveValue::String(n) => {
                    if let RigzType::Custom(c) = self.rigz_type.as_ref() {
                        match c.fields.iter().position(|(f, _)| f == n) {
                            None => Err(VMError::UnsupportedOperation(format!(
                                "Field {n} does not exist on {self:?}"
                            ))),
                            Some(i) => match self.values.get(i) {
                                None => Err(VMError::UnsupportedOperation(format!(
                                    "Field {n}({i}) does not exist on {self:?}"
                                ))),
                                Some(v) => Ok(v.clone()),
                            },
                        }
                    } else {
                        Err(VMError::UnsupportedOperation(format!(
                            "Cannot get {s} from {self:?}"
                        )))
                    }
                }
                _ => Err(VMError::UnsupportedOperation(format!(
                    "Cannot get {s} from {self:?}"
                ))),
            }
        } else {
            Err(VMError::UnsupportedOperation(format!(
                "Cannot get {attr} from {self:?}"
            )))
        }
    }

    fn set(&mut self, attr: &ObjectValue, value: ObjectValue) -> Result<(), VMError> {
        if let ObjectValue::Primitive(p) = attr {
            match p {
                PrimitiveValue::Number(n) => {
                    let index = n.to_int();
                    let index = if index.is_negative() {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Negative indexes are not supported yet {index}, {self:?}"
                        )));
                    } else {
                        index as usize
                    };
                    self.values[index] = value;
                    Ok(())
                }
                PrimitiveValue::String(s) => {
                    if let RigzType::Custom(c) = self.rigz_type.as_ref() {
                        match c.fields.iter().position(|(f, _)| f == s) {
                            None => Err(VMError::UnsupportedOperation(format!(
                                "Field {s} does not exist on {self:?} (set)"
                            ))),
                            Some(i) => match self.values.get_mut(i) {
                                None => Err(VMError::UnsupportedOperation(format!(
                                    "Field {s}({i}) does not exist on {self:?}"
                                ))),
                                Some(v) => {
                                    *v = value;
                                    Ok(())
                                }
                            },
                        }
                    } else {
                        Err(VMError::UnsupportedOperation(format!(
                            "Cannot set {s} from {self:?}"
                        )))
                    }
                }
                _ => Err(VMError::UnsupportedOperation(format!(
                    "Cannot set {attr} on {self:?}"
                ))),
            }
        } else {
            Err(VMError::UnsupportedOperation(format!(
                "Cannot set {attr} on {self:?}"
            )))
        }
    }
}

impl CreateObject for RigzObject {
    fn create(value: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        Err(VMError::UnsupportedOperation(format!(
            "Create Object RigzObject should not be called directly - {value:?}"
        )))
    }
}

impl Definition for RigzObject {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        warn!("Definition::name() called for RigzObject");
        "Object"
    }

    fn trait_definition() -> &'static str
    where
        Self: Sized,
    {
        warn!("Definition::trait_definition() called for RigzObject");
        "object Object\nend"
    }
}

#[typetag::serde]
impl Object for RigzObject {}

impl From<RigzObject> for ObjectValue {
    fn from(value: RigzObject) -> Self {
        ObjectValue::Object(Box::new(value))
    }
}


#[cfg(target_family = "wasm")]
mod wasm_workaround {
    extern "C" {
        pub(super) fn __wasm_call_ctors();
    }
}

// https://github.com/rustwasm/wasm-bindgen/issues/4446
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
fn start() {

    // fix:
    // rigz_core::rigz_object::_::__ctor::h0eda6d1718dbd1e1: Read a negative address value from the stack. Did we run out of memory?
    unsafe { wasm_workaround::__wasm_call_ctors()};

}