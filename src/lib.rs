extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rigz_ast::{CustomType, Element, Expression, FunctionArgument, FunctionDeclaration, FunctionDefinition, FunctionSignature, FunctionType, Lifecycle, ModuleTraitDefinition, Number, Parser, RigzType, Statement, Value};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use syn::{parse_macro_input, parse_quote, parse_str, LitStr, ReturnType, Type};

type Tokens = proc_macro2::TokenStream;

// todo create derive_macro for ParsedModule that doesn't require implementing a custom trait, i.e. Module is implemented manually

/// Generate Module & ParsedModule implementations
/// Requires Rigz Trait Definition as input, `trait <Name> ... end`, creates struct <Name>Module and trait Rigz<Name>.
/// Rigz<Name> must be implemented manually
#[proc_macro]
pub fn derive_module(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr).value();

    let input = input.as_str();
    let mut parser = Parser::prepare(input).expect("Failed to setup parser");
    let module = parser
        .parse_module_trait_definition()
        .expect("Failed to parse input");

    let name = module.definition.name;

    let module_name = Ident::new(format!("{name}Module").as_str(), Span::call_site());
    let module_trait = Ident::new(format!("Rigz{name}").as_str(), Span::call_site());

    let mut methods = Vec::new();

    let mut needs_lifetime = false;

    let mut all_fcs: HashMap<&str, Vec<&FunctionSignature>> = HashMap::new();
    for func in &module.definition.functions {
        match func {
            FunctionDeclaration::Declaration {
                name,
                type_definition: fs,
            } => {
                let method_name = method_name(name, &fs);
                let mutable_return = fs.return_type.mutable;
                let definition = if fs.arguments.is_empty() && fs.self_type.is_none() {
                    match rigz_type_to_return_type(&fs.return_type.rigz_type) {
                        None => {
                            quote! {
                                fn #method_name(&self);
                            }
                        }
                        Some(rt) => {
                            quote! {
                                fn #method_name(&self) -> #rt;
                            }
                        }
                    }
                } else {
                    let mut var_arg = false;
                    let args: Vec<_> = fs
                        .arguments
                        .iter()
                        .map(|a| {
                            var_arg = var_arg || a.var_arg;
                            let name = Ident::new(a.name, Span::call_site());
                            let ty = rigz_type_to_return_type(&a.function_type.rigz_type).unwrap();
                            if var_arg {
                                quote! {
                                    #name: Vec<#ty>,
                                }
                            } else {
                                quote! {
                                    #name: #ty,
                                }
                            }
                        })
                        .collect();
                    let mut is_vm = false;
                    let args = match &fs.self_type {
                        None => args,
                        Some(t) if t.rigz_type.is_vm() && t.mutable => {
                            is_vm = true;
                            needs_lifetime = true;
                            args
                        }
                        Some(t) => {
                            let name = Ident::new("this", Span::call_site());
                            let ty = rigz_type_to_rust_str(&t.rigz_type).unwrap();
                            let ty = parse_str::<Type>(ty.as_str()).expect("Failed to read type");
                            let first = if t.mutable {
                                quote! {
                                    #name: &mut #ty,
                                }
                            } else {
                                quote! {
                                    #name: #ty,
                                }
                            };
                            let mut r = vec![first];
                            r.extend(args);
                            r
                        }
                    };
                    if is_vm {
                        match rigz_type_to_return_type(&fs.return_type.rigz_type) {
                            None => {
                                quote! {
                                    fn #method_name(&self, vm: &mut VM<'vm>, #(#args)*);
                                }
                            }
                            Some(rt) => {
                                quote! {
                                    fn #method_name(&self, vm: &mut VM<'vm>, #(#args)*) -> #rt;
                                }
                            }
                        }
                    } else {
                        match rigz_type_to_return_type(&fs.return_type.rigz_type) {
                            None => {
                                quote! {
                                    fn #method_name(&self, #(#args)*);
                                }
                            }
                            Some(rt) => {
                                quote! {
                                    fn #method_name(&self, #(#args)*) -> #rt;
                                }
                            }
                        }
                    }
                };
                methods.push(definition);
                match all_fcs.entry(name) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(fs);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![fs]);
                    }
                }
            }
            FunctionDeclaration::Definition(_) => {}
        }
    }

    let mut module_methods = Vec::new();

    // todo support polymorphic functions
    let calls: Vec<_> = all_fcs
        .iter()
        .map(|(name, f)| {
            (
                name,
                f.iter()
                    .filter(|fs| fs.self_type.is_none())
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, f)| !f.is_empty())
        .map(|(name, fs)| create_matched_call(name, fs, FirstArg::None))
        .collect();

    if !calls.is_empty() {
        module_methods.push(quote! {
            fn call(&self, function: &'vm str, args: RigzArgs) -> Result<Value, VMError> {
                match function {
                    #(#calls)*
                    _ => Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
            }
        });
    }

    let ext_calls: Vec<_> = all_fcs
        .iter()
        .map(|(name, f)| {
            (
                name,
                f.iter()
                    .filter(|fs| match &fs.self_type {
                        Some(f) if f.mutable => false,
                        Some(_) => true,
                        None => false,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, f)| !f.is_empty())
        .map(|(name, fs)| create_matched_call(name, fs, FirstArg::This))
        .collect();

    if !ext_calls.is_empty() {
        module_methods.push(quote! {
            fn call_extension(
                &self,
                this: Value,
                function: &'vm str,
                args: RigzArgs,
            ) -> Result<Value, VMError> {
                match function {
                    #(#ext_calls)*
                    _ => Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
            }
        });
    }

    let mut_ext_calls: Vec<_> = all_fcs
        .iter()
        .map(|(name, f)| {
            (
                name,
                f.iter()
                    .filter(|fs| match &fs.self_type {
                        Some(f) if f.mutable && !f.rigz_type.is_vm() => true,
                        Some(_) => false,
                        None => false,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, f)| !f.is_empty())
        .map(|(name, fs)| create_matched_call(name, fs, FirstArg::MutThis))
        .collect();

    if !mut_ext_calls.is_empty() {
        module_methods.push(quote! {
            fn call_mutable_extension(
                &self,
                this: &mut Value,
                function: &'vm str,
                args: RigzArgs,
            ) -> Result<Option<Value>, VMError> {
                match function {
                    #(#mut_ext_calls)*
                    _ => return Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
                Ok(None)
            }
        });
    }

    let vm_calls: Vec<_> = all_fcs
        .iter()
        .map(|(name, f)| {
            (
                name,
                f.iter()
                    .filter(|fs| match &fs.self_type {
                        Some(f) if f.mutable && f.rigz_type.is_vm() => true,
                        Some(_) => false,
                        None => false,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, f)| !f.is_empty())
        .map(|(name, fs)| create_matched_call(name, fs, FirstArg::VM))
        .collect();

    if !vm_calls.is_empty() {
        module_methods.push(quote! {
            fn vm_extension(
                &self,
                vm: &mut VM<'vm>,
                function: &'vm str,
                args: RigzArgs,
            ) -> Result<Value, VMError> {
                match function {
                    #(#vm_calls)*
                    _ => Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
            }
        });
    }

    let functions: Vec<_> = module
        .definition
        .functions
        .into_iter()
        .map(|f| match f {
            FunctionDeclaration::Declaration {
                name,
                type_definition,
            } => {
                let fs = parse_function_signature(type_definition);
                quote! {
                    FunctionDeclaration::Declaration {
                        name: #name,
                        type_definition: #fs
                    },
                }
            }
            FunctionDeclaration::Definition(fd) => {
                let FunctionDefinition {
                    name,
                    type_definition,
                    body,
                    lifecycle
                } = fd;
                let fs = parse_function_signature(type_definition);
                let elements: Vec<Tokens> = body
                    .elements
                    .into_iter()
                    .map(|element| match element {
                        Element::Expression(e) => {
                            let expr = match e {
                                Expression::This => quote! {
                                    Expression::This
                                },
                                _ => todo!(),
                            };
                            quote! {
                                Element::Expression(#expr),
                            }
                        }
                        _ => todo!(),
                    })
                    .collect();
                let lifecycle = match lifecycle {
                    None => quote! { None },
                    Some(l) => match l {
                        _ => todo!("lifecycle not supported"),
                        Lifecycle::Test(_) => quote! { Some(Lifecycle::Test(TestLifecycle)) },
                    }
                };
                quote! {
                    FunctionDeclaration::Definition(
                        FunctionDefinition {
                            name: #name,
                            type_definition: #fs,
                            body: Scope {
                                elements: vec![#(#elements)*]
                            },
                            lifecycle: #lifecycle
                        }
                    ),
                }
            }
        })
        .collect();

    let module_def = if needs_lifetime {
        quote! {
            trait #module_trait<'vm> {
                #(#methods)*
            }
        }
    } else {
        quote! {
            trait #module_trait {
                #(#methods)*
            }
        }
    };

    final_definition(
        input,
        module.auto_import,
        name,
        module_name,
        module_methods,
        functions,
        module_def,
    )
}

fn final_definition(
    input: &str,
    auto_import: bool,
    name: &str,
    module_name: Ident,
    module_methods: Vec<Tokens>,
    functions: Vec<Tokens>,
    module_def: Tokens,
) -> TokenStream {
    let tokens = quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #module_name {}

        #module_def

        impl <'vm> Module<'vm> for #module_name {
            #[inline]
            fn name(&self) -> &'static str {
                #name
            }

            #(#module_methods)*

            #[inline]
            fn trait_definition(&self) -> &'static str {
                #input
            }
        }

        impl <'a> ParsedModule<'a> for #module_name {
            #[inline]
            fn module_definition(&self) -> ModuleTraitDefinition<'static> {
                ModuleTraitDefinition {
                    auto_import: #auto_import,
                    definition: TraitDefinition {
                        name: #name,
                        functions: vec![#(#functions)*]
                    }
                }
            }
        }
    };
    tokens.into()
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum FirstArg {
    None,
    VM,
    This,
    MutThis,
}

impl Into<Option<Ident>> for FirstArg {
    fn into(self) -> Option<Ident> {
        match self {
            FirstArg::None => None,
            FirstArg::VM => Some(Ident::new("vm", Span::call_site())),
            FirstArg::MutThis | FirstArg::This => Some(Ident::new("this", Span::call_site())),
        }
    }
}

fn create_matched_call(name: &str, fs: Vec<&&FunctionSignature>, first_arg: FirstArg) -> Tokens {
    let first_arg = first_arg.into();

    if fs.len() == 1 {
        let fs = fs.first().unwrap();
        return create_method_call(name, fs, first_arg);
    }

    let first_arg = first_arg.expect("Multi match not supported for non-extension functions");

    let mut has_any = false;
    let match_arms: Vec<_> = fs
        .iter()
        .map(|fs| match &fs.self_type {
            None => panic!("Matched call only supported for extension functions currently"),
            Some(ft) => {
                let base_call = base_call(name, fs, Some(Ident::new("v", Span::call_site())), true);
                match &ft.rigz_type {
                    RigzType::Any => {
                        has_any = true;
                        quote! {
                            v => {
                                #base_call
                            }
                        }
                    }
                    RigzType::Bool => {
                        quote! {
                            Value::Bool(v) => {
                                #base_call
                            }
                        }
                    }
                    RigzType::Int => {
                        quote! {
                            Value::Number(n) => {
                                let v = n.to_int();
                                #base_call
                            }
                        }
                    }
                    RigzType::Float => {
                        quote! {
                            Value::Number(n) => {
                                let v = n.to_float();
                                #base_call
                            }
                        }
                    }
                    RigzType::Number => {
                        quote! {
                            Value::Number(v) => {
                                #base_call
                            }
                        }
                    }
                    RigzType::String => {
                        quote! {
                            Value::String(v) => {
                                #base_call
                            }
                        }
                    }
                    RigzType::List(_) => {
                        quote! {
                            Value::List(v) => {
                                #base_call
                            }
                        }
                    }
                    RigzType::Map(_, _) => {
                        quote! {
                            Value::Map(v) => {
                                #base_call
                            }
                        }
                    }
                    RigzType::Error => {
                        quote! {
                            Value::Error(v) => {
                                #base_call
                            }
                        }
                    }
                    r => todo!("Type not supported yet - {r}"),
                }
            }
        })
        .collect();

    let match_arms = if has_any {
        quote! {
            #(#match_arms)*
        }
    } else {
        quote! {
            #(#match_arms)*
            v => return Err(VMError::RuntimeError(format!("Cannot call {function} on {v}"))),
        }
    };

    quote! {
        #name => match #first_arg {
            #match_arms
        }
    }
}

fn base_call(
    name: &str,
    function_signature: &FunctionSignature,
    first_arg: Option<Ident>,
    matched: bool,
) -> Tokens {
    let method_name = method_name(name, function_signature);
    let (args, call_args, var_args) = setup_call_args(function_signature);
    let fn_args = match var_args {
        None => quote! { #(#args)* },
        Some(index) => {
            let (args, var) = args.split_at(index);
            if index == 0 {
                quote! { #(#var)* }
            } else {
                quote! { #(#args)*, #(#var)* }
            }
        }
    };
    let base_call = match first_arg {
        None => {
            quote! {
                self.#method_name(#fn_args)
            }
        }
        Some(first_arg) => match &function_signature.self_type {
            None => {
                quote! {
                    self.#method_name(#first_arg, #fn_args)
                }
            }
            Some(ft) if !matched => {
                let f = first_arg.clone();
                match convert_type_for_arg(first_arg.clone(), &ft.rigz_type, ft.mutable) {
                    None => {
                        quote! {
                            self.#method_name(#f, #fn_args)
                        }
                    }
                    Some(t) => {
                        quote! {
                            self.#method_name(#t, #fn_args)
                        }
                    }
                }
            }
            Some(_) => {
                quote! {
                    self.#method_name(#first_arg, #fn_args)
                }
            }
        },
    };

    let mut_result = match &function_signature.self_type {
        None => false,
        Some(t) => t.mutable && !t.rigz_type.is_vm(),
    };

    let method_call = if mut_result {
        quote! {
            #base_call;
        }
    } else {
        match &function_signature.return_type.rigz_type {
            RigzType::None => {
                quote! {
                    #base_call;
                    Ok(Value::None)
                }
            }
            RigzType::Any => {
                quote! {
                    Ok(#base_call)
                }
            }
            RigzType::Error => {
                quote! {
                    Err(#base_call)
                }
            }
            t => {
                if let RigzType::Type {
                    base_type,
                    optional,
                    can_return_error,
                } = t
                {
                    // todo optional logic is wrong
                    if *optional {
                        quote! {
                            match #base_call {
                                None => Ok(Value::None),
                                Some(s) => Ok(s.into())
                            }
                        }
                    } else {
                        if *can_return_error && base_type.as_ref() == &RigzType::None {
                            quote! {
                                #base_call?;
                                Ok(Value::None)
                            }
                        } else if *can_return_error {
                            if base_type.as_ref() != &RigzType::Any {
                                quote! {
                                    let result = #base_call?;
                                    Ok(result.into())
                                }
                            } else {
                                quote! {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                let result = #base_call;
                                Ok(result.into())
                            }
                        }
                    }
                } else {
                    quote! {
                        let result = #base_call;
                        Ok(result.into())
                    }
                }
            }
        }
    };

    let args = if args.is_empty() {
        quote! {}
    } else {
        match var_args {
            None => {
                quote! {
                    let [#(#args)*] = args.take()?;
                    #(#call_args)*
                }
            }
            Some(start) => {
                let (args, var) = args.split_at(start);
                quote! {
                    let ([#(#args)*], [#(#var)*]) = args.var_args()?;
                    #(#call_args)*
                }
            }
        }
    };

    quote! {
        #args
        #method_call
    }
}

fn create_method_call(
    name: &str,
    function_signature: &FunctionSignature,
    first_arg: Option<Ident>,
) -> Tokens {
    let base_call = base_call(name, function_signature, first_arg, false);

    quote! {
        #name => {
            #base_call
        }
    }
}

fn setup_call_args(
    function_signature: &FunctionSignature,
) -> (Vec<Tokens>, Vec<Tokens>, Option<usize>) {
    let mut args = Vec::with_capacity(function_signature.arguments.len());
    let mut call_args = Vec::with_capacity(function_signature.arguments.len());

    let mut var_args = None;
    for (index, arg) in function_signature.arguments.iter().enumerate() {
        let name = Ident::new(arg.name, Span::call_site());

        if arg.var_arg {
            var_args = Some(index);
            args.push(quote! {
                #name
            });
        } else {
            if index == 0 {
                args.push(quote! {
                    #name
                })
            } else {
                args.push(quote! {
                    , #name
                });
            }
        }

        if var_args.is_some() {
            continue;
        }

        let name = Ident::new(arg.name, Span::call_site());
        match convert_type_for_arg(
            name.clone(),
            &arg.function_type.rigz_type,
            arg.function_type.mutable,
        ) {
            None => {}
            Some(value) => call_args.push(quote! {
                let #name = #value;
            }),
        }
    }

    (args, call_args, var_args)
}

fn convert_type_for_arg(name: Ident, rigz_type: &RigzType, mutable: bool) -> Option<Tokens> {
    if rigz_type.is_vm() {
        return None;
    }

    let t = if mutable {
        match &rigz_type {
            RigzType::Any => return None,
            RigzType::String => quote! { #name.as_string() },
            RigzType::Number => quote! { #name.as_number()? },
            RigzType::Int => quote! { #name.as_int()? },
            RigzType::Float => quote! { #name.as_float()? },
            RigzType::Bool => quote! { #name.as_bool() },
            RigzType::List(_) => quote! { #name.as_list() },
            RigzType::Map(_, _) => quote! { #name.as_map() },
            RigzType::Type {
                base_type,
                optional,
                can_return_error,
            } => return None, // todo this will need to be improved
            r => todo!("call arg {r:?} is not supported"),
        }
    } else {
        match &rigz_type {
            RigzType::Any => return None,
            RigzType::String => quote! { #name.to_string() },
            RigzType::Number => quote! { #name.to_number()? },
            RigzType::Int => quote! { #name.to_int()? },
            RigzType::Float => quote! { #name.to_float()? },
            RigzType::Bool => quote! { #name.to_bool() },
            RigzType::List(_) => quote! { #name.to_list() },
            RigzType::Map(_, _) => quote! { #name.to_map() },
            RigzType::Type {
                base_type,
                optional,
                can_return_error,
            } => return None, // todo this will need to be improved
            r => todo!("call arg {r:?} is not supported"),
        }
    };
    Some(t)
}

fn method_name(name: &str, fs: &FunctionSignature) -> Ident {
    let method_name = match &fs.self_type {
        None => name.to_string(),
        Some(s) => {
            let type_name = match &s.rigz_type {
                RigzType::List(_) => "list".to_string(),
                RigzType::Map(_, _) => "map".to_string(),
                t => t.to_string().to_lowercase(),
            };
            if s.mutable {
                format!("mut_{type_name}_{name}")
            } else {
                format!("{type_name}_{name}")
            }
        }
    };
    Ident::new(method_name.as_str(), Span::call_site())
}

fn parse_function_signature(function_signature: FunctionSignature) -> Tokens {
    let FunctionSignature {
        arguments,
        return_type,
        self_type,
        positional,
    } = function_signature;
    let self_type = match self_type {
        None => quote! { None },
        Some(s) => {
            let ft = parse_function_type(s);
            quote! { Some(#ft) }
        }
    };
    let return_type = parse_function_type(return_type);
    let arguments: Vec<Tokens> = arguments
        .into_iter()
        .map(|arg| {
            let FunctionArgument {
                name,
                default,
                function_type,
                var_arg,
            } = arg;
            let function_type = parse_function_type(function_type);
            let default = match default {
                None => quote! { None },
                Some(s) => {
                    let v = match s {
                        Value::None => quote! { Value::None },
                        Value::Bool(b) => quote! { Value::Bool(#b) },
                        Value::Number(n) => match n {
                            Number::Int(i) => quote! { #i.into() },
                            Number::Float(f) => quote! { #f.into() },
                        },
                        Value::String(s) => quote! { Value::String(#s.into()) },
                        v => todo!("default not implemented for {v}"),
                    };
                    quote! { Some(#v) }
                }
            };
            quote! {
                FunctionArgument {
                    name: #name,
                    var_arg: #var_arg,
                    function_type: #function_type,
                    default: #default
                },
            }
        })
        .collect();
    quote! {
        FunctionSignature {
            arguments: vec![#(#arguments)*],
            positional: #positional,
            return_type: #return_type,
            self_type: #self_type
        }
    }
}

fn parse_function_type(function_type: FunctionType) -> Tokens {
    let FunctionType { rigz_type, mutable } = function_type;
    let rigz_type = rigz_type_to_tokens(rigz_type);
    quote! {
        FunctionType {
            rigz_type: #rigz_type,
            mutable: #mutable
        }
    }
}

fn rigz_type_to_tokens(rigz_type: RigzType) -> Tokens {
    match rigz_type {
        RigzType::None => quote! { RigzType::None },
        RigzType::Any => quote! { RigzType::Any },
        RigzType::Bool => quote! { RigzType::Bool },
        RigzType::Int => quote! { RigzType::Int },
        RigzType::Float => quote! { RigzType::Float },
        RigzType::Number => quote! { RigzType::Number },
        RigzType::String => quote! { RigzType::String },
        RigzType::Error => quote! { RigzType::Error },
        RigzType::This => quote! { RigzType::This },
        RigzType::Range => quote! { RigzType::Range },
        RigzType::List(t) => {
            let t = rigz_type_to_tokens(*t);
            quote! { RigzType::List(Box::new(#t)) }
        }
        RigzType::Map(k, v) => {
            let k = rigz_type_to_tokens(*k);
            let v = rigz_type_to_tokens(*v);
            quote! { RigzType::Map(Box::new(#k), Box::new(#v)) }
        }
        RigzType::Type {
            base_type,
            optional,
            can_return_error,
        } => {
            let base_type = rigz_type_to_tokens(*base_type);
            quote! {
                RigzType::Type {
                    base_type: Box::new(#base_type),
                    optional: #optional,
                    can_return_error: #can_return_error,
                }
            }
        }
        RigzType::Custom(c) => {
            let CustomType { name, fields } = c;
            let fields: Vec<_> = fields
                .into_iter()
                .map(|(name, ty)| {
                    let ty = rigz_type_to_tokens(ty);
                    quote! {
                        (#name, #ty),
                    }
                })
                .collect();
            quote! {
                RigzType::Custom(
                    CustomType {
                        name: #name.into(),
                        fields: vec![#(#fields)*]
                    }
                )
            }
        }
        RigzType::Function(args, ret) => {
            let ret = rigz_type_to_tokens(*ret);
            let args: Vec<_> = args
                .into_iter()
                .map(|arg| {
                    let arg = rigz_type_to_tokens(arg);
                    quote! { #arg, }
                })
                .collect();
            quote! {
                RigzType::Function(vec![#(#args)*], #ret)
            }
        }
    }
}

fn rigz_type_to_return_type(rigz_type: &RigzType) -> Option<Type> {
    if rigz_type == &RigzType::This {
        return None;
    }

    match rigz_type_to_rust_str(rigz_type) {
        None => None,
        Some(type_str) => parse_str::<Type>(&type_str).ok(),
    }
}

fn rigz_type_to_rust_str(rigz_type: &RigzType) -> Option<String> {
    let type_str = match rigz_type {
        RigzType::None => return None,
        RigzType::Bool => "bool".to_string(),
        RigzType::Int => "i64".to_string(),
        RigzType::Float => "f64".to_string(),
        RigzType::Any => "Value".to_string(),
        RigzType::Type {
            base_type,
            optional,
            can_return_error,
        } => match (base_type.as_ref(), optional, can_return_error) {
            (t, false, false) => return rigz_type_to_rust_str(t),
            (t, true, false) => match rigz_type_to_rust_str(t) {
                None => "Option<()>".to_string(),
                Some(t) => format!("Option<{t}>"),
            },
            (t, false, true) => match rigz_type_to_rust_str(t) {
                None => "Result<(), VMError>".to_string(),
                Some(t) => format!("Result<{t}, VMError>"),
            },
            (t, true, true) => match rigz_type_to_rust_str(t) {
                None => "Result<Option<()>, VMError>".to_string(),
                Some(t) => format!("Result<Option<{t}>, VMError>"),
            },
        },
        RigzType::Custom(_) => "Value".to_string(),
        RigzType::List(v) => {
            let v = rigz_type_to_rust_str(v.as_ref()).expect("None is not valid for list types");
            format!("Vec<{v}>")
        }
        RigzType::Map(k, v) => {
            let k = rigz_type_to_rust_str(k.as_ref()).expect("None is not valid for map key types");
            let v =
                rigz_type_to_rust_str(v.as_ref()).expect("None is not valid for map value types");
            format!("IndexMap<{k}, {v}>")
        }
        t => t.to_string(),
    };
    Some(type_str)
}
