use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use clap::Args;
use rigz_ast::{generate_docs, Element, FunctionDeclaration, ParserOptions, Statement};

#[derive(Args)]
pub struct DocArgs {
    #[arg(help = "Rigz Entrypoint")]
    main: PathBuf,
}

pub(crate) fn docs(args: DocArgs) -> io::Result<()> {
    let mut file = File::open(args.main).expect("Failed to open main");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read main");
    let str = contents;
    let mut options = ParserOptions::default();
    options.parse_doc_comments = true;
    let program = rigz_ast::parse(&str, options).expect("Failed to read input");
    let mut functions = vec![];
    for element in program.elements {
        if let Element::Statement(s) = element {
            match s {
                Statement::FunctionDefinition(f) => {
                    functions.push(FunctionDeclaration::Definition(f));
                }
                // Statement::ObjectDefinition(o) => {}
                // Statement::Module(module, def) => {
                //
                // }
                _ => {}
            }
        }
    }
    let docs = generate_docs("Program", &functions);
    std::fs::write("docs.md", docs)?;
    Ok(())
}