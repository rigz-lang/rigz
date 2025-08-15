# rigz_runtime
## Rust Minimum Version: 1.84

Handles parsing and converting rigz to its VM instructions. 

**If `threaded` feature is enabled, a tokio runtime is required. Enabled by default**

## WASM Support

Create `.cargo/config.toml` with the following

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
```