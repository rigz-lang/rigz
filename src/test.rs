use std::path::PathBuf;
use clap::Args;
use crate::debug::DebugArgs;

#[derive(Args)]
pub struct TestArgs {
    #[arg(short, long)]
    test_directory: PathBuf,
}

pub(crate) fn test(args: TestArgs) {

}