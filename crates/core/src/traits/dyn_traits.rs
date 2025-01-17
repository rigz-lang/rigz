use std::any::Any;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

// DynCompare & DynHash are from https://quinedot.github.io/rust-learning/dyn-trait-examples.html
pub trait AsDynCompare: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_dyn_compare(&self) -> &dyn DynCompare;
}

// Sized types only
impl<T: Any + DynCompare> AsDynCompare for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_dyn_compare(&self) -> &dyn DynCompare {
        self
    }
}

pub trait DynCompare: AsDynCompare {
    fn dyn_eq(&self, other: &dyn DynCompare) -> bool;

    fn dyn_partial_cmp(&self, other: &dyn DynCompare) -> Option<Ordering>;
}

impl<T: Any + PartialEq + PartialOrd> DynCompare for T {
    fn dyn_eq(&self, other: &dyn DynCompare) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_partial_cmp(&self, other: &dyn DynCompare) -> Option<Ordering> {
        other
            .as_any()
            .downcast_ref::<Self>()
            .and_then(|other| self.partial_cmp(other))
    }
}

impl PartialEq<dyn DynCompare> for dyn DynCompare {
    fn eq(&self, other: &dyn DynCompare) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialOrd<dyn DynCompare> for dyn DynCompare {
    fn partial_cmp(&self, other: &dyn DynCompare) -> Option<Ordering> {
        self.dyn_partial_cmp(other)
    }
}

pub trait DynHash {
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Hash> DynHash for T {
    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state)
    }
}

impl Hash for dyn DynHash + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state)
    }
}
