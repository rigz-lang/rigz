#[macro_export]
macro_rules! handle_js {
    ($enabled: expr, $default: expr) => {
        #[cfg(feature = "js")]
        {
            $enabled;
        }
        #[cfg(not(feature = "js"))]
        {
            $default;
        }
    };
}

#[macro_export]
macro_rules! outln {
    () => {
        $crate::handle_js! {
            web_sys::console::log_0(),
            println!()
        }
    };
    ($($arg:tt)*) => {{
        $crate::handle_js! {
            web_sys::console::log_1(&format_args!($($arg)*).to_string().into()),
            println!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! out {
    () => {
        $crate::handle_js! {
            web_sys::console::log_0(),
            print!()
        }
    };
    ($($arg:tt)*) => {{
        $crate::handle_js! {
            web_sys::console::log_1(&format_args!($($arg)*).to_string().into()),
            print!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! err {
    () => {
        $crate::handle_js! {
           web_sys::console::error_0(),
           eprint!()
        }
    };
    ($($arg:tt)*) => {{
        $crate::handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprint!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! errln {
    () => {
        $crate::handle_js! {
           web_sys::console::error_0(),
           eprintln!()
        }
    };
    ($($arg:tt)*) => {{
        $crate::handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprintln!($($arg)*)
        }
    }};
}
