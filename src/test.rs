use crate::utils::{current_dir, path_to_string, read_rigz_files};
use clap::Args;
use rigz_runtime::Runtime;
use rigz_vm::{Lifecycle, TestResults, VMError};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;

#[derive(Args)]
pub struct TestArgs {
    #[arg(help = "Test Entrypoint, defaults to current directory")]
    input: Option<PathBuf>,
}

pub(crate) fn test(args: TestArgs) {
    let input = args.input.unwrap_or_else(current_dir);
    let test_files = read_rigz_files(input).expect("Failed to open test files");
    // # of tests
    let mut total = TestResults::default();
    for file in test_files {
        match read_to_string(&file) {
            Ok(s) => {
                match Runtime::create_unverified(s) {
                    Ok(mut r) => {
                        if r.vm()
                            .scopes
                            .iter()
                            .filter(|s| matches!(s.lifecycle, Some(Lifecycle::Test(_))))
                            .count()
                            == 0
                        {
                            continue;
                        }
                        println!("Running {}", path_to_string(&file));
                        let results = r.test();
                        total += results.clone();
                        println!("{results}")
                    }
                    Err(e) => {
                        total.failure_messages.push((
                            format!("{} - Create Runtime Failed", path_to_string(&file)),
                            VMError::RuntimeError(e.to_string()),
                        ));
                    }
                };
            }
            Err(e) => eprintln!("Failed to open {} - {e}", path_to_string(&file)),
        }
    }
    println!("{total}");
    if !total.failure_messages.is_empty() {
        exit(1)
    }
}
