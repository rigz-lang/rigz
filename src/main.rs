mod run;
mod repl;
mod test;
mod debug;

use std::io::Read;
use std::path::PathBuf;
use clap::{Args, CommandFactory, Parser, Subcommand};
use crate::debug::DebugArgs;
use crate::repl::ReplArgs;
use crate::run::RunArgs;
use crate::test::TestArgs;
use debug::debug;
use log::{log, warn, LevelFilter};
use repl::repl;
use run::run;
use test::test;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CLI {
    #[arg(short, long, env = "RIGZ_VERBOSE", default_value = "0")]
    verbose: i8,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Run(RunArgs),
    // Repl(ReplArgs),
    // Debug(DebugArgs),
    // Test(TestArgs)
}


fn main() {
    let cli = CLI::parse();
    pretty_env_logger::init();
    match cli.verbose {
        0 => {
            log::set_max_level(LevelFilter::Error)
        }
        1 => {
            log::set_max_level(LevelFilter::Warn)
        }
        2 => {
            log::set_max_level(LevelFilter::Info)
        }
        3 => {
            log::set_max_level(LevelFilter::Debug)
        }
        4 => {
            log::set_max_level(LevelFilter::Trace)
        }
        unsupported => {
            if unsupported.is_negative() {
                log::set_max_level(LevelFilter::Off);
            } else {
                log::set_max_level(LevelFilter::Warn);
                warn!("Unsupported Level {}, defaulting to warn", unsupported)
            }
        }
    }
    match cli.command {
        None => {
            let mut c = CLI::command();
            c.print_help().expect("print_help failed");
        }
        Some(c) => {
            match c {
                Commands::Run(args) => run(args),
                // Commands::Repl(args) => repl(args),
                // Commands::Debug(args) => debug(args),
                // Commands::Test(args) => test(args),
            }
        }
    }
}
