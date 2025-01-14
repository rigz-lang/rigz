use crate::utils::{current_dir, path_to_string, read_rigz_files};
use clap::Args;
use rigz_runtime::Runtime;
use rigz_vm::TestResults;
use std::fs::read_to_string;
use std::path::PathBuf;

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
                        println!("Running {}", path_to_string(&file));
                        let results = r.test();
                        total += results.clone();
                        println!("{results}")
                    }
                    Err(e) => eprintln!("Failed to parse tests {} - {e}", path_to_string(&file)),
                };
            }
            Err(e) => eprintln!("Failed to open {} - {e}", path_to_string(&file)),
        }
    }
    println!("{total}")
}
