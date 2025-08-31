use crate::{FunctionDeclaration, FunctionSignature, FunctionType};

fn function_type_docs(function_type: &FunctionType) -> String {
    format!("{}{}", if function_type.mutable { "mut " } else { "" }, function_type.rigz_type)
}

fn append_type_def(res: &mut String, name: &str, type_definition: &FunctionSignature, docs: &Option<String>) {
    res.push_str(format!("## {}\n", name).as_str());
    if let Some(d) = docs {
        res.push_str(format!("{}\n", d.replace("\n*", "\n")).as_str());
    }

    if !type_definition.arguments.is_empty() {
        res.push_str("\n### Parameters\n");
        if let Some(s) = &type_definition.self_type {
            res.push_str(format!("self: {}\n", function_type_docs(s)).as_str());
        }

        for arg in &type_definition.arguments {
            res.push_str(format!("{}: {}{}\n", arg.name, if arg.var_arg { "var " } else { "" }, function_type_docs(&arg.function_type)).as_str());
        }
    }

    res.push_str(format!("\nReturns {}", function_type_docs(&type_definition.return_type)).as_str());
}

pub fn generate_docs(name: &str, args: &[FunctionDeclaration]) -> String {
    let mut docs = format!("# {}\n\n", name);
    for func in args {
        match func {
            FunctionDeclaration::Definition(def) => {
                append_type_def(&mut docs, &def.name, &def.type_definition, &def.docs);
            }
            FunctionDeclaration::Declaration { name, type_definition, docs: d } => {
                append_type_def(&mut docs, name, type_definition, d);
            }
        }
        docs.push_str("\n\n");
    }
    docs
}