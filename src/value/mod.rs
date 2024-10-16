mod add;
mod bitand;
mod bitor;
mod bitxor;
mod div;
mod mul;
mod not;
mod neg;
mod rem;
mod rev;
mod shl;
mod shr;
mod sub;
mod logical;

use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use indexmap::IndexMap;
use crate::number::Number;
use crate::VMError;

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Number(Number),
    String(String),
    List(Vec<Value>),
    Map(IndexMap<Value, Value>),
    Error(VMError),
}

impl Value {
    pub fn to_bool(&self) -> bool {
        match self {
            Value::None => false,
            Value::Error(_) => false,
            Value::Bool(b) => *b,
            Value::Number(n) => !n.is_zero(),
            Value::String(s) => {
                let empty = s.is_empty();
                if empty {
                    return false
                }

                s.parse().unwrap_or(true)
            },
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty()
        }
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
        }
    }
}

impl Eq for Value {}

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
            (Value::Bool(a), Value::Bool(b)) => *a == *b,
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
            (Value::Number(a), Value::Number(b)) => *a == *b,
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
            (Value::Map(a), Value::List(b)) => a.is_empty() && b.is_empty()
        }
    }
}