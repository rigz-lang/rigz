mod builder;

#[macro_export]
macro_rules! define_value_tests {
    ($op:tt { $($test_name:ident => ($val1:expr, $val2:expr, $expected:expr));* $(;)? }) => {
        $(
            #[test]
            fn $test_name() {
                assert_eq!($expected, $val1 $op $val2);
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
