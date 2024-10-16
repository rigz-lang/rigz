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
