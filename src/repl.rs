use clap::Args;
use rigz_core::{ObjectValue, PrimitiveValue, VMError};
use rigz_runtime::{Runtime, RuntimeError};
use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::DerefMut;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Args)]
pub struct ReplArgs {
    #[arg(short, long, default_value = "false", help = "Save History on exit")]
    save_history: bool,
}

struct RigzHelper<'r> {
    highlighter: RefCell<Highlighter>,
    config: &'r HighlightConfiguration,
}

impl rustyline::highlight::Highlighter for RigzHelper<'_> {
    // todo this could definitely be optimized
    #[allow(unused_variables)]
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let mut m = self.highlighter.borrow_mut();
        let s = highlight(m.deref_mut(), self.config, line.as_bytes());
        Cow::Owned(s)
    }

    #[allow(unused_variables)]
    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        true
    }
}

impl Completer for RigzHelper<'_> {
    type Candidate = String;
}

impl Hinter for RigzHelper<'_> {
    type Hint = String;
}

impl Validator for RigzHelper<'_> {}

impl Helper for RigzHelper<'_> {}

pub(crate) fn repl(args: ReplArgs) {
    let mut highlighter = Highlighter::new();
    let rigz_lang = tree_sitter_rigz::LANGUAGE;
    let rigz_lang = rigz_lang.into();
    let mut rigz_config = HighlightConfiguration::new(
        rigz_lang,
        "rigz",
        tree_sitter_rigz::HIGHLIGHTS_QUERY,
        tree_sitter_rigz::INJECTIONS_QUERY,
        tree_sitter_rigz::LOCALS_QUERY,
    )
    .unwrap();

    rigz_config.configure(&tree_sitter_rigz::NAMES);

    let rigz_helper = RigzHelper {
        highlighter: RefCell::new(Highlighter::new()),
        config: &rigz_config, // todo pass in runtime to auto complete identifiers and functions
    };

    let mut runtime = Runtime::new();
    let mut r = Editor::new().expect("Failed to create REPL");
    r.set_helper(Some(&rigz_helper));

    let mut needs_reset = false;
    let mut last_success = 0;
    loop {
        if needs_reset {
            let vm = runtime.vm_mut();
            let len = vm.scopes[vm.sp].instructions.len();
            for _ in last_success..len {
                vm.scopes[vm.sp].instructions.remove(last_success);
            }
            vm.frames.current.borrow_mut().pc = last_success;
            needs_reset = false;
        } else {
            let vm = runtime.vm();
            last_success = vm.frames.current.borrow().pc;
        };

        // todo add line numbers, runtime will need to keep track of them too for error messages
        let next = r.readline("> ").expect("Failed to read line");
        // todo listen for Ctrl+C, Up & Down arrows
        match next.trim() {
            "exit" => {
                if args.save_history {
                    let path = format!("{}.rigz", chrono::Utc::now());
                    println!("REPL history saved to {path}");
                    r.save_history(&path).expect("Failed to save history");
                }
                break;
            }
            "" => continue,
            next => {
                // currently eval will convert VMError into a runtime error
                match runtime.eval(next.to_string()) {
                    Ok(v) => {
                        highlight_value(&mut highlighter, &rigz_config, v);
                    }
                    Err(RuntimeError::Parse(p)) => {
                        eprintln!("\x1b[31mInvalid Input {p:?}\x1b[0m");
                    }
                    Err(RuntimeError::Validation(p)) => {
                        eprintln!("\x1b[31mValidation Failed {p:?}\x1b[0m");
                    }
                    Err(RuntimeError::Run(p)) => {
                        needs_reset = match p {
                            VMError::EmptyStack(_) => {
                                // imports and function definitions create an empty register
                                false
                            }
                            _ => {
                                eprintln!("\x1b[31mError: {p:?}\x1b[0m");
                                true
                            }
                        };
                    }
                };
            }
        };
    }
}

fn highlight_value(
    highlighter: &mut Highlighter,
    rigz_config: &HighlightConfiguration,
    value: ObjectValue,
) {
    let str = if matches!(value, ObjectValue::Primitive(PrimitiveValue::String(_))) {
        format!("'{value}'")
    } else {
        value.to_string()
    };
    println!("=> {}", highlight(highlighter, rigz_config, str.as_bytes()))
}

fn highlight(
    highlighter: &mut Highlighter,
    rigz_config: &HighlightConfiguration,
    bytes: &[u8],
) -> String {
    let mut current = None;
    let mut result = String::new();
    for event in highlighter
        .highlight(rigz_config, bytes, None, |_| None)
        .unwrap()
    {
        match event.unwrap() {
            HighlightEvent::Source { start, end } => {
                let str =
                    String::from_utf8(bytes[start..end].to_vec()).expect("Failed to read string");
                match current {
                    None => {
                        result.push_str(str.as_str());
                    }
                    Some(h) => {
                        let start = match h {
                            9 => "\x1b[38m",
                            1 => "\x1b[31m",
                            2 => "\x1b[32m",
                            3 => "\x1b[33m",
                            4..=6 => "\x1b[34m",
                            7 => "\x1b[35m",
                            8 => "\x1b[36m",
                            0 => "\x1b[37m",
                            _ => {
                                result.push_str(str.as_str());
                                continue;
                            }
                        };
                        result.push_str(format!("{start}{str}\x1b[0m").as_str())
                    }
                }
            }
            HighlightEvent::HighlightStart(s) => {
                let Highlight(h) = s;
                current = Some(h);
            }
            HighlightEvent::HighlightEnd => {
                current = None;
            }
        }
    }
    result
}
