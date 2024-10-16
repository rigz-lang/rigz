mod add;
mod bitand;
mod bitor;
mod bitxor;
mod div;
mod mul;
mod neg;
mod not;
mod rem;
mod rev;
mod shl;
mod shr;
mod sub;
mod logical;

use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::VMError;

#[derive(Copy, Clone, Debug)]
pub enum Number {
    Int(i64),
    UInt(u64),
    Float(f64),
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Number::Int(v) => v.hash(state),
            Number::UInt(v) => v.hash(state),
            Number::Float(v) => v.to_bits().hash(state),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(i) => {
                write!(f, "{}", *i)
            }
            Number::UInt(u) => {
                write!(f, "{}", *u)
            }
            Number::Float(v) => {
                write!(f, "{}", *v)
            }
        }
    }
}

impl Eq for Number {}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Number::Int(a), &Number::Int(b)) => a == b,
            (&Number::UInt(a), &Number::UInt(b)) => a == b,
            (&Number::Float(a), &Number::Float(b)) => a == b,
            (&Number::Int(a), &Number::UInt(b)) =>  {
                if a.is_negative() {
                    return false
                }
                a as u64 == b
            }
            (&Number::Int(a), &Number::Float(b)) => {
                a as f64 == b
            }
            (&Number::UInt(a), &Number::Int(b)) => {
                if b.is_negative() {
                    return false
                }
                a == b as u64
            }
            (&Number::UInt(a), &Number::Float(b)) => {
                if b.is_sign_negative() {
                    return false
                }
                a == b as u64
            }
            (&Number::Float(a), &Number::Int(b)) => {
                a == b as f64
            }
            (&Number::Float(a), &Number::UInt(b)) => {
                a == b as f64
            }
        }
    }
}

impl Number {
    #[inline]
    pub fn is_one(&self) -> bool {
        match self {
            Number::Int(i) => *i == 1,
            Number::UInt(u) => *u == 1,
            Number::Float(f) => *f == 1.0
        }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        match self {
            Number::Int(i) => *i == 0,
            Number::UInt(u) => *u == 0,
            Number::Float(f) => *f == 0.0
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            Number::Int(i) => *i as f64,
            Number::UInt(u) => *u as f64,
            Number::Float(f) => *f,
        }
    }

    pub fn to_int(&self) -> i64 {
        match self {
            Number::Int(i) => *i,
            Number::UInt(u) => *u as i64,
            Number::Float(f) => *f as i64,
        }
    }

    pub fn to_uint(&self) -> Result<u64, VMError> {
        if self.is_negative() {
            return Err(VMError::ConversionError("Cannot convert negative to UINT".to_string()))
        }
        let u = match self {
            Number::Int(i) => *i as u64,
            Number::UInt(u) => *u,
            Number::Float(f) => *f as u64,
        };
        Ok(u)
    }

    pub fn to_usize(&self) -> Result<usize, VMError> {
        if self.is_negative() {
            return Err(VMError::ConversionError("Cannot convert negative to UINT".to_string()))
        }
        let u = match self {
            Number::Int(i) => *i as usize,
            Number::UInt(u) => *u as usize,
            Number::Float(f) => *f as usize,
        };
        Ok(u)
    }

    pub fn is_negative(&self) -> bool {
        match self {
            Number::Int(i) => i.is_negative(),
            Number::UInt(_) => false,
            Number::Float(f) => f.is_sign_negative(),
        }
    }
}