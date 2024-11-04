pub trait Reverse {
    type Output;

    fn reverse(&self) -> Self::Output;
}

pub trait Logical<Rhs> {
    type Output;

    fn and(self, rhs: Rhs) -> Self::Output;
    fn or(self, rhs: Rhs) -> Self::Output;
    fn xor(self, rhs: Rhs) -> Self::Output;
    fn elvis(self, rhs: Rhs) -> Self::Output;
}