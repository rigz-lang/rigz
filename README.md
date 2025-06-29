# rigz

## Rust Minimum Version: 1.84

## Installation
`cargo install rigz`

## Usage

`rigz <command>`

If no command is passed in the help message is displayed

#### Optional Arguments
Before <command> the following args are valid

##### Log Level (verbose 0 - 4)

- 0 error
- 1 warn
- 2 info
- 3 debug
- 4 trace
- Any negative number can be used to disable all logging output

defaults to 0 can be set with one of the following:

- `-v 3`
- `--verbose 4`
- RIGZ_VERBOSE environment variable, `RIGZ_VERBOSE=2`

### Commands
- version (-V, --version)
- help (-h, --help, or no arguments)
- repl
- run
- test
- debug (coming soon)


### REPL
Interactive console to run rigz, use `exit` to end session.

Usage: `rigz repl [OPTIONS]`

#### Options:
- `-s, --save-history`:  Save History on exit
- `-h, --help`: Print help

### Run
Run a file

Usage: `rigz run [OPTIONS] <MAIN>`

#### Arguments:
- `<MAIN>`: Rigz Entrypoint

#### Options:
- `-s, --show-output`: Show output from eval
- `-p, --print-vm`: Print VM before run
- `-h, --help`: Print help

### Test
Test all functions with @test lifecycle

Usage: `rigz test <INPUT>`

#### Arguments:
- `<INPUT>`: Test Entrypoint

#### Options:
- `-h, --help`: Print help


### WASM

When running against a wasm target you'll need to do two things

1. Use feature `js` to support some rigz library functions
2. Add the following to your main method, this is done to support [inventory](https://docs.rs/inventory/0.3.20/inventory/index.html#webassembly-and-constructors) being used internally
   - Rigz repl has an example as well
   - ```rust
     #[cfg(target_family = "wasm")]
     unsafe extern "C" {
         fn __wasm_call_ctors();
     }
   
     fn main() {
        #[cfg(target_family = "wasm")]
        unsafe {
         __wasm_call_ctors();
        }
     }
     ```

