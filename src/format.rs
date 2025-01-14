use crate::utils::{current_dir, path_to_string, read_rigz_files};
use clap::Args;
use rigz_ast::Parser;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Args)]
pub struct FormatArgs {
    #[arg(help = "Formatter Entrypoint, defaults to current directory")]
    input: Option<PathBuf>,
}

pub(crate) fn format(args: FormatArgs) {
    let input = args.input.unwrap_or_else(current_dir);
    let files = read_rigz_files(input).expect("failed to read input files");
    for file in files {
        match read_to_string(&file) {
            Ok(input) => {
                let formatted = rigz_ast::format(input);
                let mut output = match File::open(&file) {
                    Ok(output) => output,
                    Err(e) => {
                        eprintln!(
                            "Failed to open {} for writing - {}",
                            path_to_string(&file),
                            e
                        );
                        continue;
                    }
                };
                if let Err(e) = output.write_all(formatted.as_bytes()) {
                    eprintln!(
                        "Failed to write formatted value to {} - {}",
                        path_to_string(&file),
                        e
                    );
                }
            }
            Err(e) => eprintln!("Failed to open {} - {}", path_to_string(&file), e),
        }
    }
}
