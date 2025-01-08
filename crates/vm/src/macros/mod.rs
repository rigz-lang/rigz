#[macro_export]
macro_rules! outln {
    () => {
        #[cfg(feature = "log_std_out")]
        log::info!("");
        #[cfg(not(feature = "log_std_out"))]
        println!();
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "log_std_out")]
        log::info!($($arg)*);
        #[cfg(not(feature = "log_std_out"))]
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! out {
    () => {
        #[cfg(feature = "log_std_out")]
        log::info!("");
        #[cfg(not(feature = "log_std_out"))]
        print!()
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "log_std_out")]
        log::info!($($arg)*);
        #[cfg(not(feature = "log_std_out"))]
        print!($($arg)*);
    }};
}

#[macro_export]
macro_rules! err {
    () => {
        #[cfg(feature = "log_std_out")]
        log::error!("");
        #[cfg(not(feature = "log_std_out"))]
        eprint!();
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "log_std_out")]
        log::error!($($arg)*);
        #[cfg(not(feature = "log_std_out"))]
        eprint!($($arg)*);
    }};
}

#[macro_export]
macro_rules! errln {
    () => {
        #[cfg(feature = "log_std_out")]
        log::error!();
        #[cfg(not(feature = "log_std_out"))]
        eprintln!();
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "log_std_out")]
        log::error!($($arg)*);
        #[cfg(not(feature = "log_std_out"))]
        eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! define_value_tests {
    ($op:tt { $($test_name:ident => ($val1:expr, $val2:expr, $expected:expr));* $(;)? }) => {
        $(
            #[test]
            fn $test_name() {
                assert_eq!($expected, &$val1 $op &$val2);
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
