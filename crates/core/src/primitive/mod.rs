mod error;

mod ops;
#[cfg(feature = "snapshot")]
mod snapshot;
mod value_range;

pub use error::VMError;
pub use value_range::ValueRange;

use std::cell::RefCell;

use crate::{impl_from, AsPrimitive, Number, RigzType};
use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PrimitiveValue {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(String),
    Range(ValueRange),
    Error(VMError),
    // todo create dedicated object value to avoid map usage everywhere, might need to be a trait. Create to_o method for value
    Type(RigzType),
}

impl From<PrimitiveValue> for Rc<RefCell<PrimitiveValue>> {
    #[inline]
    fn from(value: PrimitiveValue) -> Self {
        Rc::new(RefCell::new(value))
    }
}

impl_from! {
    bool, PrimitiveValue, PrimitiveValue::Bool;
    VMError, PrimitiveValue, PrimitiveValue::Error;
    String, PrimitiveValue, PrimitiveValue::String;
    ValueRange, PrimitiveValue, PrimitiveValue::Range;
    RigzType, PrimitiveValue, PrimitiveValue::Type;
}

impl From<&'_ str> for PrimitiveValue {
    #[inline]
    fn from(value: &'_ str) -> Self {
        PrimitiveValue::String(value.to_string())
    }
}

impl<T: Into<Number>> From<T> for PrimitiveValue {
    #[inline]
    fn from(value: T) -> Self {
        PrimitiveValue::Number(value.into())
    }
}

impl From<()> for PrimitiveValue {
    #[inline]
    fn from(_value: ()) -> Self {
        PrimitiveValue::None
    }
}

impl AsPrimitive<PrimitiveValue> for PrimitiveValue {
    fn rigz_type(&self) -> RigzType {
        match self {
            PrimitiveValue::None => RigzType::None,
            PrimitiveValue::Bool(_) => RigzType::Bool,
            PrimitiveValue::Number(_) => RigzType::Number,
            PrimitiveValue::String(_) => RigzType::String,
            PrimitiveValue::Range(_) => RigzType::Range,
            PrimitiveValue::Error(_) => RigzType::Error,
            PrimitiveValue::Type(r) => r.clone(),
        }
    }

    fn to_list(&self) -> Result<Vec<PrimitiveValue>, VMError> {
        if let PrimitiveValue::Range(r) = self {
            Ok(r.to_list())
        } else {
            Err(VMError::RuntimeError(format!(
                "Cannot convert {self} to List"
            )))
        }
    }

    fn to_map(&self) -> Result<IndexMap<PrimitiveValue, PrimitiveValue>, VMError> {
        if let PrimitiveValue::Range(r) = self {
            Ok(r.to_map())
        } else {
            Err(VMError::RuntimeError(format!(
                "Cannot convert {self} to Map"
            )))
        }
    }

    #[inline]
    fn to_number(&self) -> Result<Number, VMError> {
        match self {
            PrimitiveValue::None => Ok(Number::zero()),
            PrimitiveValue::Bool(b) => {
                let n = if *b { Number::one() } else { Number::zero() };
                Ok(n)
            }
            PrimitiveValue::Number(n) => Ok(*n),
            PrimitiveValue::String(s) => match s.parse() {
                Ok(n) => Ok(n),
                Err(e) => Err(VMError::ConversionError(format!(
                    "Cannot convert {s} to Number: {e}"
                ))),
            },
            v => Err(VMError::ConversionError(format!(
                "Cannot convert {v} to Number"
            ))),
        }
    }

    fn as_number(&mut self) -> Result<&mut Number, VMError> {
        if let PrimitiveValue::Number(m) = self {
            return Ok(m);
        }

        *self = PrimitiveValue::Number(self.to_number()?);
        self.as_number()
    }

    fn to_bool(&self) -> bool {
        match self {
            PrimitiveValue::None => false,
            PrimitiveValue::Error(_) => false,
            PrimitiveValue::Type(_) => false,
            PrimitiveValue::Bool(b) => *b,
            PrimitiveValue::Number(n) => !n.is_zero(),
            PrimitiveValue::String(s) => {
                let empty = s.is_empty();
                if empty {
                    return false;
                }

                s.parse().unwrap_or(true)
            }
            PrimitiveValue::Range(r) => !r.is_empty(),
        }
    }

