use clap::Args;
use rigz_runtime::Runtime;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

#[derive(Args)]
pub struct TestArgs {
    #[arg(help = "Test Entrypoint")]
    input: PathBuf,
}

fn read_files(input: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::with_capacity(1);
    if input.is_dir() {
        for f in input.read_dir()? {
            files.extend(read_files(f?.path())?);
        }
    } else {
        files.push(input);
    }
    Ok(files)
}

pub(crate) fn test(args: TestArgs) {
    let test_files = read_files(args.input).expect("Failed to open test files");
    // # of tests
    for file in test_files {
        match read_to_string(&file) {
            Ok(s) => {
                match Runtime::create_unverified(s) {
                    Ok(mut r) => {
                        println!("Running {}", path_to_string(&file));
                        let results = r.test();
                        println!("{results}")
                    }
                    Err(e) => eprintln!("Failed to parse tests {} - {e}", path_to_string(&file)),
                };
            }
            Err(e) => eprintln!("Failed to open {} - {e}", path_to_string(&file)),
        }
    }
    // final result, passed, failed, ignored, finished in 0.0s
}

fn path_to_string(path_buf: &PathBuf) -> String {
    match path_buf.to_str() {
        None => format!("Invalid Path {path_buf:?}"),
        Some(s) => s.to_string(),
    }
}
