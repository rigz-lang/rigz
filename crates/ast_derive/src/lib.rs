mod derive_module;
mod derive_object;

extern crate proc_macro;

use crate::derive_module::DeriveModule;
use crate::derive_object::DeriveObject;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rigz_ast::FunctionSignature;
use rigz_core::derive::{rigz_type_to_rust_str, Tokens};
use rigz_core::RigzType;
use syn::{parse_macro_input, parse_str, Type};

/// Generate Module & ParsedModule implementations
/// Requires Rigz Trait Definition as input, `trait <Name> ... end`, creates struct <Name>Module and trait Rigz<Name>.
/// Rigz<Name> must be implemented manually
#[proc_macro]
pub fn derive_module(input: TokenStream) -> TokenStream {
    let full = parse_macro_input!(input as DeriveModule);
    quote! { #full }.into()
}

#[proc_macro]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let object = parse_macro_input!(input as DeriveObject);
    quote! { #object }.into()
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum FirstArg {
    None,
    VM,
    This,
    MutThis,
}

impl From<FirstArg> for Option<Ident> {
    fn from(val: FirstArg) -> Self {
        match val {
            FirstArg::None => None,
            FirstArg::VM => Some(Ident::new("self.vm", Span::call_site())),
            FirstArg::MutThis | FirstArg::This => Some(Ident::new("this", Span::call_site())),
        }
    }
}

impl From<FirstArg> for Option<Tokens> {
    fn from(val: FirstArg) -> Self {
        match val {
            FirstArg::None => None,
            FirstArg::VM => Some(quote! { vm }),
            FirstArg::MutThis => Some(quote! { this.borrow().rigz_type() }),
            FirstArg::This => Some(quote! { this.borrow().clone() }),
        }
    }
}

fn create_matched_call(name: &str, fs: Vec<&&FunctionSignature>, first_arg: FirstArg) -> Tokens {
    if fs.len() == 1 {
        let fs = fs.first().unwrap();
        return create_method_call(name, fs, first_arg);
    }

    let is_mut = FirstArg::MutThis == first_arg;
    let first_arg: Option<Tokens> = first_arg.into();
    let first_arg = first_arg.expect("Multi match not supported for non-extension functions");

    let mut has_any = false;
    let match_arms: Vec<_> = fs
        .iter()
        .map(|fs| match &fs.self_type {
            None => panic!("Matched call only supported for extension functions currently"),
            Some(ft) => {
                let v = if is_mut {
                    match convert_type_for_borrowed_arg(quote! { this }, &ft.rigz_type, true) {
                        None => Some(quote! { this.borrow_mut().deref_mut() }),
                        Some((s, _)) => Some(s),
                    }
                } else {
                    Some(quote! { v })
                };
                let base_call = base_call(name, fs, v, true);
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
                        if is_mut {
                            quote! {
                                RigzType::Bool => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Primitive(PrimitiveValue::Bool(v)) => {
                                    let v = v.to_bool();
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Int => {
                        if is_mut {
                            quote! {
                                RigzType::Int => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Primitive(PrimitiveValue::Number(n)) => {
                                    let v = n.to_int();
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Float => {
                        if is_mut {
                            quote! {
                                RigzType::Float => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Primitive(PrimitiveValue::Number(n)) => {
                                    let v = n.to_float();
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Number => {
                        if is_mut {
                            quote! {
                                RigzType::Number => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Primitive(PrimitiveValue::Number(v)) => {
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::String => {
                        if is_mut {
                            quote! {
                                RigzType::String => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Primitive(PrimitiveValue::String(v)) => {
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::List(_) => {
                        if is_mut {
                            quote! {
                                RigzType::List(_) => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::List(v) => {
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Set(_) => {
                        if is_mut {
                            quote! {
                                RigzType::Set(_) => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Set(v) => {
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Map(_, _) => {
                        if is_mut {
                            quote! {
                                RigzType::Map(_, _) => {
                                    #base_call
                                }
                            }
                        } else {
                            quote! {
                                ObjectValue::Map(v) => {
                                    #base_call
                                }
                            }
                        }
                    }
                    RigzType::Error => {
                        quote! {
                            ObjectValue::Primitive(PrimitiveValue::Error(v)) => {
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
            v => return Err(VMError::runtime(format!("Cannot call {function} on {v}"))),
        }
    };

    if is_mut {
        quote! {
            #name => {
                let rt = #first_arg;
                match rt {
                    #match_arms
                }
            }
        }
    } else {
        quote! {
            #name => match #first_arg {
                #match_arms
            }
        }
    }
}

fn convert_response(base_call: Tokens, function_signature: &FunctionSignature) -> Tokens {
    let (mut_result, is_vm) = match &function_signature.self_type {
        None => (false, false),
        Some(t) => (t.mutable, t.rigz_type.is_vm()),
    };

    if mut_result {
        if matches!(
            &function_signature.return_type.rigz_type,
            RigzType::None | RigzType::This
        ) {
            quote! {
                #base_call;
            }
        } else {
            quote! {
                return Ok(Some(#base_call.into()));
            }
        }
    } else {
        match &function_signature.return_type.rigz_type {
            RigzType::Error => {
                quote! {
                    Err(#base_call)
                }
            }
            RigzType::Tuple(v) => {
                let v = tuple_call(base_call, v, None);
                quote! {
                    #v
                }
            }
            t => {
                if let RigzType::Wrapper {
                    base_type,
                    optional,
                    can_return_error,
                } = t
                {
                    if let RigzType::Tuple(values) = base_type.as_ref() {
                        let args = tuple_args(values, None);
                        let call_args = tuple_call_args(values, None);
                        if *optional {
                            if *can_return_error {
                                quote! {
                                    match #base_call? {
                                        Some(v) => {
                                            let (#args) = v;
                                            Ok(ObjectValue::Tuple(vec![#call_args]))
                                        },
                                        None => Ok(ObjectValue::default()),
                                    }
                                }
                            } else {
                                quote! {
                                    match #base_call {
                                        Some(v) => {
                                            let (#args) = v;
                                            Ok(ObjectValue::Tuple(vec![#call_args]))
                                        },
                                        None => Ok(ObjectValue::default()),
                                    }
                                }
                            }
                        } else if *can_return_error {
                            quote! {
                                let (#args) = #base_call?;
                                Ok(ObjectValue::Tuple(vec![#call_args]))
                            }
                        } else {
                            let v = tuple_call(base_call, values, None);
                            quote! {
                                #v
                            }
                        }
                    } else if *optional {
                        if *can_return_error {
                            quote! {
                                match #base_call {
                                    Ok(Some(v)) => Ok(v.into()),
                                    Ok(None) => Ok(ObjectValue::default()),
                                    Err(e) => Err(e)
                                }
                            }
                        } else {
                            quote! {
                                Ok(#base_call.into())
                            }
                        }
                    } else if *can_return_error && base_type.as_ref() == &RigzType::None {
                        quote! {
                            #base_call?;
                            Ok(ObjectValue::default())
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
                } else {
                    quote! {
                        let result = #base_call;
                        Ok(result.into())
                    }
                }
            }
        }
    }
}

fn base_call(
    name: &str,
    function_signature: &FunctionSignature,
    first_arg: Option<Tokens>,
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
                match convert_type_for_borrowed_arg(first_arg.clone(), &ft.rigz_type, ft.mutable) {
                    None => {
                        quote! {
                            self.#method_name(#f, #fn_args)
                        }
                    }
                    Some((t, _)) => {
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

    let method_call = convert_response(base_call, function_signature);

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
                let (_, var_args) = function_signature.arguments.split_at(start);
                let mut call_var = quote! {};
                for v in var_args {
                    if v.function_type.rigz_type == RigzType::default() {
                        continue;
                    }
                    let name = Ident::new(v.name.as_str(), Span::call_site());
                    if let Some((v, _)) = convert_type_for_arg(
                        quote! { n },
                        &v.function_type.rigz_type,
                        v.function_type.mutable,
                    ) {
                        call_var = quote! {
                            #call_var
                            let #name = #name.into_iter().map(|n| #v).collect();
                        };
                    }
                }
                quote! {
                    let ([#(#args)*], [#(#var)*]) = args.var_args()?;
                    #call_var
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

fn rigz_type_to_arg(value: &RigzType, index: usize, offset: Option<usize>) -> Tokens {
    if let RigzType::Tuple(t) = value {
        tuple_args(t, Some(index))
    } else {
        let id = match value {
            RigzType::None => "none",
            RigzType::Never => panic!("Never cannot be used as argument type"),
            RigzType::Any
            | RigzType::Custom(_)
            | RigzType::Composite(_)
            | RigzType::Union(_)
            | RigzType::Wrapper { .. } => "any",
            RigzType::Bool => "bool",
            RigzType::Int => "int",
            RigzType::Float => "float",
            RigzType::Number => "number",
            RigzType::String => "string",
            RigzType::List(_) => "list",
            RigzType::Set(_) => "set",
            RigzType::Map(_, _) => "map",
            RigzType::Error => "error",
            RigzType::Range => "range",
            RigzType::Tuple(_) => unreachable!(),
            RigzType::This => "any",
            RigzType::Type => "rigz_type",
            RigzType::Function(_, _) => {
                todo!("Functions are not supported as module return values")
            }
            RigzType::Enum(_) => {
                todo!("Enums are not supported as module return values")
            }
        };
        let id = match offset {
            None => Ident::new(format!("{id}{index}").as_str(), Span::call_site()),
            Some(o) => Ident::new(format!("{id}_{o}_{index}").as_str(), Span::call_site()),
        };
        quote! { #id }
    }
}

fn tuple_args(values: &[RigzType], offset: Option<usize>) -> Tokens {
    let values = values
        .iter()
        .enumerate()
        .map(|(index, v)| rigz_type_to_arg(v, index, offset));

    quote! { #(#values, )* }
}

fn tuple_call_args(values: &[RigzType], offset: Option<usize>) -> Tokens {
    let values = values.iter().enumerate().map(|(index, v)| {
        if let RigzType::Tuple(t) = v {
            let v = tuple_call_args(t, offset);
            quote! { ObjectValue::Primitive(PrimitiveValue::Tuple(vec![#v])) }
        } else {
            let v = rigz_type_to_arg(v, index, offset);
            quote! {
                #v.into()
            }
        }
    });

    quote! { #(#values, )* }
}

fn tuple_call(base_call: Tokens, values: &[RigzType], offset: Option<usize>) -> Tokens {
    let args = tuple_args(values, offset);
    let call_args = tuple_call_args(values, offset);
    quote! {
        let (#args) = #base_call;
        Ok(ObjectValue::Tuple(vec![#call_args]))
    }
}

fn create_method_call(
    name: &str,
    function_signature: &FunctionSignature,
    first_arg: FirstArg,
) -> Tokens {
    let first_arg = match &function_signature.self_type {
        None => None,
        Some(v) => match first_arg {
            FirstArg::None => unreachable!(),
            FirstArg::VM => Some(quote! { self.vm }),
            FirstArg::MutThis => {
                let fs: Option<Ident> = first_arg.into();
                let fs = fs.unwrap();
                Some(quote! { #fs })
            }
            FirstArg::This => {
                if v.rigz_type == RigzType::Any {
                    first_arg.into()
                } else {
                    let fs: Option<Ident> = first_arg.into();
                    let fs = fs.unwrap();
                    Some(quote! { #fs })
                }
            }
        },
    };

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
        let name = Ident::new(&arg.name, Span::call_site());

        if arg.var_arg {
            var_args = Some(index);
            args.push(quote! {
                #name
            });
        } else if index == 0 {
            args.push(quote! {
                #name
            })
        } else {
            args.push(quote! {
                , #name
            });
        }

        if var_args.is_some() {
            continue;
        }

        let name = Ident::new(&arg.name, Span::call_site());
        let name = quote! { #name };
        match convert_type_for_borrowed_arg(
            name.clone(),
            &arg.function_type.rigz_type,
            arg.function_type.mutable,
        ) {
            None => call_args.push(quote! {
                let #name = #name.borrow().clone();
            }),
            Some((value, _)) => call_args.push(quote! {
                let #name = #value;
            }),
        }
    }

    (args, call_args, var_args)
}

fn convert_type_for_borrowed_arg(
    name: Tokens,
    rigz_type: &RigzType,
    mutable: bool,
) -> Option<(Tokens, bool)> {
    if rigz_type.is_vm() {
        return None;
    }

    if let RigzType::Tuple(tu) = rigz_type {
        let mut error = false;
        let tuple = tu
            .iter()
            .enumerate()
            .map(
                |(i, r)| match convert_type_for_borrowed_arg(quote! { #name.#i }, r, mutable) {
                    None => quote! { #name.#i },
                    Some((t, e)) => {
                        if e {
                            error = true;
                        }
                        t
                    }
                },
            )
            .collect::<Vec<_>>();
        return Some((quote! { (#(#tuple)*) }, error));
    }

    let t = if mutable {
        match &rigz_type {
            RigzType::Any | RigzType::Custom(_) => return None,
            RigzType::Wrapper {
                base_type,
                optional: false,
                can_return_error: false,
            } => return convert_type_for_arg(name, base_type, mutable),
            RigzType::Wrapper {
                base_type,
                optional: true,
                can_return_error: false,
            } => match convert_type_for_arg(name.clone(), base_type, true) {
                None => (
                    quote! { #name.borrow_mut().deref_mut().map_mut(|t| t) },
                    false,
                ),
                Some((t, e)) => {
                    if e {
                        (
                            quote! { #name.borrow_mut().deref_mut().maybe_map_mut(|#name| Ok(#t))? },
                            true,
                        )
                    } else {
                        (
                            quote! { #name.borrow_mut().deref_mut().map_mut(|#name| #t) },
                            false,
                        )
                    }
                }
            },
            RigzType::String => (quote! { #name.borrow_mut().as_string()? }, true),
            RigzType::Number => (quote! { #name.borrow_mut().as_number()? }, true),
            RigzType::Int => (quote! { #name.borrow_mut().as_int()? }, true),
            RigzType::Float => (quote! { #name.borrow_mut().as_float()? }, true),
            RigzType::Bool => (quote! { #name.borrow_mut().as_bool()? }, true),
            RigzType::List(_) => (quote! { #name.borrow_mut().as_list()? }, true),
            RigzType::Set(_) => (quote! { #name.borrow_mut().as_set()? }, true),
            RigzType::Map(_, _) => (quote! { #name.borrow_mut().as_map()? }, true),
            RigzType::Type => (quote! { #name.borrow_mut().rigz_type() }, false),
            r => todo!("borrowed call arg {r:?} is not supported"),
        }
    } else {
        match &rigz_type {
            RigzType::Any | RigzType::Custom(_) => return None,
            RigzType::Wrapper {
                base_type,
                optional: false,
                can_return_error: false,
            } => return convert_type_for_arg(name, base_type, false),
            RigzType::Wrapper {
                base_type,
                optional: true,
                can_return_error: false,
            } => match convert_type_for_arg(name.clone(), base_type, mutable) {
                None => (quote! { #name.borrow().deref().map(|t| t.clone()) }, false),
                Some((t, e)) => {
                    if e {
                        (
                            quote! { #name.borrow().deref().maybe_map(|#name| Ok(#t))? },
                            true,
                        )
                    } else {
                        (quote! { #name.borrow().deref().map(|#name| #t) }, false)
                    }
                }
            },
            RigzType::String => (quote! { #name.borrow().to_string() }, false),
            RigzType::Number => (quote! { #name.borrow().to_number()? }, true),
            RigzType::Int => (quote! { #name.borrow().to_int()? }, true),
            RigzType::Float => (quote! { #name.borrow().to_float()? }, true),
            RigzType::Bool => (quote! { #name.borrow().to_bool() }, false),
            RigzType::List(_) => (quote! { #name.borrow().to_list()? }, true),
            RigzType::Set(_) => (quote! { #name.borrow().to_set()? }, true),
            RigzType::Map(_, _) => (quote! { #name.borrow().to_map()? }, true),
            RigzType::Type => (quote! { #name.borrow().rigz_type() }, false),
            r => todo!("borrowed call arg {r:?} is not supported"),
        }
    };
    Some(t)
}

fn convert_type_for_arg(
    name: Tokens,
    rigz_type: &RigzType,
    mutable: bool,
) -> Option<(Tokens, bool)> {
    if rigz_type.is_vm() {
        return None;
    }

    if let RigzType::Tuple(tu) = rigz_type {
        let mut error = false;
        let tuple = tu
            .iter()
            .enumerate()
            .map(
                |(i, r)| match convert_type_for_arg(quote! { #name.#i }, r, mutable) {
                    None => quote! { #name.#i },
                    Some((t, e)) => {
                        if e {
                            error = true;
                        }
                        t
                    }
                },
            )
            .collect::<Vec<_>>();
        return Some((quote! { (#(#tuple)*) }, error));
    }

    let t = if mutable {
        match &rigz_type {
            RigzType::Any | RigzType::Custom(_) => return None,
            RigzType::Wrapper {
                base_type,
                optional: false,
                can_return_error: false,
            } => return convert_type_for_arg(name, base_type, mutable),
            RigzType::Wrapper {
                base_type,
                optional: true,
                can_return_error: false,
            } => match convert_type_for_arg(name.clone(), base_type, true) {
                None => (quote! { #name.borrow().deref().map(|t| t.clone()) }, false),
                Some((t, e)) => {
                    if e {
                        (quote! { #name.maybe_map_mut(|#name| Ok(#t))? }, true)
                    } else {
                        (quote! { #name.map_mut(|#name| #t) }, false)
                    }
                }
            },
            RigzType::String => (quote! { #name.as_string()? }, true),
            RigzType::Number => (quote! { #name.as_number()? }, true),
            RigzType::Int => (quote! { #name.as_int()? }, true),
            RigzType::Float => (quote! { #name.as_float()? }, true),
            RigzType::Bool => (quote! { #name.as_bool()? }, true),
            RigzType::List(_) => (quote! { #name.as_list()? }, true),
            RigzType::Set(_) => (quote! { #name.as_set()? }, true),
            RigzType::Map(_, _) => (quote! { #name.as_map()? }, true),
            RigzType::Type => (quote! { #name.rigz_type() }, false),
            r => todo!("call arg {r:?} is not supported"),
        }
    } else {
        match &rigz_type {
            RigzType::Any | RigzType::Custom(_) => return None,
            RigzType::Wrapper {
                base_type,
                optional: false,
                can_return_error: false,
            } => return convert_type_for_arg(name, base_type, false),
            RigzType::Wrapper {
                base_type,
                optional: true,
                can_return_error: false,
            } => match convert_type_for_arg(name.clone(), base_type, mutable) {
                None => return None,
                Some((t, e)) => {
                    if e {
                        (quote! { #name.maybe_map(|#name| Ok(#t))? }, true)
                    } else {
                        (quote! { #name.map(|#name| #t) }, false)
                    }
                }
            },
            RigzType::String => (quote! { #name.to_string() }, false),
            RigzType::Number => (quote! { #name.to_number()? }, true),
            RigzType::Int => (quote! { #name.to_int()? }, true),
            RigzType::Float => (quote! { #name.to_float()? }, true),
            RigzType::Bool => (quote! { #name.to_bool() }, false),
            RigzType::List(_) => (quote! { #name.to_list()? }, true),
            RigzType::Set(_) => (quote! { #name.to_set()? }, true),
            RigzType::Map(_, _) => (quote! { #name.to_map()? }, true),
            RigzType::Type => (quote! { #name.rigz_type() }, false),
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
                RigzType::Set(_) => "set".to_string(),
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

fn rigz_type_to_return_type(rigz_type: &RigzType) -> Option<Type> {
    if rigz_type == &RigzType::This {
        return None;
    }

    match rigz_type_to_rust_str(rigz_type) {
        None => None,
        Some(type_str) => parse_str::<Type>(&type_str).ok(),
    }
}
