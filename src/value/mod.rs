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
use crate::{impl_from, impl_from_into, impl_from_into_lifetime, impl_from_lifetime, Register, RigzType, VMError};
use indexmap::IndexMap;
use log::trace;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Default)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(String),
    List(Vec<Value>),
    Map(IndexMap<Value, Value>),
    ScopeId(usize, Register),
    Error(VMError),
}

impl_from! {
    bool, Value, Value::Bool;
    String, Value, Value::String;
    Vec<Value>, Value, Value::List;
    IndexMap<Value, Value>, Value, Value::Map;
}

impl_from_into! {
    i32, Value, Value::Number;
    i64, Value, Value::Number;
    u32, Value, Value::Number;
    u64, Value, Value::Number;
    f32, Value, Value::Number;
    f64, Value, Value::Number;
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
            (Value::String(_), _) => Some(Ordering::Less),
            (_, Value::String(_)) => Some(Ordering::Greater),
            (Value::List(_), _) => Some(Ordering::Less),
            (_, Value::List(_)) => Some(Ordering::Greater),
            (Value::Map(_), _) => Some(Ordering::Less),
            (_, Value::Map(_)) => Some(Ordering::Greater),
            // (_, Value::Object(_)) => Some(Ordering::Greater),
            (Value::ScopeId(_, _), _) => todo!(),
            (_, Value::ScopeId(_, _)) => todo!(),
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
            Value::ScopeId(_u, _) => todo!(),
        }
    }

    #[inline]
    pub fn rigz_type(&self) -> RigzType {
        match self {
            Value::None => RigzType::None,
            Value::Bool(_) => RigzType::Bool,
            Value::Number(_) => RigzType::Number,
            Value::String(_) => RigzType::String,
            Value::List(_) => RigzType::List,
            Value::Map(_) => RigzType::Map,
            Value::Error(_) => RigzType::Error,
            Value::ScopeId(_u, _) => todo!(),
        }
    }

    #[inline]
    pub fn cast(&self, rigz_type: RigzType) -> Result<Value, VMError> {
        let rigz_type = match rigz_type {
            RigzType::None => return Ok(Value::None),
            RigzType::Bool => return Ok(Value::Bool(self.to_bool())),
            RigzType::String => return Ok(Value::String(self.to_string())),
            _ => rigz_type,
        };

        let self_type = self.rigz_type();
        if self_type == rigz_type {
            return Ok(self.to_owned());
        }

        let v = match (self_type, rigz_type) {
            (RigzType::None, RigzType::Number) => Value::Number(Number(0.0)),
            (RigzType::None, RigzType::List) => Value::String(String::new()),
            (RigzType::None, RigzType::Map) => Value::String(String::new()),
            (RigzType::Bool, RigzType::Number) => {
                if let &Value::Bool(b) = self {
                    return Ok(Value::Number(b.into()));
                }
                unreachable!()
            }
            (RigzType::String, RigzType::List) => {
                if let Value::String(s) = self {
                    return Ok(Value::List(
                        s.chars().map(|c| Value::String(c.to_string())).collect(),
                    ));
                }
                unreachable!()
            }
            _ => unreachable!(),
        };
        Ok(v)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            Value::Error(e) => write!(f, "{:?}", *e),
            Value::Bool(v) => write!(f, "{}", *v),
            Value::Number(v) => write!(f, "{}", *v),
            Value::String(v) => write!(f, "{}", *v),
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
            Value::ScopeId(u, _) => write!(f, "0x{}", *u),
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
            Value::ScopeId(u, _) => u.hash(state),
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
            (Value::ScopeId(a, _), Value::ScopeId(b, _)) => a == b,
            (Value::ScopeId(_, _), _) => todo!(),
            (_, Value::ScopeId(_, _)) => todo!(),
        }
    }
}
