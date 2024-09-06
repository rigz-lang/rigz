use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use clap::Args;
use rigz_runtime::Value;

#[derive(Args)]
pub struct RunArgs {
    #[arg(short, long)]
    main: PathBuf,
    #[arg(short, long, default_value = "false")]
    binary: bool,
}

pub(crate) fn run(args: RunArgs) {
    let input = args.main;
    let mut file = File::open(input).expect("failed to open input");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read contents");
    let mut runtime = rigz_runtime::Runtime::prepare(contents.as_str()).expect("Failed to parse input");
    match runtime.run() {
        Ok(v) => {
            println!("{}", v)
        }
        Err(e) => {
            eprintln!("VM Run Failed: {:?}", e);
            exit(1)
        }
    }
}