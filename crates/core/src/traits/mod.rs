mod dyn_traits;
mod snapshot;

use crate::{Number, ObjectValue, RigzArgs, RigzType, VMError};
use dyn_clone::DynClone;
use indexmap::IndexMap;
pub use snapshot::Snapshot;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::vec::IntoIter;

pub use dyn_traits::*;

pub trait Definition {
    fn name(&self) -> &'static str;

    fn trait_definition(&self) -> &'static str;
}

#[allow(unused_variables)]
pub trait Module: Debug + Definition {
    fn call(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call`",
            self.name()
        )))
    }

    fn call_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_extension`",
            self.name()
        )))
    }

    fn call_mutable_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_extension`",
            self.name()
        )))
    }
}

#[allow(unused_variables)]
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait Object:
    DynCompare + DynClone + DynHash + AsPrimitive<ObjectValue> + Definition + Send + Sync
{
    fn create() -> Self
    where
        Self: Sized;

    fn call(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call`",
            self.name()
        )))
    }

    fn call_extension(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_extension`",
            self.name()
        )))
    }

    fn call_mutable_extension(
        &mut self,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_mutable_extension`",
            self.name()
        )))
    }
}

dyn_clone::clone_trait_object!(Object);

impl Hash for dyn Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state)
    }
}

impl PartialEq<dyn Object> for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.as_dyn_compare() == other.as_dyn_compare()
    }
}

impl PartialEq<&Self> for Box<dyn Object> {
    fn eq(&self, other: &&Self) -> bool {
        <Self as PartialEq>::eq(self, *other)
    }
}

impl PartialOrd<dyn Object> for dyn Object {
    fn partial_cmp(&self, other: &dyn Object) -> Option<Ordering> {
        self.as_dyn_compare().partial_cmp(other.as_dyn_compare())
    }
}

impl PartialOrd<&Self> for Box<dyn Object> {
    fn partial_cmp(&self, other: &&Self) -> Option<Ordering> {
        <Self as PartialOrd>::partial_cmp(self, *other)
    }
}

// first pass will use serde to read/write from bytes
impl Snapshot for Box<dyn Object + '_> {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

pub trait AsPrimitive<T: Clone + AsPrimitive<T> + Default + Sized>: Display + Debug {
    fn rigz_type(&self) -> RigzType;

    fn reverse(&self) -> Result<T, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot reverse {self}"
        )))
    }

    fn as_list(&mut self) -> Result<&mut Vec<T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut List"
        )))
    }

    fn to_list(&self) -> Result<Vec<T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to List"
        )))
    }

    fn to_map(&self) -> Result<IndexMap<T, T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to Map"
        )))
    }

    fn as_map(&mut self) -> Result<&mut IndexMap<T, T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Map"
        )))
    }

    fn to_number(&self) -> Result<Number, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to Number"
        )))
    }

    fn as_number(&mut self) -> Result<&mut Number, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Number"
        )))
    }

    fn to_bool(&self) -> bool {
        true
    }

    fn as_bool(&mut self) -> Result<&mut bool, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Bool"
        )))
    }

    fn as_string(&mut self) -> Result<&mut String, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut String"
        )))
    }

    fn to_float(&self) -> Result<f64, VMError> {
        Ok(self.to_number()?.to_float())
    }

    fn to_usize(&self) -> Result<usize, VMError> {
        self.to_number()?.to_usize()
    }

    fn to_int(&self) -> Result<i64, VMError> {
        Ok(self.to_number()?.to_int())
    }

    fn as_float(&mut self) -> Result<&mut f64, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Float"
        )))
    }

    fn as_int(&mut self) -> Result<&mut i64, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Int"
        )))
    }
}

pub trait Reverse {
    type Output;

    fn reverse(&self) -> Self::Output;
}

pub trait Logical<Rhs> {
    type Output;

    fn and(self, rhs: Rhs) -> Self::Output;
    fn or(self, rhs: Rhs) -> Self::Output;
    fn xor(self, rhs: Rhs) -> Self::Output;
}

impl<T: Clone + AsPrimitive<T> + Default + Sized> Logical<&T> for &T {
    type Output = T;

    #[inline]
    fn and(self, rhs: &T) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, _) => self,
            (true, _) => rhs,
        }
        .clone()
    }

    #[inline]
    fn or(self, rhs: &T) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, _) => rhs,
            (true, _) => self,
        }
        .clone()
    }

    #[inline]
    fn xor(self, rhs: &T) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, false) | (true, true) => T::default(),
            (false, _) => rhs.clone(),
            (true, _) => self.clone(),
        }
    }
}
