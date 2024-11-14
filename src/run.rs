use clap::Args;
use rigz_runtime::eval;
use rigz_runtime::runtime::eval_print_vm;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

#[derive(Args)]
pub struct RunArgs {
    #[arg(help = "Rigz Entrypoint")]
    main: PathBuf,
    #[arg(short, long, default_value = "false", help = "Show output from eval")]
    show_output: bool,
    #[arg(short, long, default_value = "false", help = "Print VM before run")]
    print_vm: bool,
}

pub(crate) fn run(args: RunArgs) {
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read main");
    let v = if args.print_vm {
        eval_print_vm(contents.as_str())
    } else {
        eval(contents.as_str())
    };
    match v {
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
