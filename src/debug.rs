use std::path::PathBuf;
use clap::Args;
use crate::run::RunArgs;

#[derive(Args)]
pub struct DebugArgs {
    #[arg(short, long)]
    main: PathBuf,
}

pub(crate) fn debug(args: DebugArgs) {

}