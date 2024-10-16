use std::ops::Add;
use crate::number::Number;
use crate::VMError;

impl Add for Number {
    type Output = Result<Number, VMError>;

    fn add(self, rhs: Self) -> Self::Output {
        let v = match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i + rhs.to_int()),
            (Number::UInt(u), rhs) => {
                match rhs.to_uint() {
                    Ok(rhs) => Number::UInt(u + rhs),
                    Err(e) => return Err(e)
                }
            },
            (Number::Float(f), rhs) => {
                Number::Float(f + rhs.to_float())
            }
        };
        Ok(v)
    }
}