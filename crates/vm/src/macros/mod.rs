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
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.out.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        o.applied("\n".to_string())
                    }
                }
                Err(_) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }
        $crate::handle_js! {
            web_sys::console::log_0(),
            println!()
        }
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.out.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        let mut s = format_args!($($arg)*).to_string();
                        s.push('\n');
                        o.applied(s)
                    }
                }
                Err(_) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }

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
            print!("")
        }
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.out.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        o.applied(format_args!($($arg)*).to_string())
                    }
                }
                Err(e) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }

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
           eprint!("")
        }
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.err.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        o.applied(format_args!($($arg)*).to_string())
                    }
                }
                Err(e) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }

        $crate::handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprint!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! errln {
    () => {
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.err.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        o.applied("\n".to_string())
                    }
                }
                Err(_) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }

        $crate::handle_js! {
           web_sys::console::error_0(),
           eprintln!()
        }
    };
    ($($arg:tt)*) => {{
        #[cfg(feature = "std_capture")]
        {
            match $crate::CAPTURE.out.read() {
                Ok(curr) => {
                    if let Some(o) = curr.deref() {
                        let mut s = format_args!($($arg)*).to_string();
                        s.push('\n');
                        o.applied(s)
                    }
                }
                Err(_) => {
                    // todo notify that RwLock is poisoned
                }
            }
        }

        $crate::handle_js! {
           web_sys::console::error_1(&format_args!($($arg)*).to_string().into()),
           eprintln!($($arg)*)
        }
    }};
}
