use clap::Args;
use rigz_runtime::Runtime;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct DebugArgs {
    main: PathBuf,
}

#[allow(unused)]
pub(crate) fn debug(args: DebugArgs) {
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read main");
    let _runtime = Runtime::create(contents.as_str()).expect("Failed to create runtime");
    // todo create tui for debugging, show current frame w/ registers, allow interacting with VM
}
