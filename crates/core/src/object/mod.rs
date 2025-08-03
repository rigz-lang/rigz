pub mod from;
mod ops;

#[cfg(feature = "snapshot")]
mod snapshot;

use crate::{
    AsPrimitive, IndexMap, IndexSet, Number, Object, PrimitiveValue, RigzType, VMError,
    WithTypeInfo,
};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ObjectValue {
    Primitive(PrimitiveValue),
    // todo Enum, Lists, Maps, & Tuples should use Rc<RefCell<ObjectValue>> to make this language fully pass by reference
    List(Vec<ObjectValue>),
    Set(IndexSet<ObjectValue>),
    Map(IndexMap<ObjectValue, ObjectValue>),
    Tuple(Vec<ObjectValue>),
    Object(Box<dyn Object>),
    Enum(usize, usize, Option<Box<ObjectValue>>),
}

impl ObjectValue {
    pub fn new(obj: impl Object) -> Self {
        ObjectValue::Object(Box::new(obj))
    }
}

impl Default for ObjectValue {
    fn default() -> Self {
        ObjectValue::Primitive(PrimitiveValue::default())
    }
}

impl Hash for ObjectValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ObjectValue::Primitive(p) => p.hash(state),
            ObjectValue::List(l) => l.hash(state),
            ObjectValue::Set(l) => {
                for v in l {
                    v.hash(state)
                }
            }
            ObjectValue::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            ObjectValue::Tuple(t) => t.hash(state),
            ObjectValue::Object(o) => o.hash(state),
            ObjectValue::Enum(e, i, v) => {
                e.hash(state);
                i.hash(state);
                v.hash(state);
            }
        }
    }
}

impl PartialEq for ObjectValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectValue::Primitive(left), ObjectValue::Primitive(right)) => left == right,
            (
                ObjectValue::Primitive(PrimitiveValue::None)
                | ObjectValue::Primitive(PrimitiveValue::Bool(false)),
                ObjectValue::List(l) | ObjectValue::Tuple(l),
            ) => l.is_empty(),
            (
                ObjectValue::Primitive(PrimitiveValue::None)
                | ObjectValue::Primitive(PrimitiveValue::Bool(false)),
                ObjectValue::Map(l),
            ) => l.is_empty(),
            (
                ObjectValue::Primitive(PrimitiveValue::None)
                | ObjectValue::Primitive(PrimitiveValue::Bool(false)),
                ObjectValue::Set(l),
            ) => l.is_empty(),
            (ObjectValue::List(l) | ObjectValue::Tuple(l), ObjectValue::Map(m)) => {
                l.is_empty() && m.is_empty()
            }
            (
                ObjectValue::List(left) | ObjectValue::Tuple(left),
                ObjectValue::List(right) | ObjectValue::Tuple(right),
            ) => left == right,
            (ObjectValue::Map(left), ObjectValue::Map(right)) => left == right,
            (ObjectValue::Set(left), ObjectValue::Set(right)) => left == right,
            (ObjectValue::Object(left), ObjectValue::Object(right)) => left == right,
            (ObjectValue::Enum(l_e, l_i, l_v), ObjectValue::Enum(r_e, r_i, r_v)) => {
                l_e == r_e && l_i == r_i && l_v == r_v
            }
            _ => false,
        }
    }
}

impl PartialOrd for ObjectValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ObjectValue::Primitive(left), ObjectValue::Primitive(right)) => Some(left.cmp(right)),
            (ObjectValue::List(lhs), ObjectValue::List(rhs)) => lhs.partial_cmp(rhs),
            (ObjectValue::Set(lhs), ObjectValue::Set(rhs)) => lhs.into_iter().partial_cmp(rhs),
            (ObjectValue::Map(lhs), ObjectValue::Map(rhs)) => lhs.into_iter().partial_cmp(rhs),
            (ObjectValue::Tuple(lhs), ObjectValue::Tuple(rhs)) => lhs.partial_cmp(rhs),
            (ObjectValue::Object(lhs), ObjectValue::Object(rhs)) => lhs.dyn_partial_cmp(rhs),
            _ => None,
        }
    }
}

impl Ord for ObjectValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

