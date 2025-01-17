use clap::Args;
use rigz_runtime::Runtime;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct AstArgs {
    #[arg(help = "Rigz Entrypoint")]
    main: PathBuf,
    #[arg(short, long, default_value = "false", help = "Print VM before run")]
    vm: bool,
}

pub(crate) fn ast(args: AstArgs) {
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read main");
    let str = contents;
    let program = rigz_ast::parse(&str, false).expect("Failed to read input");
    println!("AST:\n{program:#?}");
    if args.vm {
        let vm = Runtime::create(str).expect("Failed to create VM");
        println!("\nVM:\n{:#?}", vm.vm())
    }
}
