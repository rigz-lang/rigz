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

use crate::{impl_from, impl_from_cast, VMError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl_from! {
    i64, Number, Number::Int;
    f64, Number, Number::Float;
}

impl_from_cast! {
    i32 as i64, Number, Number::Int;
    u32 as i64, Number, Number::Int;
    f32 as f64, Number, Number::Float;
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Number::Int(v) => v.hash(state),
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
            Number::Float(v) => {
                write!(f, "{}", *v)
            }
        }
    }
}

impl Eq for Number {}

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Number::Int(a), &Number::Int(b)) => a == b,
            (&Number::Float(a), &Number::Float(b)) => a == b,
            (&Number::Int(a), &Number::Float(b)) => a as f64 == b,
            (&Number::Float(a), &Number::Int(b)) => a == b as f64,
        }
    }
}

impl FromStr for Number {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.contains('.') => match s.parse::<f64>() {
                Ok(f) => Ok(f.into()),
                Err(e) => Err(e.to_string()),
            },
            _ => match s.parse::<i64>() {
                Ok(i) => Ok(i.into()),
                Err(e) => Err(e.to_string()),
            },
        }
    }
}

impl Number {
    #[inline]
    pub fn zero() -> Number {
        Number::Int(0)
    }

    #[inline]
    pub fn one() -> Number {
        Number::Int(1)
    }

    #[inline]
    pub fn is_one(self) -> bool {
        match self {
            Number::Int(i) => i == 1,
            Number::Float(f) => f == 1.0,
        }
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        match self {
            Number::Int(i) => i == 0,
            Number::Float(f) => f == 0.0,
        }
    }

    #[inline]
    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Float(f) => f,
        }
    }

    #[inline]
    pub fn to_int(self) -> i64 {
        match self {
            Number::Int(i) => i,
            Number::Float(f) => f as i64,
        }
    }

    #[inline]
    pub fn to_bits(self) -> u64 {
        match self {
            Number::Int(i) => u64::from_le_bytes(i.to_le_bytes()),
            Number::Float(f) => f.to_bits(), // this could panic, may need to switch to le_bytes
        }
    }

    #[inline]
    pub fn to_usize(self) -> Result<usize, VMError> {
        if self.is_negative() {
            return Err(VMError::ConversionError(
                "Cannot convert negative to UINT".to_string(),
            ));
        }
        let u = match self {
            Number::Int(i) => i as usize,
            Number::Float(f) => f as usize,
        };
        Ok(u)
    }

    #[inline]
    pub fn is_negative(&self) -> bool {
        match self {
            Number::Int(i) => i.is_negative(),
            Number::Float(f) => f.is_sign_negative(),
        }
    }
}