impl Eq for ObjectValue {}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectValue::Primitive(p) => write!(f, "{}", p),
            ObjectValue::Object(o) => write!(f, "{}", o),
            ObjectValue::List(l) => {
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
            ObjectValue::Set(l) => {
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
            ObjectValue::Tuple(l) => {
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
            ObjectValue::Map(m) => {
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
            ObjectValue::Enum(_, _, _) => {
                // todo need to figure this out
                write!(f, "")
            }
        }
    }
}

impl ObjectValue {
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self, ObjectValue::Primitive(PrimitiveValue::Error(_)))
    }

    #[inline]
    pub fn map<F, T>(&self, map: F) -> Option<T>
    where
        F: FnOnce(&Self) -> T,
    {
        if let ObjectValue::Primitive(PrimitiveValue::None) = self {
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
        if let ObjectValue::Primitive(PrimitiveValue::None) = self {
            None
        } else {
            Some(map(self))
        }
    }

    #[inline]
    pub fn maybe_map<F, T>(&self, map: F) -> Result<Option<T>, VMError>
    where
        F: FnOnce(&Self) -> Result<T, VMError>,
    {
        if let ObjectValue::Primitive(PrimitiveValue::None) = self {
            Ok(None)
        } else {
            Ok(Some(map(self)?))
        }
    }

    #[inline]
    pub fn maybe_map_mut<F, T>(&mut self, map: F) -> Result<Option<T>, VMError>
    where
        F: FnOnce(&mut Self) -> Result<T, VMError>,
    {
        if let ObjectValue::Primitive(PrimitiveValue::None) = self {
            Ok(None)
        } else {
            Ok(Some(map(self)?))
        }
    }

    pub fn get(&self, attr: &ObjectValue) -> Result<Option<ObjectValue>, VMError> {
        // todo support negative numbers as index, -1 is last element
        let v = match (self, attr) {
            // todo support ranges as attr
            (
                ObjectValue::Primitive(PrimitiveValue::String(source)),
                ObjectValue::Primitive(PrimitiveValue::Number(n)),
            ) => {
                let index = n.to_usize()?;
                match source.chars().nth(index) {
                    None => return Ok(None),
                    Some(c) => c.to_string().into(),
                }
            }
            (ObjectValue::List(source), ObjectValue::Primitive(PrimitiveValue::Number(n)))
            | (ObjectValue::Tuple(source), ObjectValue::Primitive(PrimitiveValue::Number(n))) => {
                match n.to_usize() {
                    Ok(index) => match source.get(index) {
                        None => return Ok(None),
                        Some(c) => c.clone(),
                    },
                    Err(e) => e.into(),
                }
            }
            (ObjectValue::Map(source), index) => match source.get(index) {
                None => return Ok(None),
                Some(c) => c.clone(),
            },
            (ObjectValue::Set(source), index) => match source.get(index) {
                None => return Ok(None),
                Some(c) => c.clone(),
            },
            (
                ObjectValue::Primitive(PrimitiveValue::Number(source)),
                ObjectValue::Primitive(PrimitiveValue::Number(n)),
            ) => (source.to_bits() & (1 << n.to_int()) != 0).into(),
            (ObjectValue::Object(o), v) => o.get(v)?,
            (source, attr) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot read {} for {}",
                    attr, source
                )))
            }
        };
        Ok(Some(v))
    }

    pub fn instance_set(
        &mut self,
        attr: Rc<RefCell<ObjectValue>>,
        value: &ObjectValue,
    ) -> Result<(), VMError> {
        // todo support negative numbers as index, -1 is last element
        let e = match (self, attr.borrow().deref()) {
            // todo support ranges as attr
            (
                ObjectValue::Primitive(PrimitiveValue::String(s)),
                ObjectValue::Primitive(PrimitiveValue::Number(n)),
            ) => match n.to_usize() {
                Ok(index) => {
                    s.insert_str(index, value.to_string().as_str());
                    None
                }
                Err(e) => Some(e),
            },
            (ObjectValue::List(s), ObjectValue::Primitive(PrimitiveValue::Number(n)))
            | (ObjectValue::Tuple(s), ObjectValue::Primitive(PrimitiveValue::Number(n))) => {
                match n.to_usize() {
                    Ok(index) => {
                        s.insert(index, value.clone());
                        None
                    }
                    Err(e) => Some(e),
                }
            }
            (ObjectValue::Set(s), ObjectValue::Primitive(PrimitiveValue::Number(n))) => {
                match n.to_usize() {
                    Ok(index) => {
                        s.shift_insert(index, value.clone());
                        None
                    }
                    Err(e) => Some(e),
                }
            }
            (ObjectValue::Map(source), index) => {
                source.insert(index.clone(), value.clone());
                None
            }
            (
                ObjectValue::Primitive(PrimitiveValue::Number(source)),
                ObjectValue::Primitive(PrimitiveValue::Number(n)),
            ) => {
                let value = if value.to_bool() { 1 } else { 0 };
                *source = match source {
                    Number::Int(_) => {
                        i64::from_le_bytes((source.to_bits() & (value << n.to_int())).to_le_bytes())
                            .into()
                    }
                    Number::Float(_) => {
                        f64::from_bits(source.to_bits() & (value << n.to_int())).into()
                    }
                };
                None
            }
            (source, attr) => {
                if let ObjectValue::Object(o) = source {
                    let value = value.clone();
                    let v = o.set(attr, value);
                    if let Err(e) = v {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    Some(VMError::UnsupportedOperation(format!(
                        "Cannot read {} for {}",
                        attr, source
                    )))
                }
            }
        };

        if let Some(e) = e {
            Err(e)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn cast(&self, rigz_type: &RigzType) -> ObjectValue {
        match (self, rigz_type) {
            (s, RigzType::Error) => VMError::RuntimeError(Box::new(s.clone())).into(),
            (_, RigzType::None) => ObjectValue::default(),
            (v, RigzType::Bool) => v.to_bool().into(),
            (v, RigzType::String) => v.to_string().into(),
            (v, RigzType::Number) => match v.to_number() {
                Err(e) => e.into(),
                Ok(n) => n.into(),
            },
            (s, RigzType::Any) => s.clone(),
            (v, RigzType::Int) => match v.to_int() {
                Err(_) => VMError::ConversionError(format!("Cannot convert {} to Int", v)).into(),
                Ok(n) => n.into(),
            },
            (v, RigzType::Float) => match v.to_float() {
                Err(_) => VMError::ConversionError(format!("Cannot convert {} to Float", v)).into(),
                Ok(n) => n.into(),
            },
            (v, RigzType::List(_)) => {
                let v = match v.to_list() {
                    Ok(v) => v,
                    Err(e) => return e.into(),
                };
                ObjectValue::List(v)
            }
            (v, RigzType::Set(_)) => {
                let v = match v.to_set() {
                    Ok(v) => v,
                    Err(e) => return e.into(),
                };
                ObjectValue::Set(v)
            }
            // todo check each type of tuple, probably with a to_tuple method
            (v, RigzType::Tuple(_)) => {
                let v = match v.to_list() {
                    Ok(v) => v,
                    Err(e) => return e.into(),
                };
                ObjectValue::Tuple(v)
            }
            (v, RigzType::Map(_, _)) => {
                let v = match v.to_map() {
                    Ok(v) => v,
                    Err(e) => return e.into(),
                };
                ObjectValue::Map(v)
            }
            (v, RigzType::Custom(def)) => {
                let mut res = match v.to_map() {
                    Ok(m) => m,
                    Err(e) => return e.into(),
                };
                for (field, rigz_type) in &def.fields {
                    let field: ObjectValue = field.clone().into();
                    match res.get_mut(&field) {
                        None => {
                            return VMError::ConversionError(format!(
                                "Cannot convert value {} to {}, missing {}",
                                v, def.name, field
                            ))
                            .into()
                        }
                        Some(current) => *current = current.clone().cast(rigz_type),
                    }
                }
                ObjectValue::Map(res)
            }
            (v, t) => {
                VMError::ConversionError(format!("Cannot convert value {} to {:?}", v, t)).into()
            }
        }
    }
}

impl WithTypeInfo for ObjectValue {
    fn rigz_type(&self) -> RigzType {
        match self {
            ObjectValue::Primitive(p) => p.rigz_type(),
            // todo figure out concrete types
            ObjectValue::Set(_) => RigzType::Set(Box::default()),
            ObjectValue::List(_) => RigzType::List(Box::default()),
            ObjectValue::Map(_) => RigzType::Map(Box::default(), Box::default()),
            ObjectValue::Tuple(t) => RigzType::Tuple(t.iter().map(|i| i.rigz_type()).collect()),
            ObjectValue::Object(o) => o.rigz_type(),
            // todo these should be updated
            ObjectValue::Enum(i, _, v) => match v {
                None => RigzType::Enum(*i),
                Some(v) => v.rigz_type(),
            },
        }
    }
}

impl AsPrimitive<ObjectValue> for ObjectValue {
    fn as_list(&mut self) -> Result<&mut Vec<ObjectValue>, VMError> {
        *self = ObjectValue::List(AsPrimitive::to_list(self)?);
        let ObjectValue::List(m) = self else {
            unreachable!()
        };
        Ok(m)
    }

    fn as_set(&mut self) -> Result<&mut IndexSet<ObjectValue>, VMError> {
        *self = ObjectValue::Set(AsPrimitive::to_set(self)?);
        let ObjectValue::Set(m) = self else {
            unreachable!()
        };
        Ok(m)
    }

    fn to_list(&self) -> Result<Vec<ObjectValue>, VMError> {
        match self {
            ObjectValue::Tuple(v) | ObjectValue::List(v) => Ok(v.clone()),
            ObjectValue::Set(s) => Ok(s.iter().cloned().collect()),
            ObjectValue::Map(m) => Ok(m
                .iter()
                .map(|(k, v)| ObjectValue::Tuple(vec![k.clone(), v.clone()]))
                .collect()),
            ObjectValue::Primitive(p) => Ok(p.to_list()?.into_iter().map(|p| p.into()).collect()),
            ObjectValue::Object(o) => o.to_list(),
            _ => Err(VMError::UnsupportedOperation(format!(
                "Cannot convert {self} to List"
            ))),
        }
    }

    fn to_set(&self) -> Result<IndexSet<ObjectValue>, VMError> {
        match self {
            ObjectValue::Tuple(v) | ObjectValue::List(v) => Ok(v.iter().cloned().collect()),
            ObjectValue::Set(s) => Ok(s.clone()),
            ObjectValue::Map(m) => Ok(m
                .iter()
                .map(|(k, v)| ObjectValue::Tuple(vec![k.clone(), v.clone()]))
                .collect()),
            ObjectValue::Primitive(p) => Ok(p.to_list()?.into_iter().map(|p| p.into()).collect()),
            ObjectValue::Object(o) => o.to_set(),
            _ => Err(VMError::UnsupportedOperation(format!(
                "Cannot convert {self} to List"
            ))),
        }
    }

    fn to_map(&self) -> Result<indexmap::IndexMap<ObjectValue, ObjectValue>, VMError> {
        match self {
            ObjectValue::Primitive(m) => Ok(m
                .to_map()?
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect()),
            ObjectValue::Map(m) => Ok(m.clone()),
            ObjectValue::List(l) => Ok(l.iter().map(|v| (v.clone(), v.clone())).collect()),
            ObjectValue::Set(l) => Ok(l.iter().map(|v| (v.clone(), v.clone())).collect()),
            ObjectValue::Tuple(t) => Ok(t
                .chunks(2)
                .map(|c| match &c[..2] {
                    [k, v] => (k.clone(), v.clone()),
                    [v] => (v.clone(), v.clone()),
                    _ => unreachable!(),
                })
                .collect()),
            ObjectValue::Object(m) => m.to_map(),
            ObjectValue::Enum(e, i, value) => match value {
                None => Err(VMError::UnsupportedOperation(format!(
                    "Cannot convert enum {e} to {i}"
                ))),
                Some(v) => v.to_map(),
            },
        }
    }

    fn to_number(&self) -> Result<Number, VMError> {
        match self {
            ObjectValue::Primitive(p) => p.to_number(),
            ObjectValue::Object(m) => m.to_number(),
            _ => Err(VMError::runtime(format!("Cannot convert {self} to number"))),
        }
    }

    fn to_bool(&self) -> bool {
        match self {
            ObjectValue::Tuple(l) => !l.is_empty(),
            ObjectValue::List(l) => !l.is_empty(),
            ObjectValue::Set(l) => !l.is_empty(),
            ObjectValue::Map(m) => !m.is_empty(),
            ObjectValue::Primitive(p) => p.to_bool(),
            ObjectValue::Object(o) => o.to_bool(),
            ObjectValue::Enum(_, _, v) => match v {
                None => true, // todo should variant 0 be false?
                Some(e) => e.to_bool(),
            },
        }
    }
}
