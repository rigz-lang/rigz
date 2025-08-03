use clap::Args;
use rigz_runtime::eval;
use rigz_runtime::runtime::eval_print_vm;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

#[derive(Args)]
pub struct RunArgs {
    #[arg(help = "Rigz Entrypoint")]
    main: PathBuf,
    #[arg(short, long, default_value = "false", help = "Show output from eval")]
    output: bool,
    #[arg(short, long, default_value = "false", help = "Print VM before run")]
    debug_vm: bool,
    #[arg(short, long, default_value = "false", help = "Time run")]
    timed: bool,
}

pub(crate) fn run(args: RunArgs) {
    let start = if args.timed {
        Some(Instant::now())
    } else {
        None
    };
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read main");
    let v = if args.debug_vm {
        eval_print_vm(contents)
    } else {
        eval(contents)
    };
    if let Some(s) = start {
        println!("Elapsed: {:?}", s.elapsed())
    }
    match v {
        Err(e) => {
            eprintln!("VM Run Failed: {:?}", e);
            exit(1)
        }
        Ok(v) if args.output => {
            println!("{v}")
        }
        Ok(_) => {}
    }
}
