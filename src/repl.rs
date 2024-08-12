use clap::Args;
use rigz_runtime::VM;
use rustyline::DefaultEditor;

#[derive(Args)]
pub struct ReplArgs {
    #[arg(short, long, default_value = "false")]
    persist_output: bool,
}

pub(crate) fn repl(args: ReplArgs) {
    let mut r = DefaultEditor::new().expect("Failed to create REPL");

    loop {
        let next = r.readline(">").expect("Failed to read line");

    }
}