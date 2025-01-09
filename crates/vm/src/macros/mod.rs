#[macro_export]
macro_rules! handle_js {
    ($enabled: expr, $default: expr) => {
        #[cfg(feature = "js")]
        $enabled;
        #[cfg(not(feature = "js"))]
        $default;
    };
}

#[macro_export]
macro_rules! outln {
    () => {
        handle_js! {
            web_sys::console::log_0(),
            println!()
        }
    };
    ($($arg:tt)*) => {{
        handle_js! {
            web_sys::console::log_1(&format_args!($($arg)*).to_string().into()),
            println!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! out {
    () => {
        handle_js! {
            web_sys::console::log_0(),
            print!()
        }
    };
    ($($arg:tt)*) => {{
        handle_js! {
            web_sys::console::log_1(&format_args!($($arg)*).to_string().into()),
            print!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! err {
    () => {
        handle_js! {
           web_sys::console::error_0(),
           eprint!()
        }
    };
    ($($arg:tt)*) => {{
        handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprint!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! errln {
    () => {
        handle_js! {
           web_sys::console::error_0(),
           eprintln!()
        }
    };
    ($($arg:tt)*) => {{
        handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprintln!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! define_value_tests {
    ($op:tt { $($test_name:ident => ($lhs:expr, $rhs:expr) = $expected:expr);* $(;)? }) => {
        use $crate::value::Value;
        use wasm_bindgen_test::*;

        $(
            #[wasm_bindgen_test(unsupported = test)]
            fn $test_name() {
                let lhs: Value = $lhs.into();
                let expected: Value = $expected.into();
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
