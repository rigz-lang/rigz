mod as_primitive;
mod dyn_traits;
#[cfg(feature = "snapshot")]
mod snapshot;

use crate::{ObjectValue, RigzArgs, VMError};
pub use as_primitive::{AsPrimitive, WithTypeInfo};
use dyn_clone::DynClone;
pub use dyn_traits::*;
use log::{error, warn};
use mopa::mopafy;
#[cfg(feature = "snapshot")]
pub use snapshot::Snapshot;
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::vec::IntoIter;

pub trait Definition {
    fn name() -> &'static str
    where
        Self: Sized;

    fn trait_definition() -> &'static str
    where
        Self: Sized;
}

pub struct Dependency {
    pub create: fn(RigzArgs) -> Result<Box<dyn Object>, VMError>,
    pub call: fn(String, RigzArgs) -> Result<ObjectValue, VMError>,
}

impl Debug for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dependency")
    }
}

impl Dependency {
    pub fn new<T: Object + 'static>() -> Self {
        Self {
            create: |value| Ok(Box::new(T::create(value)?)),
            call: |func, args| T::call(func, args),
        }
    }
}

// todo convert function: String to function: usize?

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
            "{self:?} does not implement `call` - {function}"
        )))
    }

    fn call_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension` - {function}"
        )))
    }

    fn call_mutable_extension(
        &self,
        this: Rc<RefCell<ObjectValue>>,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension` - {function}",
        )))
    }
}

pub trait CreateObject {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized;

    fn post_deserialize(&mut self) {}
}

#[allow(unused_variables)]
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait Object:
    mopa::Any
    + DynCompare
    + DynClone
    + DynHash
    + AsPrimitive<ObjectValue>
    + CreateObject
    + Definition
    + Send
    + Sync
{
    fn call(function: String, args: RigzArgs) -> Result<ObjectValue, VMError>
    where
        Self: Sized,
    {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call` - {function}",
            Self::name()
        )))
    }

    fn call_extension(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_extension` - {function}"
        )))
    }

    fn call_mutable_extension(
        &mut self,
        function: String,
        args: RigzArgs,
    ) -> Result<Option<ObjectValue>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{self:?} does not implement `call_mutable_extension` - {function}"
        )))
    }
}

mopafy!(Object);
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
        error!("Snapshot is not supported for Objects yet - {self:?}");
        vec![]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Err(VMError::todo("Snapshot is not supported for Objects yet"))
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
