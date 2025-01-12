mod add;
mod bitand;
mod bitor;
mod bitxor;
mod div;
mod error;
mod logical;
mod mul;
mod neg;
mod not;
mod rem;
mod reverse;
mod shl;
mod shr;
mod sub;

pub use error::VMError;
use std::cell::RefCell;

use crate::{impl_from, Number, RigzType, Snapshot, ValueRange};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::vec::IntoIter;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(String),
    // todo Lists, Maps, & Tuples should use Rc<RefCell<Value>> to make this language fully pass by reference
    List(Vec<Value>),
    Map(IndexMap<Value, Value>),
    Range(ValueRange),
    Error(VMError),
    Tuple(Vec<Value>),
    // todo create dedicated object value to avoid map usage everywhere, might need to be a trait. Create to_o method for value
    Type(RigzType),
}

impl Snapshot for Value {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Value::None => vec![0],
            Value::Bool(b) => vec![1, *b as u8],
            Value::Number(n) => {
                let mut res = match n {
                    Number::Int(_) => vec![2],
                    Number::Float(_) => vec![3],
                };
                res.extend(n.to_bytes());
                res
            }
            Value::String(s) => {
                let mut res = vec![4];
                res.extend(s.as_bytes());
                res
            }
            Value::List(v) => {
                let mut res = vec![5];
                res.extend(v.as_bytes());
                res
            }
            Value::Map(m) => {
                let mut res = vec![6];
                res.extend(m.as_bytes());
                res
            }
            Value::Range(r) => {
                let mut res = vec![7];
                res.extend(r.as_bytes());
                res
            }
            Value::Error(e) => {
                let mut res = vec![8];
                res.extend(e.as_bytes());
                res
            }
            Value::Tuple(v) => {
                let mut res = vec![9];
                res.extend(v.as_bytes());
                res
            }
            Value::Type(t) => {
                let mut res = vec![10];
                res.extend(t.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

impl From<Value> for Rc<RefCell<Value>> {
    #[inline]
    fn from(value: Value) -> Self {
        Rc::new(RefCell::new(value))
    }
}

impl_from! {
    bool, Value, Value::Bool;
    String, Value, Value::String;
    VMError, Value, Value::Error;
    ValueRange, Value, Value::Range;
    RigzType, Value, Value::Type;
}

impl From<&'_ str> for Value {
    #[inline]
    fn from(value: &'_ str) -> Self {
        Value::String(value.to_string())
    }
}

impl<T: Into<Number>> From<T> for Value {
    #[inline]
    fn from(value: T) -> Self {
        Value::Number(value.into())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Value::List(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<K: Into<Value>, V: Into<Value>> From<IndexMap<K, V>> for Value {
    #[inline]
    fn from(value: IndexMap<K, V>) -> Self {
        Value::Map(
            value
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<()> for Value {
    #[inline]
    fn from(_value: ()) -> Self {
        Value::None
    }
}

impl<A: Into<Value>, B: Into<Value>> From<(A, B)> for Value {
    #[inline]
    fn from(value: (A, B)) -> Self {
        Value::Tuple(vec![value.0.into(), value.1.into()])
    }
}

impl<A: Into<Value>, B: Into<Value>, C: Into<Value>> From<(A, B, C)> for Value {
    #[inline]
    fn from(value: (A, B, C)) -> Self {
        Value::Tuple(vec![value.0.into(), value.1.into(), value.2.into()])
    }
}

impl<A: Into<Value>, B: Into<Value>, C: Into<Value>, D: Into<Value>> From<(A, B, C, D)> for Value {
    #[inline]
    fn from(value: (A, B, C, D)) -> Self {
        Value::Tuple(vec![
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        ])
    }
}

impl<A: Into<Value>, B: Into<Value>, C: Into<Value>, D: Into<Value>, E: Into<Value>>
    From<(A, B, C, D, E)> for Value
{
    #[inline]
    fn from(value: (A, B, C, D, E)) -> Self {
        Value::Tuple(vec![
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
            value.4.into(),
        ])
    }
}

impl<V: Into<Value>> From<Option<V>> for Value {
    #[inline]
    fn from(value: Option<V>) -> Self {
        match value {
            None => Value::None,
            Some(v) => v.into(),
        }
    }
}

impl<V: Into<Value>> From<Result<V, VMError>> for Value {
    #[inline]
    fn from(value: Result<V, VMError>) -> Self {
        match value {
            Ok(v) => v.into(),
            Err(e) => e.into(),
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            return Some(Ordering::Equal);
        }

        match (self, other) {
            // todo some of these should probably use None?
            (Value::Error(_), _) => Some(Ordering::Less),
            (_, Value::Error(_)) => Some(Ordering::Greater),
            (Value::Type(a), Value::Type(b)) => a.partial_cmp(b),
            (Value::Type(_), _) => Some(Ordering::Less),
            (_, Value::Type(_)) => Some(Ordering::Greater),
            (Value::None, _) => Some(Ordering::Less),
            (_, Value::None) => Some(Ordering::Greater),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            (Value::Bool(_), _) => Some(Ordering::Less),
            (_, Value::Bool(_)) => Some(Ordering::Greater),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Number(_), _) => Some(Ordering::Less),
            (_, Value::Number(_)) => Some(Ordering::Greater),
            (Value::Range(a), Value::Range(b)) => a.partial_cmp(b),
            (Value::Range(_), _) => Some(Ordering::Less),
            (_, Value::Range(_)) => Some(Ordering::Greater),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::String(_), _) => Some(Ordering::Less),
            (_, Value::String(_)) => Some(Ordering::Greater),
            (Value::Tuple(a), Value::Tuple(b)) => a.partial_cmp(b),
            (Value::Tuple(_), _) => Some(Ordering::Less),
            (_, Value::Tuple(_)) => Some(Ordering::Greater),
            (Value::List(a), Value::List(b)) => a.partial_cmp(b),
            (Value::List(_), _) => Some(Ordering::Less),
            (_, Value::List(_)) => Some(Ordering::Greater),
            (Value::Map(a), Value::Map(b)) => a.into_iter().partial_cmp(b),
        }
    }
}

impl Value {
    #[inline]
    pub fn to_number(&self) -> Result<Number, VMError> {
        match self {
            Value::None => Ok(Number::zero()),
            Value::Bool(b) => {
                let n = if *b { Number::one() } else { Number::zero() };
                Ok(n)
            }
            Value::Number(n) => Ok(*n),
            Value::String(s) => match s.parse() {
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

    #[inline]
    pub fn to_float(&self) -> Result<f64, VMError> {
        Ok(self.to_number()?.to_float())
    }

    #[inline]
    pub fn to_int(&self) -> Result<i64, VMError> {
        Ok(self.to_number()?.to_int())
    }

    #[inline]
    pub fn to_usize(&self) -> Result<usize, VMError> {
        self.to_number()?.to_usize()
    }

    pub fn as_bool(&mut self) -> &mut bool {
        if let Value::Bool(m) = self {
            return m;
        }

        *self = Value::Bool(self.to_bool());
        self.as_bool()
    }

    pub fn as_float(&mut self) -> Result<&mut f64, VMError> {
        if let Value::Number(m) = self {
            return match m {
                Number::Int(_) => {
                    *m = Number::Float(m.to_float());
                    let Number::Float(f) = m else { unreachable!() };
                    Ok(f)
                }
                Number::Float(f) => Ok(f),
            };
        }

        *self = Value::Number(Number::Float(self.to_float()?));
        self.as_float()
    }

    pub fn as_number(&mut self) -> Result<&mut Number, VMError> {
        if let Value::Number(m) = self {
            return Ok(m);
        }

        *self = Value::Number(self.to_number()?);
        self.as_number()
    }

    pub fn as_int(&mut self) -> Result<&mut i64, VMError> {
        if let Value::Number(m) = self {
            return match m {
                Number::Int(i) => Ok(i),
                Number::Float(_) => {
                    *m = Number::Int(m.to_int());
                    let Number::Int(i) = m else { unreachable!() };
                    Ok(i)
                }
            };
        }

        *self = Value::Number(Number::Int(self.to_int()?));
        self.as_int()
    }

    pub fn as_string(&mut self) -> &mut String {
        if let Value::String(m) = self {
            return m;
        }

        *self = Value::String(self.to_string());
        self.as_string()
    }

    pub fn as_map(&mut self) -> &mut IndexMap<Value, Value> {
        if let Value::Map(m) = self {
            return m;
        }

        *self = Value::Map(self.to_map());
        self.as_map()
    }

    pub fn as_list(&mut self) -> &mut Vec<Value> {
        if let Value::List(m) = self {
            return m;
        }

        *self = Value::List(self.to_list());
        self.as_list()
    }

    #[inline]
    pub fn to_bool(&self) -> bool {
        match self {
            Value::None => false,
            Value::Error(_) => false,
            Value::Type(_) => false,
            Value::Bool(b) => *b,
            Value::Number(n) => !n.is_zero(),
            Value::String(s) => {
                let empty = s.is_empty();
                if empty {
                    return false;
                }

                s.parse().unwrap_or(true)
            }
            Value::Tuple(l) => !l.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Range(r) => !r.is_empty(),
        }
    }

    pub fn to_list(&self) -> Vec<Value> {
        match self {
            Value::None => vec![],
            Value::Bool(b) => {
                if *b {
                    vec![Value::Bool(*b)]
                } else {
                    vec![]
                }
            }
            Value::Number(n) => {
                vec![(*n).into()]
            }
            Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
            Value::List(l) | Value::Tuple(l) => l.clone(),
            Value::Map(m) => {
                let mut result = Vec::with_capacity(m.len());
                for (k, v) in m {
                    match k {
                        Value::Number(i) => match i.to_usize() {
                            Ok(index) => {
                                result.insert(index, v.clone());
                            }
                            Err(_) => return vec![Value::Map(m.clone())],
                        },
                        _ => return vec![Value::Map(m.clone())],
                    }
                }
                result
            }
            Value::Range(r) => r.to_list(),
            Value::Error(e) => vec![Value::Error(e.clone())],
            Value::Type(e) => vec![Value::Type(e.clone())],
        }
    }

    pub fn to_map(&self) -> IndexMap<Value, Value> {
        match self {
            Value::None => IndexMap::new(),
            Value::Bool(b) => {
                if *b {
                    IndexMap::from([(Value::Bool(*b), Value::Bool(*b))])
                } else {
                    IndexMap::new()
                }
            }
            Value::Number(n) => IndexMap::from([((*n).into(), (*n).into())]),
            Value::String(s) => {
                let s = s.clone();
                IndexMap::from([(s.clone().into(), s.into())])
            }
            Value::List(l) => l.iter().map(|v| (v.clone(), v.clone())).collect(),
            Value::Tuple(l) => l
                .chunks(2)
                .map(|v| {
                    let [k, v] = v else {
                        return (v[0].clone(), Value::None);
                    };
                    (k.clone(), v.clone())
                })
                .collect(),
            Value::Map(m) => m.clone(),
            Value::Error(e) => {
                let e = e.clone();
                IndexMap::from([(e.clone().into(), e.into())])
            }
            Value::Type(e) => {
                let e = e.clone();
                IndexMap::from([(e.clone().into(), e.into())])
            }
            Value::Range(r) => r.to_map(),
        }
    }

    #[inline]
    pub fn rigz_type(&self) -> RigzType {
        match self {
            Value::None => RigzType::None,
            Value::Bool(_) => RigzType::Bool,
            Value::Number(_) => RigzType::Number,
            Value::String(_) => RigzType::String,
            // todo add type info to lists & maps
            Value::List(_) => RigzType::List(Box::new(RigzType::Any)),
            Value::Map(_) => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            Value::Range(_) => RigzType::Range,
            Value::Error(_) => RigzType::Error,
            Value::Tuple(v) => RigzType::Tuple(v.iter().map(|v| v.rigz_type()).collect()),
            Value::Type(r) => r.clone(),
        }
    }

    #[inline]
    pub fn cast(&self, rigz_type: &RigzType) -> Value {
        match (self, rigz_type) {
            (s, RigzType::Error) => Value::Error(VMError::RuntimeError(s.to_string())),
            (_, RigzType::None) => Value::None,
            (v, RigzType::Bool) => Value::Bool(v.to_bool()),
            (v, RigzType::String) => Value::String(v.to_string()),
            (v, RigzType::Number) => match v.to_number() {
                Err(e) => e.into(),
                Ok(n) => n.into(),
            },
            (s, RigzType::Any) => s.clone(),
            (v, RigzType::Int) => match v.to_int() {
                Err(_) => {
                    VMError::ConversionError(format!("Cannot convert {} to Int", v)).to_value()
                }
                Ok(n) => n.into(),
            },
            (v, RigzType::Float) => match v.to_float() {
                Err(_) => {
                    VMError::ConversionError(format!("Cannot convert {} to Float", v)).to_value()
                }
                Ok(n) => n.into(),
            },
            (v, RigzType::List(_)) => Value::List(v.to_list()),
            (v, RigzType::Map(_, _)) => Value::Map(v.to_map()),
            (v, RigzType::Custom(def)) => {
                let mut res = v.to_map();
                for (field, rigz_type) in &def.fields {
                    match res.get_mut(&Value::String(field.clone())) {
                        None => {
                            return VMError::ConversionError(format!(
                                "Cannot convert value {} to {}, missing {}",
                                v, def.name, field
                            ))
                            .to_value()
                        }
                        Some(current) => *current = current.clone().cast(rigz_type),
                    }
                }
                Value::Map(res)
            }
            (v, t) => VMError::ConversionError(format!("Cannot convert value {} to {:?}", v, t))
                .to_value(),
        }
    }

    pub fn get(&self, attr: &Value) -> Result<Option<Value>, VMError> {
        let v = match (self, attr) {
            // todo support ranges as attr
            (Value::String(source), Value::Number(n)) => match n.to_usize() {
                Ok(index) => match source.chars().nth(index) {
                    None => return Ok(None),
                    Some(c) => Value::String(c.to_string()),
                },
                Err(e) => e.into(),
            },
            (Value::List(source), Value::Number(n)) | (Value::Tuple(source), Value::Number(n)) => {
                match n.to_usize() {
                    Ok(index) => match source.get(index) {
                        None => return Ok(None),
                        Some(c) => c.clone(),
                    },
                    Err(e) => e.into(),
                }
            }
            (Value::Map(source), index) => match source.get(index) {
                None => {
                    if let Value::Number(index) = index {
                        if let Ok(index) = index.to_usize() {
                            return Ok(source
                                .get_index(index)
                                .map(|(k, v)| Value::Tuple(vec![k.clone(), v.clone()])));
                        }
                    }
                    return Ok(None);
                }
                Some(c) => c.clone(),
            },
            (Value::Number(source), Value::Number(n)) => {
                Value::Bool(source.to_bits() & (1 << n.to_int()) != 0)
            }
            (source, attr) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot read {} for {}",
                    attr, source
                )))
            }
        };
        Ok(Some(v))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            // todo dedicated to_string instead of debug
            Value::Error(e) => write!(f, "{}", e),
            Value::Type(e) => write!(f, "{}", e),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Range(v) => write!(f, "{}", v),
            Value::List(l) => {
                let mut values = String::new();
                let len = l.len();
                for (index, v) in l.iter().enumerate() {
                    values.push_str(v.to_string().as_str());
                    if index != len - 1 {
                        values.push(',')
                    }
                }
                write!(f, "[{}]", values)
            }
            Value::Tuple(l) => {
                let mut values = String::new();
                let len = l.len();
                for (index, v) in l.iter().enumerate() {
                    values.push_str(v.to_string().as_str());
                    if index != len - 1 {
                        values.push(',')
                    }
                }
                write!(f, "({})", values)
            }
            Value::Map(m) => {
                let mut values = String::new();
                let len = m.len();
                for (index, (k, v)) in m.iter().enumerate() {
                    values.push_str(k.to_string().as_str());
                    values.push_str(" = ");
                    values.push_str(v.to_string().as_str());
                    if index != len - 1 {
                        values.push(',')
                    }
                }
                write!(f, "{{{}}}", values)
            }
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::None => 0.hash(state),
            Value::Error(e) => e.hash(state),
            Value::Type(e) => e.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Number(n) => n.hash(state),
            Value::String(s) => s.hash(state),
            Value::Range(s) => s.hash(state),
            Value::List(l) | Value::Tuple(l) => {
                for v in l {
                    v.hash(state);
                }
            }
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Error(a), Value::Error(b)) => *a == *b,
            (Value::Type(a), Value::Type(b)) => *a == *b,
            (Value::None, Value::Bool(false)) => true,
            (Value::None, Value::Number(n)) => n.is_zero(),
            (Value::Bool(false), Value::Number(n)) => n.is_zero(),
            (Value::None, Value::String(s)) => s.is_empty() || s.eq("none"),
            (Value::None, Value::List(v)) => v.is_empty(),
            (Value::None, Value::Map(m)) => m.is_empty(),
            (Value::Bool(false), Value::String(s)) => s.is_empty() || s.eq("false"),
            (Value::Bool(false), Value::List(v)) => v.is_empty(),
            (Value::Bool(false), Value::Map(m)) => m.is_empty(),
            (Value::Bool(true), Value::String(s)) => s.eq("true"),
            (Value::Bool(true), Value::Number(n)) => n.is_one(),
            (Value::Bool(false), Value::None) => true,
            (&Value::Bool(a), &Value::Bool(b)) => a == b,
            (Value::Number(n), Value::None) => n.is_zero(),
            (Value::Number(n), Value::Bool(false)) => n.is_zero(),
            (Value::String(s), Value::None) => s.is_empty() || s.eq("none"),
            (Value::List(v), Value::None) => v.is_empty(),
            (Value::Map(m), Value::None) => m.is_empty(),
            (Value::String(s), Value::Bool(false)) => s.is_empty() || s.eq("false"),
            (Value::List(v), Value::Bool(false)) => v.is_empty(),
            (Value::Map(m), Value::Bool(false)) => m.is_empty(),
            (Value::String(s), Value::Bool(true)) => s.eq("true"),
            (Value::Number(n), Value::Bool(true)) => n.is_one(),
            (&Value::Number(a), &Value::Number(b)) => a == b,
            (Value::Range(a), Value::Range(b)) => a == b,
            (Value::String(a), Value::String(b)) => *a == *b,
            (Value::Tuple(a), Value::List(b)) | (Value::List(a), Value::Tuple(b)) => *a == *b,
            (Value::List(a), Value::List(b)) | (Value::Tuple(a), Value::Tuple(b)) => *a == *b,
            (Value::Map(a), Value::Map(b)) => *a == *b,
            (Value::Number(n), Value::String(s)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (Value::String(s), Value::Number(n)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (Value::String(s), v) => s.eq(v.to_string().as_str()),
            (v, Value::String(s)) => s.eq(v.to_string().as_str()),
            (Value::List(a), Value::Map(b)) => a.is_empty() && b.is_empty(),
            (Value::Map(a), Value::List(b)) => a.is_empty() && b.is_empty(),
            (_, _) => false,
        }
    }
}

#[cfg(test)]
pub mod value_tests {
    use crate::{Number, Value};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn value_eq() {
        assert_eq!(Value::None, Value::None);
        assert_eq!(Value::None, Value::Bool(false));
        assert_eq!(Value::None, Value::Number(Number::Int(0)));
        assert_eq!(Value::None, Value::Number(Number::Float(0.0)));
        assert_eq!(Value::None, Value::String(String::new()));
        assert_eq!(Value::Bool(false), Value::String(String::new()));
        assert_eq!(Value::Number(Number::Int(0)), Value::String(String::new()));
    }
}
