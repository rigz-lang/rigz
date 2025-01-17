// mod objects;

use rigz_core::{AsPrimitive, IndexMap, Object, ObjectValue, PrimitiveValue, RigzType, VMError};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
// use crate::objects::{Boolean, NoneObjectModule, NumberObject};

// pub trait AsObject: AsPrimitive<Self> {
//     fn as_object(&self) -> Box<dyn Object>;
// }
//
// impl AsObject for PrimitiveValue {
//     fn as_object(&self) -> Box<dyn Object> {
//         match self {
//             PrimitiveValue::None => Box::new(NoneObjectModule),
//             PrimitiveValue::Bool(b) => Box::new(Boolean { value: *b }),
//             PrimitiveValue::Number(n) => Box::new(NumberObject { value: *n }),
//             PrimitiveValue::String(_) => {}
//             PrimitiveValue::Range(_) => {}
//             PrimitiveValue::Error(_) => {}
//             PrimitiveValue::Type(_) => {}
//         }
//     }
// }
//
// impl AsObject for ObjectValue {
//     fn as_object(&self) -> Box<dyn Object> {
//         match self {
//             ObjectValue::Primitive(p) => p.as_object(),
//             ObjectValue::Object(o) => o.clone(),
//             ObjectValue::List(_) => {}
//             ObjectValue::Map(_) => {}
//             ObjectValue::Tuple(_) => {}
//         }
//     }
// }

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
