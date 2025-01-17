#[macro_export]
macro_rules! define_value_tests {
    ($op:tt { $($test_name:ident => ($lhs:expr, $rhs:expr) = $expected:expr);* $(;)? }) => {
        use $crate::PrimitiveValue;
        use wasm_bindgen_test::*;

        $(
            #[wasm_bindgen_test(unsupported = test)]
            fn $test_name() {
                let lhs: PrimitiveValue = $lhs.into();
                let expected: PrimitiveValue = $expected.into();
                assert_eq!(expected, &lhs $op &$rhs.into());
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_from {
    ($($From:ty, $To:ty, $Constructor:expr;)*) => {
        $(
            impl From<$From> for $To {
                #[inline]
                fn from(value: $From) -> Self {
                    $Constructor(value)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_from_cast {
    ($($From:ty as $cast:ty, $To:ty, $Constructor:expr;)*) => {
        $(
            impl From<$From> for $To {
                #[inline]
                fn from(value: $From) -> Self {
                    $Constructor(value as $cast)
                }
            }
        )*
    };
}
