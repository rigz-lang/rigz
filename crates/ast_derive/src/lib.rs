extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rigz_ast::{FunctionDeclaration, FunctionSignature, ModuleTraitDefinition, Parser, RigzType};
use rigz_core::derive::{rigz_type_to_rust_str, Tokens};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_str, LitStr, Token, Type};

struct DeriveModule {
    ident: Option<Ident>,
    literal: LitStr,
}

impl Parse for DeriveModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(DeriveModule {
                ident: None,
                literal: input.parse()?,
            })
        } else {
            let ident = Some(input.parse()?);
            input.parse::<Token![,]>()?;
            Ok(DeriveModule {
                ident,
                literal: input.parse()?,
            })
        }
    }
}

/// Generate Module & ParsedModule implementations
/// Requires Rigz Trait Definition as input, `trait <Name> ... end`, creates struct <Name>Module and trait Rigz<Name>.
/// Rigz<Name> must be implemented manually
#[proc_macro]
pub fn derive_module(input: TokenStream) -> TokenStream {
    let full = parse_macro_input!(input as DeriveModule);
    let input = full.literal.value();

    let input = &input;
    let mut parser = Parser::prepare(input, false).expect("Failed to setup parser");
    let module = parser
        .parse_module_trait_definition()
        .expect("Failed to parse input");

    let name = module.definition.name.as_str();

    let module_trait = Ident::new(format!("Rigz{name}").as_str(), Span::call_site());

    let mut methods = Vec::new();

    let mut all_fcs: HashMap<&str, Vec<&FunctionSignature>> = HashMap::new();
    for func in &module.definition.functions {
        match func {
            FunctionDeclaration::Declaration {
                name,
                type_definition: fs,
            } => {
                let method_name = method_name(name, fs);
                // todo this is probably necessary
                // let mutable_return = fs.return_type.mutable;
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
                            let name = Ident::new(&a.name, Span::call_site());
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
                                    fn #method_name(&self, vm: &mut VM, #(#args)*);
                                }
                            }
                            Some(rt) => {
                                quote! {
                                    fn #method_name(&self, vm: &mut VM, #(#args)*) -> #rt;
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
            // nothing needed for Definitions here
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
            fn call(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
                match function.as_str() {
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
                this: Rc<RefCell<ObjectValue>>,
                function: String,
                args: RigzArgs,
            ) -> Result<ObjectValue, VMError> {
                match function.as_str() {
                    #(#ext_calls)*
                    _ => Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
            }
        });
    }

    let mut mut_ext_calls: Vec<_> = all_fcs
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

    let has_vm = !vm_calls.is_empty();
    if has_vm {
        mut_ext_calls.extend(vm_calls);
    }

    if !mut_ext_calls.is_empty() {
        module_methods.push(quote! {
            fn call_mutable_extension(
                &self,
                this: Rc<RefCell<ObjectValue>>,
                function: String,
                args: RigzArgs,
            ) -> Result<Option<ObjectValue>, VMError> {
                match function.as_str() {
                    #(#mut_ext_calls)*
                    _ => return Err(VMError::InvalidModuleFunction(format!(
                        "Function {function} does not exist"
                    )))
                }
                Ok(None)
            }
        });
    }

    let module_def = quote! {
        trait #module_trait {
            #(#methods)*
        }
    };

    final_definition(full, module, module_methods, module_def, has_vm)
}

#[proc_macro]
pub fn derive_object(input: TokenStream) -> TokenStream {
    todo!()
}

fn final_definition(
    full: DeriveModule,
    module: ModuleTraitDefinition,
    module_methods: Vec<Tokens>,
    module_def: Tokens,
    has_vm: bool,
) -> TokenStream {
    let name = &module.definition.name;
    let module_name = match &full.ident {
        Some(id) => id.clone(),
        None => Ident::new(format!("{name}Module").as_str(), Span::call_site()),
    };

    let input = full.literal.value();
    let input = input.as_str();

    let lifetime_module = if has_vm {
        quote! { #module_name<'_> }
    } else {
        quote! { #module_name }
    };

    let base = quote! {
        #module_def

        impl Definition for #lifetime_module {
            #[inline]
            fn name(&self) -> &'static str {
                #name
            }

            #[inline]
            fn trait_definition(&self) -> &'static str {
                #input
            }
        }

        impl Module for #lifetime_module {
            #(#module_methods)*
        }

        impl ParsedModule for #lifetime_module {
            #[inline]
            fn module_definition(&self) -> ModuleTraitDefinition {
                #module
            }
        }
    };

    match full.ident {
        None => {
            let struct_def = if has_vm {
                quote! {
                    pub struct #module_name<'v> {
                        vm: &'v mut VM
                    }
                }
            } else {
                quote! {
                    #[derive(Debug)]
                    pub struct #module_name;
                }
            };

            quote! {
                #struct_def

                #base
            }
        }
        Some(_) => base,
    }
    .into()
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
                        Some(s) => Some(s),
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
            v => return Err(VMError::RuntimeError(format!("Cannot call {function} on {v}"))),
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

    let (mut_result, is_vm) = match &function_signature.self_type {
        None => (false, false),
        Some(t) => (t.mutable, t.rigz_type.is_vm()),
    };

    let method_call = if mut_result {
        quote! {
            #base_call;
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
                                    Ok(Some(v)) => Ok(v),
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
                let (_, var_args) = function_signature.arguments.split_at(start);
                let mut call_var = quote! {};
                for v in var_args {
                    if v.function_type.rigz_type == RigzType::default() {
                        continue;
                    }
                    let name = Ident::new(v.name.as_str(), Span::call_site());
                    if let Some(v) = convert_type_for_arg(
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
            RigzType::Map(_, _) => "map",
            RigzType::Error => "error",
            RigzType::Range => "range",
            RigzType::Tuple(_) => unreachable!(),
            RigzType::This => "any",
            RigzType::Type => "rigz_type",
            RigzType::Function(_, _) => {
                todo!("Functions are not supported as module return values")
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
            Some(value) => call_args.push(quote! {
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
) -> Option<Tokens> {
    if rigz_type.is_vm() {
        return None;
    }

    if let RigzType::Tuple(tu) = rigz_type {
        let tuple = tu
            .iter()
            .enumerate()
            .map(
                |(i, r)| match convert_type_for_borrowed_arg(quote! { #name.#i }, r, mutable) {
                    None => quote! { #name.#i },
                    Some(t) => t,
                },
            )
            .collect::<Vec<_>>();
        return Some(quote! { (#(#tuple)*) });
    }

    let t = if mutable {
        match &rigz_type {
            RigzType::Any => return None,
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
                None => quote! { #name.borrow_mut().deref_mut().map_mut(|t| t) },
                Some(t) => quote! { #name.borrow_mut().deref_mut().map_mut(|#name| #t) },
            },
            RigzType::String => quote! { #name.borrow_mut().as_string()? },
            RigzType::Number => quote! { #name.borrow_mut().as_number()? },
            RigzType::Int => quote! { #name.borrow_mut().as_int()? },
            RigzType::Float => quote! { #name.borrow_mut().as_float()? },
            RigzType::Bool => quote! { #name.borrow_mut().as_bool()? },
            RigzType::List(_) => quote! { #name.borrow_mut().as_list()? },
            RigzType::Map(_, _) => quote! { #name.borrow_mut().as_map()? },
            RigzType::Type => quote! { #name.borrow_mut().rigz_type() },
            r => todo!("call arg {r:?} is not supported"),
        }
    } else {
        match &rigz_type {
            RigzType::Any => return None,
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
                None => quote! { #name.borrow().deref().map(|t| t).cloned() },
                Some(t) => quote! { #name.borrow().deref().map(|#name| #t) },
            },
            RigzType::String => quote! { #name.borrow().to_string() },
            RigzType::Number => quote! { #name.borrow().to_number()? },
            RigzType::Int => quote! { #name.borrow().to_int()? },
            RigzType::Float => quote! { #name.borrow().to_float()? },
            RigzType::Bool => quote! { #name.borrow().to_bool() },
            RigzType::List(_) => quote! { #name.borrow().to_list()? },
            RigzType::Map(_, _) => quote! { #name.borrow().to_map()? },
            RigzType::Type => quote! { #name.borrow().rigz_type() },
            r => todo!("call arg {r:?} is not supported"),
        }
    };
    Some(t)
}

fn convert_type_for_arg(name: Tokens, rigz_type: &RigzType, mutable: bool) -> Option<Tokens> {
    if rigz_type.is_vm() {
        return None;
    }

    if let RigzType::Tuple(tu) = rigz_type {
        let tuple = tu
            .iter()
            .enumerate()
            .map(
                |(i, r)| match convert_type_for_arg(quote! { #name.#i }, r, mutable) {
                    None => quote! { #name.#i },
                    Some(t) => t,
                },
            )
            .collect::<Vec<_>>();
        return Some(quote! { (#(#tuple)*) });
    }

    let t = if mutable {
        match &rigz_type {
            RigzType::Any => return None,
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
                None => return None,
                Some(t) => quote! { #name.map_mut(|#name| #t) },
            },
            RigzType::String => quote! { #name.as_string()? },
            RigzType::Number => quote! { #name.as_number()? },
            RigzType::Int => quote! { #name.as_int()? },
            RigzType::Float => quote! { #name.as_float()? },
            RigzType::Bool => quote! { #name.as_bool()? },
            RigzType::List(_) => quote! { #name.as_list()? },
            RigzType::Map(_, _) => quote! { #name.as_map()? },
            RigzType::Type => quote! { #name.rigz_type() },
            r => todo!("call arg {r:?} is not supported"),
        }
    } else {
        match &rigz_type {
            RigzType::Any => return None,
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
                Some(t) => quote! { #name.map(|#name| #t) },
            },
            RigzType::String => quote! { #name.to_string() },
            RigzType::Number => quote! { #name.to_number()? },
            RigzType::Int => quote! { #name.to_int()? },
            RigzType::Float => quote! { #name.to_float()? },
            RigzType::Bool => quote! { #name.to_bool() },
            RigzType::List(_) => quote! { #name.to_list()? },
            RigzType::Map(_, _) => quote! { #name.to_map()? },
            RigzType::Type => quote! { #name.rigz_type() },
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

fn rigz_type_to_return_type(rigz_type: &RigzType) -> Option<Type> {
    if rigz_type == &RigzType::This {
        return None;
    }

    match rigz_type_to_rust_str(rigz_type) {
        None => None,
        Some(type_str) => parse_str::<Type>(&type_str).ok(),
    }
}
