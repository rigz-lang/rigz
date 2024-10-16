mod add;
mod bitand;
mod bitor;
mod bitxor;
mod div;
mod logical;
mod mul;
mod neg;
mod not;
mod rem;
mod rev;
mod shl;
mod shr;
mod sub;

use crate::{impl_from, impl_from_cast, VMError};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Number(pub f64);

impl Number {
    #[inline]
    pub fn new(v: f64) -> Self {
        Number(v)
    }
}

impl From<f64> for Number {
    #[inline]
    fn from(value: f64) -> Self {
        Number(value)
    }
}

impl_from_cast! {
    i32 as f64, Number, Number::new;
    i64 as f64, Number, Number::new;
    u32 as f64, Number, Number::new;
    u64 as f64, Number, Number::new;
    f32 as f64, Number, Number::new;
}

impl From<bool> for Number {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Number::one()
        } else {
            Number::zero()
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for Number {}

impl FromStr for Number {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.contains('.') => match s.parse::<f64>() {
                Ok(f) => Ok(f.into()),
                Err(e) => Err(e.to_string()),
            },
            _ if s.ends_with('u') => {
                let s = s[..s.len() - 1].to_string();
                match s.parse::<u64>() {
                    Ok(u) => Ok(u.into()),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ if s.ends_with('f') => {
                let s = s[..s.len() - 1].to_string();
                match s.parse::<f64>() {
                    Ok(u) => Ok(u.into()),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ if s.ends_with('i') => {
                let s = s[..s.len() - 1].to_string();
                match s.parse::<i64>() {
                    Ok(u) => Ok(u.into()),
                    Err(e) => Err(e.to_string()),
                }
            }
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
        Number(0.0)
    }

    #[inline]
    pub fn one() -> Number {
        Number(1.0)
    }

    #[inline]
    pub fn is_one(self) -> bool {
        self.0 == 1.0
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    #[inline]
    pub fn to_float(self) -> f64 {
        self.0
    }

    #[inline]
    pub fn to_int(self) -> i64 {
        self.0 as i64
    }

    #[inline]
    pub fn to_uint(self) -> Result<u64, VMError> {
        if self.is_negative() {
            return Err(VMError::ConversionError(
                "Cannot convert negative to UINT".to_string(),
            ));
        }
        Ok(self.0 as u64)
    }

    #[inline]
    pub fn to_usize(self) -> Result<usize, VMError> {
        if self.is_negative() {
            return Err(VMError::ConversionError(
                "Cannot convert negative to UINT".to_string(),
            ));
        }
        Ok(self.0 as usize)
    }

    #[inline]
    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative()
    }
}
