use crate::number::Number;
use crate::VMError;
use std::ops::BitXor;

impl BitXor for Number {
    type Output = Result<Number, VMError>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let v = match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i ^ rhs.to_int()),
            (Number::UInt(u), rhs) => match rhs.to_uint() {
                Ok(rhs) => Number::UInt(u ^ rhs),
                Err(e) => return Err(e),
            },
            (Number::Float(f), rhs) => {
                Number::Float(f64::from_bits(f.to_bits() ^ rhs.to_float().to_bits()))
            }
        };
        Ok(v)
    }
}
