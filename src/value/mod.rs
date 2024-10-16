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

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use indexmap::IndexMap;
use crate::number::Number;
use crate::{BOOL, ERROR, LIST, MAP, NONE, NUMBER, RigzObject, RigzObjectDefinition, RigzType, STRING, VMError, Scope};

#[derive(Clone, Debug)]
pub enum Value<'vm> {
    None,
    Bool(bool),
    Number(Number),
    String(String),
    List(Vec<Value<'vm>>),
    Map(IndexMap<Value<'vm>, Value<'vm>>),
    Object(RigzObject<'vm>),
    ScopeId(usize),
    Error(VMError),
    // TODO add scope here
}

impl <'vm> Eq for Value<'vm> {}

impl <'vm> PartialOrd for Value<'vm> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            return Some(Ordering::Equal)
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
            (_, Value::Object(_)) => Some(Ordering::Greater),
            (Value::ScopeId(_), _) => todo!(),
            (_, Value::ScopeId(_)) => todo!(),
        }
    }
}

impl <'vm> Value<'vm> {
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
            Value::Map(m) => !m.is_empty(),
            Value::Object(m) => !m.fields.is_empty(),
            Value::ScopeId(u) => todo!(),
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
            Value::Object(v) => RigzType::Object(v.definition_index.clone()),
            Value::Error(_) => RigzType::Error,
            Value::ScopeId(u) => todo!(),
        }
    }



    #[inline]
    pub fn to_object(&self) -> RigzObject<'vm> {
        if let Value::Object(o) = self {
            return o.clone();
        }
        let fields = IndexMap::from([("value".to_string(), self.clone())]);
        match &self {
            Value::None => RigzObject {
                fields,
                definition_index: &NONE,
            },
            Value::Bool(_) => RigzObject {
                fields,
                definition_index: &BOOL,
            },
            Value::Number(_) => RigzObject {
                fields,
                definition_index: &NUMBER,
            },
            Value::String(_) => RigzObject {
                fields,
                definition_index: &STRING,
            },
            Value::List(_) => RigzObject {
                fields,
                definition_index: &LIST,
            },
            Value::Map(_) => RigzObject {
                fields,
                definition_index: &MAP,
            },
            Value::Error(_) => RigzObject {
                fields,
                definition_index: &ERROR,
            },
            _ => unreachable!()
        }
    }

    #[inline]
    pub fn cast_to_object(&self, rigz_object_definition: RigzObjectDefinition) -> Result<RigzObject<'vm>, VMError> {
        let object = self.to_object();
        object.cast(rigz_object_definition)
    }

    #[inline]
    pub fn cast(&self, rigz_type: RigzType) -> Result<Value<'vm>, VMError> {
        let rigz_type = match rigz_type {
            RigzType::None => return Ok(Value::None),
            RigzType::Bool => return Ok(Value::Bool(self.to_bool())),
            RigzType::String => return Ok(Value::String(self.to_string())),
            RigzType::Object(o) => return Ok(Value::Object(self.cast_to_object(o)?)),
            _ => rigz_type
        };

        let self_type = self.rigz_type();
        if self_type == rigz_type {
            return Ok(self.to_owned())
        }

        let v = match (self_type, rigz_type) {
            (RigzType::None, RigzType::Number) => Value::Number(Number::Int(0)),
            (RigzType::None, RigzType::Int) => Value::Number(Number::Int(0)),
            (RigzType::None, RigzType::UInt) => Value::Number(Number::UInt(0)),
            (RigzType::None, RigzType::Float) => Value::Number(Number::Float(0.0)),
            (RigzType::None, RigzType::List) => Value::String(String::new()),
            (RigzType::None, RigzType::Map) => Value::String(String::new()),
            (RigzType::Bool, RigzType::Number) => {
                if let Value::Bool(b) = self {
                    let v = if *b { 1 } else { 0 };
                    return Ok(Value::Number(Number::Int(v)))
                }
                unreachable!()
            },
            (RigzType::Bool, RigzType::Int) => {
                if let Value::Bool(b) = self {
                    let v = if *b { 1 } else { 0 };
                    return Ok(Value::Number(Number::Int(v)))
                }
                unreachable!()
            },
            (RigzType::Bool, RigzType::UInt) => {
                if let Value::Bool(b) = self {
                    let v = if *b { 1 } else { 0 };
                    return Ok(Value::Number(Number::UInt(v)))
                }
                unreachable!()
            },
            (RigzType::Bool, RigzType::Float) => {
                if let Value::Bool(b) = self {
                    let v = if *b { 1.0 } else { 0.0 };
                    return Ok(Value::Number(Number::Float(v)))
                }
                unreachable!()
            },
            (RigzType::Number, RigzType::Int) => {
                if let Value::Number(b) = self {
                    return Ok(Value::Number(Number::Int(b.to_int())))
                }
                unreachable!()
            }
            (RigzType::Number, RigzType::UInt) => {
                if let Value::Number(b) = self {
                    return Ok(Value::Number(Number::UInt(b.to_uint()?)))
                }
                unreachable!()
            }
            (RigzType::Number, RigzType::Float) => {
                if let Value::Number(b) = self {
                    return Ok(Value::Number(Number::Float(b.to_float())))
                }
                unreachable!()
            }
            (RigzType::String, RigzType::List) => {
                if let Value::String(s) = self {
                    return Ok(Value::List(s.chars().map(|c| Value::String(c.to_string())).collect()))
                }
                unreachable!()
            }
            _ => unreachable!()
        };
        Ok(v)
    }
}

impl <'vm> Display for Value<'vm> {
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
            Value::Object(o) => {
                let mut values = String::new();
                let len = o.fields.len();
                for (index, (k, v)) in o.fields.iter().enumerate() {
                    values.push_str(k.to_string().as_str());
                    values.push_str(" = ");
                    values.push_str(v.to_string().as_str());
                    if index != len - 1 {
                        values.push(',')
                    }
                }
                write!(f, "{} {{ {} }}", o.definition_index.name, values)
            }
            Value::ScopeId(u) => todo!(),
        }
    }
}

impl <'vm> Hash for Value<'vm> {
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
            Value::Object(m) => m.hash(state),
            Value::ScopeId(u) => todo!(),
        }
    }
}

impl <'vm> PartialEq for Value<'vm> {
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
            (Value::None, Value::Object(m)) => m.is_empty(),
            (Value::Object(m), Value::None) => m.is_empty(),
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
            (Value::List(a), Value::Object(b)) => a.is_empty() && b.is_empty(),
            (Value::Object(a), Value::List(b)) => a.is_empty() && b.is_empty(),
            (Value::Map(a), Value::List(b)) => a.is_empty() && b.is_empty(),
            (Value::Map(a), Value::Object(b)) => b.equivalent(a),
            (Value::Object(a), Value::Map(b)) => a.equivalent(b),
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::ScopeId(a), Value::ScopeId(b)) => a == b,
            (Value::ScopeId(a), b) => todo!(),
            (a, Value::ScopeId(b)) => todo!(),
        }
    }
}