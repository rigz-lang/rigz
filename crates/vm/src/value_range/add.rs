use crate::value_range::ValueRange;
use crate::Number;
use std::ops::{Add, Range};

impl Add for &ValueRange {
    type Output = Option<ValueRange>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueRange::Int(a), ValueRange::Int(b)) => Some(ValueRange::Int(Range {
                start: a.start + b.start,
                end: a.end + b.end,
            })),
            (ValueRange::Char(a), ValueRange::Char(b)) => {
                let start = match char::from_u32(a.start as u32 + b.start as u32) {
                    None => return None,
                    Some(c) => c,
                };
                let end = match char::from_u32(a.end as u32 + b.end as u32) {
                    None => return None,
                    Some(c) => c,
                };
                Some(ValueRange::Char(Range { start, end }))
            }
            (ValueRange::Int(a), ValueRange::Char(b))
            | (ValueRange::Char(b), ValueRange::Int(a)) => {
                let start = match char::from_u32(a.start as u32 + b.start as u32) {
                    None => return None,
                    Some(c) => c,
                };
                let end = match char::from_u32(a.end as u32 + b.end as u32) {
                    None => return None,
                    Some(c) => c,
                };
                Some(ValueRange::Char(Range { start, end }))
            }
        }
    }
}

impl Add<&Number> for &ValueRange {
    type Output = Option<ValueRange>;

    fn add(self, rhs: &Number) -> Self::Output {
        let rhs = rhs.to_int();
        match self {
            ValueRange::Int(r) => Some(ValueRange::Int(Range {
                start: r.start + rhs,
                end: r.end + rhs,
            })),
            ValueRange::Char(r) => {
                let start = match char::from_u32(r.start as u32 + rhs as u32) {
                    None => return None,
                    Some(c) => c,
                };
                let end = match char::from_u32(r.end as u32 + rhs as u32) {
                    None => return None,
                    Some(c) => c,
                };
                Some(ValueRange::Char(Range { start, end }))
            }
        }
    }
}