    fn as_bool(&mut self) -> Result<&mut bool, VMError> {
        if let PrimitiveValue::Bool(m) = self {
            return Ok(m);
        }

        *self = PrimitiveValue::Bool(self.to_bool());
        self.as_bool()
    }

    fn as_string(&mut self) -> Result<&mut String, VMError> {
        if let PrimitiveValue::String(m) = self {
            return Ok(m);
        }

        *self = PrimitiveValue::String(self.to_string());
        self.as_string()
    }

    #[inline]
    fn to_float(&self) -> Result<f64, VMError> {
        Ok(self.to_number()?.to_float())
    }

    #[inline]
    fn to_usize(&self) -> Result<usize, VMError> {
        self.to_number()?.to_usize()
    }

    #[inline]
    fn to_int(&self) -> Result<i64, VMError> {
        Ok(self.to_number()?.to_int())
    }

    fn as_float(&mut self) -> Result<&mut f64, VMError> {
        if let PrimitiveValue::Number(m) = self {
            return match m {
                Number::Int(_) => {
                    *m = Number::Float(m.to_float());
                    let Number::Float(f) = m else { unreachable!() };
                    Ok(f)
                }
                Number::Float(f) => Ok(f),
            };
        }

        *self = PrimitiveValue::Number(Number::Float(self.to_float()?));
        self.as_float()
    }

    fn as_int(&mut self) -> Result<&mut i64, VMError> {
        if let PrimitiveValue::Number(m) = self {
            return match m {
                Number::Int(i) => Ok(i),
                Number::Float(_) => {
                    *m = Number::Int(m.to_int());
                    let Number::Int(i) = m else { unreachable!() };
                    Ok(i)
                }
            };
        }

        *self = PrimitiveValue::Number(Number::Int(self.to_int()?));
        self.as_int()
    }
}

impl Eq for PrimitiveValue {}

impl PartialOrd for PrimitiveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrimitiveValue {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }

        match (self, other) {
            (PrimitiveValue::Error(_), _) => Ordering::Less,
            (_, PrimitiveValue::Error(_)) => Ordering::Greater,
            (PrimitiveValue::Type(a), PrimitiveValue::Type(b)) => a.cmp(b),
            (PrimitiveValue::Type(_), _) => Ordering::Less,
            (_, PrimitiveValue::Type(_)) => Ordering::Greater,
            (PrimitiveValue::None, _) => Ordering::Less,
            (_, PrimitiveValue::None) => Ordering::Greater,
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => a.cmp(b),
            (PrimitiveValue::Bool(_), _) => Ordering::Less,
            (_, PrimitiveValue::Bool(_)) => Ordering::Greater,
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => a.cmp(b),
            (PrimitiveValue::Number(_), _) => Ordering::Less,
            (_, PrimitiveValue::Number(_)) => Ordering::Greater,
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => a.cmp(b),
            (PrimitiveValue::Range(_), _) => Ordering::Less,
            (_, PrimitiveValue::Range(_)) => Ordering::Greater,
            (PrimitiveValue::String(a), PrimitiveValue::String(b)) => a.cmp(b),
        }
    }
}

impl PrimitiveValue {
    #[inline]
    pub fn map<F, T>(&self, map: F) -> Option<T>
    where
        F: FnOnce(&Self) -> T,
    {
        if let PrimitiveValue::None = self {
            None
        } else {
            Some(map(self))
        }
    }

    #[inline]
    pub fn map_mut<F, T>(&mut self, map: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> T,
    {
        if let PrimitiveValue::None = self {
            None
        } else {
            Some(map(self))
        }
    }
}

impl Display for PrimitiveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveValue::None => write!(f, "none"),
            // todo dedicated to_string instead of debug
            PrimitiveValue::Error(e) => write!(f, "{}", e),
            PrimitiveValue::Type(e) => write!(f, "{}", e),
            PrimitiveValue::Bool(v) => write!(f, "{}", v),
            PrimitiveValue::Number(v) => write!(f, "{}", v),
            PrimitiveValue::String(v) => write!(f, "{}", v),
            PrimitiveValue::Range(v) => write!(f, "{}", v),
        }
    }
}

