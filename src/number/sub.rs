use std::ops::{Sub};
use crate::number::Number;
use crate::VMError;

impl Sub for Number {
    type Output = Result<Number, VMError>;

    fn sub(self, rhs: Self) -> Self::Output {
        let v = match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i - rhs.to_int()),
            (Number::UInt(u), rhs) => {
                match rhs.to_uint() {
                    Ok(rhs) => {
                        if rhs > u {
                            return Err(VMError::RuntimeError(format!("{} - {} would be negative, use ints", u, rhs)))
                        }
                        Number::UInt(u - rhs)
                    },
                    Err(e) => return Err(e)
                }
            },
            (Number::Float(f), rhs) => {
                Number::Float(f - rhs.to_float())
            }
        };
        Ok(v)
    }
}