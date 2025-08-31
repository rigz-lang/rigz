mod ast;
mod debug;
mod docs;
mod format;
mod repl;
mod run;
mod test;
mod utils;

use crate::ast::{ast, AstArgs};
use crate::format::{format, FormatArgs};
use crate::repl::ReplArgs;
use crate::run::RunArgs;
use crate::test::TestArgs;
use clap::{CommandFactory, Parser, Subcommand};
use log::{warn, LevelFilter};
use repl::repl;
use run::run;
use test::test;
// use crate::docs::DocArgs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CLI {
    #[arg(
        short,
        long,
        env = "RIGZ_VERBOSE",
        default_value = "0",
        help = "0 - 4, sets the log level from Error - Trace, negative numbers disable all logging"
    )]
    verbose: i8,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Ast(AstArgs),
    Run(RunArgs),
    Repl(ReplArgs),
    Fmt(FormatArgs),
    // Docs(DocArgs),
    // Debug(DebugArgs),
    Test(TestArgs),
    // todo add a Lock command that verifies or writes a checksum of all URLs (and eventually packages)
    // todo add an Update command that gets latest checksum of all URLs (and eventually packages)
}

fn main() {
    let cli = CLI::parse();
    match cli.verbose {
        0 => log::set_max_level(LevelFilter::Error),
        1 => log::set_max_level(LevelFilter::Warn),
        2 => log::set_max_level(LevelFilter::Info),
        3 => log::set_max_level(LevelFilter::Debug),
        4 => log::set_max_level(LevelFilter::Trace),
        unsupported => {
            if unsupported.is_negative() {
                log::set_max_level(LevelFilter::Off);
            } else {
                log::set_max_level(LevelFilter::Warn);
                warn!("Unsupported Level {}, defaulting to warn", unsupported)
            }
        }
    }
    pretty_env_logger::init();
    match cli.command {
        None => {
            let mut c = CLI::command();
            c.print_help().expect("print_help failed");
        }
        Some(c) => {
            match c {
                Commands::Ast(args) => ast(args),
                Commands::Run(args) => run(args),
                Commands::Repl(args) => repl(args),
                Commands::Test(args) => test(args),
                // Commands::Docs(args) => docs(args),
                // Commands::Debug(args) => debug(args),
                Commands::Fmt(args) => format(args),
            }
        }
    }
}
