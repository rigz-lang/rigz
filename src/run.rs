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
    match rigz_runtime::parse(contents.as_str()) {
        Ok(mut vm) => {
            match vm.run() {
                Ok(v) => {
                    if v != Value::None {
                        println!("{}", v)
                    }
                }
                Err(e) => {
                    eprintln!("VM Run Failed: {:?}", e);
                    exit(1)
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to parse input: {:?}", e);
            exit(1)
        }
    }
}