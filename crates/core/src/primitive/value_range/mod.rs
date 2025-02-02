mod ops;

use crate::{impl_from, PrimitiveValue};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Neg, Range};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueRange {
    Int(Range<i64>),
    Char(Range<char>),
}

fn range_compare<Idx: PartialOrd>(a: &Range<Idx>, b: &Range<Idx>) -> Ordering {
    if a.start <= b.start && a.end <= b.end {
        Ordering::Less
    } else if a.end >= b.end {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

impl PartialOrd for ValueRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueRange {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        match (self, other) {
            (ValueRange::Int(a), ValueRange::Int(b)) => range_compare(a, b),
            (ValueRange::Char(a), ValueRange::Char(b)) => range_compare(a, b),
            (ValueRange::Int(_), ValueRange::Char(_)) => Ordering::Less,
            (ValueRange::Char(_), ValueRange::Int(_)) => Ordering::Greater,
        }
    }
}

impl Neg for &ValueRange {
    type Output = ValueRange;

    fn neg(self) -> Self::Output {
        match self {
            ValueRange::Int(i) => ValueRange::Int(Range {
                start: -i.start,
                end: i.end,
            }),
            ValueRange::Char(a) => ValueRange::Char(a.clone()),
        }
    }
}

impl ValueRange {
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            ValueRange::Int(r) => r.is_empty(),
            ValueRange::Char(r) => r.is_empty(),
        }
    }
    pub(crate) fn to_map(&self) -> IndexMap<PrimitiveValue, PrimitiveValue> {
        match self {
            ValueRange::Int(r) => r.clone().map(|v| (v.into(), v.into())).collect(),
            ValueRange::Char(r) => r
                .clone()
                .map(|v| (v.to_string().into(), v.to_string().into()))
                .collect(),
        }
    }

    pub(crate) fn to_list(&self) -> Vec<PrimitiveValue> {
        match self {
            ValueRange::Int(r) => r.clone().map(|v| v.into()).collect(),
            ValueRange::Char(r) => r.clone().map(|v| v.to_string().into()).collect(),
        }
    }
}

impl Display for ValueRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueRange::Int(r) => write!(f, "{}..{}", r.start, r.end),
            ValueRange::Char(r) => write!(f, "{}..{}", r.start, r.end),
        }
    }
}

impl_from! {
    Range<i64>, ValueRange, ValueRange::Int;
    Range<char>, ValueRange, ValueRange::Char;
}
