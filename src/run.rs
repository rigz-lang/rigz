use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use clap::Args;
use rigz_runtime::eval;

#[derive(Args)]
pub struct RunArgs {
    main: PathBuf,
    #[arg(short, long, default_value = "false")]
    show_output: bool,
}

pub(crate) fn run(args: RunArgs) {
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read main");
    match eval(contents.as_str()) {
        Err(e) => {
            eprintln!("VM Run Failed: {:?}", e);
            exit(1)
        }
        Ok(v) if args.show_output => {
            println!("{v}")
        }
        Ok(_) => {}
    }
}