impl Hash for PrimitiveValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimitiveValue::None => 0.hash(state),
            PrimitiveValue::Error(e) => e.hash(state),
            PrimitiveValue::Type(e) => e.hash(state),
            PrimitiveValue::Bool(b) => b.hash(state),
            PrimitiveValue::Number(n) => n.hash(state),
            PrimitiveValue::String(s) => s.hash(state),
            PrimitiveValue::Range(s) => s.hash(state),
        }
    }
}

impl PartialEq for PrimitiveValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PrimitiveValue::None, PrimitiveValue::None) => true,
            (PrimitiveValue::Error(a), PrimitiveValue::Error(b)) => *a == *b,
            (PrimitiveValue::Type(a), PrimitiveValue::Type(b)) => *a == *b,
            (PrimitiveValue::None, PrimitiveValue::Bool(false)) => true,
            (PrimitiveValue::None, PrimitiveValue::Number(n)) => n.is_zero(),
            (PrimitiveValue::Bool(false), PrimitiveValue::Number(n)) => n.is_zero(),
            (PrimitiveValue::None, PrimitiveValue::String(s)) => s.is_empty() || s.eq("none"),
            (PrimitiveValue::Bool(false), PrimitiveValue::String(s)) => {
                s.is_empty() || s.eq("false")
            }
            (PrimitiveValue::Bool(true), PrimitiveValue::String(s)) => s.eq("true"),
            (PrimitiveValue::Bool(true), PrimitiveValue::Number(n)) => n.is_one(),
            (PrimitiveValue::Bool(false), PrimitiveValue::None) => true,
            (&PrimitiveValue::Bool(a), &PrimitiveValue::Bool(b)) => a == b,
            (PrimitiveValue::Number(n), PrimitiveValue::None) => n.is_zero(),
            (PrimitiveValue::Number(n), PrimitiveValue::Bool(false)) => n.is_zero(),
            (PrimitiveValue::String(s), PrimitiveValue::None) => s.is_empty() || s.eq("none"),
            (PrimitiveValue::String(s), PrimitiveValue::Bool(false)) => {
                s.is_empty() || s.eq("false")
            }
            (PrimitiveValue::String(s), PrimitiveValue::Bool(true)) => s.eq("true"),
            (PrimitiveValue::Number(n), PrimitiveValue::Bool(true)) => n.is_one(),
            (&PrimitiveValue::Number(a), &PrimitiveValue::Number(b)) => a == b,
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => a == b,
            (PrimitiveValue::String(a), PrimitiveValue::String(b)) => *a == *b,
            (PrimitiveValue::Number(n), PrimitiveValue::String(s)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (PrimitiveValue::String(s), PrimitiveValue::Number(n)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (PrimitiveValue::String(s), v) => s.eq(v.to_string().as_str()),
            (v, PrimitiveValue::String(s)) => s.eq(v.to_string().as_str()),
            (_, _) => false,
        }
    }
}

#[cfg(test)]
pub mod value_tests {
    use crate::{Number, PrimitiveValue};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn value_eq() {
        assert_eq!(PrimitiveValue::None, PrimitiveValue::None);
        assert_eq!(PrimitiveValue::None, PrimitiveValue::Bool(false));
        assert_eq!(PrimitiveValue::None, PrimitiveValue::Number(Number::Int(0)));
        assert_eq!(
            PrimitiveValue::None,
            PrimitiveValue::Number(Number::Float(0.0))
        );
        assert_eq!(PrimitiveValue::None, PrimitiveValue::String(String::new()));
        assert_eq!(
            PrimitiveValue::Bool(false),
            PrimitiveValue::String(String::new())
        );
        assert_eq!(
            PrimitiveValue::Number(Number::Int(0)),
            PrimitiveValue::String(String::new())
        );
    }
}
