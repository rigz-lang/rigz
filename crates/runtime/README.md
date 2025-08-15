# rigz_runtime
## Rust Minimum Version: 1.84

Handles parsing and converting rigz to its VM instructions. 

**If `threaded` feature is enabled, a tokio runtime is required. Enabled by default**

## WASM Support

**NOTE: Rigz is untested with nodejs and other WASM targets, the following assumes a running in browser (like Leptos)**

Three steps are required to use Rigz with the wasm32-unknown-unknown target:

1. Enable `js` feature and `no-default-features`, default runtime expects a multithreaded environment. 
2. Create `.cargo/config.toml` with the following
    ```toml
    [target.wasm32-unknown-unknown]
    rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
    ```
3. Add `__wasm_call_ctors` extern and call in main
    ```rust
    #[cfg(target_family = "wasm")]
    unsafe extern "C" {
        fn __wasm_call_ctors();
    }
    
    fn main() {
        #[cfg(target_family = "wasm")]
        unsafe {
            __wasm_call_ctors();
        }
    
        // ...
    }
    ```