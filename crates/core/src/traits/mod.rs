mod as_primitive;
mod dyn_traits;
#[cfg(feature = "snapshot")]
mod snapshot;

use crate::{Number, ObjectValue, RigzArgs, RigzType, VMError};
use dyn_clone::DynClone;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::vec::IntoIter;

pub use as_primitive::{AsPrimitive, WithTypeInfo};
pub use dyn_traits::*;

#[cfg(feature = "snapshot")]
pub use snapshot::Snapshot;

pub trait Definition {
    fn name() -> &'static str
    where
        Self: Sized;

    fn trait_definition() -> &'static str
    where
        Self: Sized;
}

#[derive(PartialEq, Eq)]
pub struct Dependency {
    pub create: fn(ObjectValue) -> Result<Box<dyn Object>, VMError>,
}

impl Debug for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Dependency {
    pub fn new<T: Object + 'static>() -> Self {
        Self {
            create: |value| Ok(Box::new(T::create(value)?)),
        }
    }
}

#[allow(unused_variables)]
pub trait Module: Debug + Definition {
    fn deps() -> Vec<Dependency>
    where
        Self: Sized,
    {
        vec![]
    }

    fn call(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call`"
        )))
    }

    fn call_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension`"
        )))
    }

    fn call_mutable_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension`",
        )))
    }
}

pub trait CreateObject {
    fn create(value: ObjectValue) -> Result<Self, VMError>
    where
        Self: Sized;

    fn post_deserialize(&mut self) {}
}

#[allow(unused_variables)]
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait Object:
    DynCompare + DynClone + DynHash + AsPrimitive<ObjectValue> + CreateObject + Definition + Send + Sync
{
    fn call(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call`"
        )))
    }

    fn call_extension(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension`"
        )))
    }

    fn call_mutable_extension(
        &mut self,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError>
    where
        Self: Sized,
    {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_mutable_extension`"
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

// todo first pass will use serde to read/write from bytes
#[cfg(feature = "snapshot")]
impl Snapshot for Box<dyn Object + '_> {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
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
