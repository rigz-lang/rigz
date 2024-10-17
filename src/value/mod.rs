mod add;
mod bitand;
mod bitor;
mod bitxor;
mod div;
mod logical;
mod mul;
mod neg;
mod not;
mod rem;
mod reverse;
mod shl;
mod shr;
mod sub;

use crate::number::Number;
use crate::value_range::ValueRange;
use crate::{impl_from, RigzType, VMError};
use indexmap::IndexMap;
use log::trace;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(String),
    List(Vec<Value>),
    Map(IndexMap<Value, Value>),
    Range(ValueRange),
    Error(VMError),
    // todo create dedicated object value to avoid map usage everywhere, might need to be a trait
}

impl_from! {
    bool, Value, Value::Bool;
    String, Value, Value::String;
    VMError, Value, Value::Error;
    Vec<Value>, Value, Value::List;
    ValueRange, Value, Value::Range;
    IndexMap<Value, Value>, Value, Value::Map;
}

impl<T: Into<Number>> From<T> for Value {
    #[inline]
    fn from(value: T) -> Self {
        Value::Number(value.into())
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            return Some(Ordering::Equal);
        }

        match (self, other) {
            (Value::Error(_), _) => Some(Ordering::Less),
            (_, Value::Error(_)) => Some(Ordering::Greater),
            (Value::None, _) => Some(Ordering::Less),
            (_, Value::None) => Some(Ordering::Greater),
            (Value::Bool(_), _) => Some(Ordering::Less),
            (_, Value::Bool(_)) => Some(Ordering::Greater),
            (Value::Number(_), _) => Some(Ordering::Less),
            (_, Value::Number(_)) => Some(Ordering::Greater),
            (Value::Range(_), _) => Some(Ordering::Less),
            (_, Value::Range(_)) => Some(Ordering::Greater),
            (Value::String(_), _) => Some(Ordering::Less),
            (_, Value::String(_)) => Some(Ordering::Greater),
            (Value::List(_), _) => Some(Ordering::Less),
            (_, Value::List(_)) => Some(Ordering::Greater),
            (_, Value::Map(_)) => Some(Ordering::Greater),
        }
    }
}

impl Value {
    #[inline]
    pub fn to_number(&self) -> Option<Number> {
        match self {
            Value::None => Some(Number::zero()),
            Value::Bool(b) => {
                let n = if *b { Number::one() } else { Number::zero() };
                Some(n)
            }
            Value::Number(n) => Some(*n),
            Value::String(s) => match s.parse() {
                Ok(n) => Some(n),
                Err(e) => {
                    trace!("Failed to convert {} to number: {}", s, e);
                    None
                }
            },
            _ => None,
        }
    }

    pub fn to_float(&self) -> Option<f64> {
        self.to_number().map(|n| n.to_float())
    }

    pub fn to_int(&self) -> Option<i64> {
        self.to_number().map(|n| n.to_int())
    }

    #[inline]
    pub fn to_bool(&self) -> bool {
        match self {
            Value::None => false,
            Value::Error(_) => false,
            Value::Bool(b) => *b,
            Value::Number(n) => !n.is_zero(),
            Value::String(s) => {
                let empty = s.is_empty();
                if empty {
                    return false;
                }

                s.parse().unwrap_or(true)
            }
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
            Value::List(l) => l.clone(),
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
            Value::Map(m) => m.clone(),
            Value::Error(e) => {
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
        }
    }

    #[inline]
    pub fn cast(self, rigz_type: RigzType) -> Value {
        match (self, rigz_type) {
            (_, RigzType::None) => Value::None,
            (v, RigzType::Bool) => Value::Bool(v.to_bool()),
            (v, RigzType::String) => Value::String(v.to_string()),
            (v, RigzType::Number) => match v.to_number() {
                None => {
                    VMError::ConversionError(format!("Cannot convert {} to Number", v)).to_value()
                }
                Some(n) => Value::Number(n),
            },
            (s, RigzType::Any) => s,
            (v, RigzType::Int) => match v.to_int() {
                None => VMError::ConversionError(format!("Cannot convert {} to Int", v)).to_value(),
                Some(n) => Value::Number(n.into()),
            },
            (v, RigzType::Float) => match v.to_float() {
                None => {
                    VMError::ConversionError(format!("Cannot convert {} to Float", v)).to_value()
                }
                Some(n) => Value::Number(n.into()),
            },
            (v, RigzType::List(_)) => Value::List(v.to_list()),
            (v, RigzType::Map(_, _)) => Value::Map(v.to_map()),
            (v, RigzType::Custom(def)) => {
                let mut res = v.to_map();
                for (field, rigz_type) in def.fields {
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
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            // todo dedicated to_string instead of debug
            Value::Error(e) => write!(f, "{:?}", e),
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
                write!(f, "[{}]", values)
            }
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::None => 0.hash(state),
            Value::Error(e) => e.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Number(n) => n.hash(state),
            Value::String(s) => s.hash(state),
            Value::Range(s) => s.hash(state),
            Value::List(l) => {
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
            (Value::Error(_), _) => false,
            (_, Value::Error(_)) => false,
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
            (Value::Bool(_), _) => false,
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
            (_, Value::Bool(_)) => false,
            (&Value::Number(a), &Value::Number(b)) => a == b,
            (Value::Range(a), Value::Range(b)) => a == b,
            (Value::Range(_), _) | (_, Value::Range(_)) => false,
            (Value::String(a), Value::String(b)) => *a == *b,
            (Value::List(a), Value::List(b)) => *a == *b,
            (Value::Map(a), Value::Map(b)) => *a == *b,
            (Value::Number(n), Value::String(s)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (Value::String(s), Value::Number(n)) => {
                (s.is_empty() && n.is_zero()) || n.to_string().eq(s)
            }
            (Value::Number(_), _) => false,
            (_, Value::Number(_)) => false,
            (Value::String(s), v) => s.eq(v.to_string().as_str()),
            (v, Value::String(s)) => s.eq(v.to_string().as_str()),
            (Value::List(a), Value::Map(b)) => a.is_empty() && b.is_empty(),
            (Value::Map(a), Value::List(b)) => a.is_empty() && b.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Number, Value};

    #[test]
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
