use crate::{create_matched_call, method_name, rigz_type_to_return_type, FirstArg};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use rigz_ast::{generate_docs, FunctionDeclaration, FunctionSignature, ModuleTraitDefinition, Parser, ParserOptions};
use rigz_core::derive::{rigz_type_to_rust_str, Tokens};
use rigz_core::RigzType;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parse_str, LitStr, Token, Type};

pub(crate) struct DeriveModule {
    ident: Option<Ident>,
    dependencies: Vec<Ident>,
    literal: LitStr,
}

impl Parse for DeriveModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (ident, dependencies) = if input.peek(LitStr) {
            (None, vec![])
        } else {
            let ident = if input.peek(syn::Ident) {
                let i: Ident = input.parse()?;
                input.parse::<Token![,]>()?;
                Some(i)
            } else {
                None
            };

            let deps = if input.peek(syn::token::Bracket) {
                let content;
                _ = bracketed!(content in input);
                let parsed = content
                    .parse_terminated(Ident::parse, Token![,])?
                    .into_iter()
                    .collect();
                input.parse::<Token![,]>()?;
                parsed
            } else {
                vec![]
            };
            (ident, deps)
        };

        Ok(DeriveModule {
            ident,
            dependencies,
            literal: input.parse()?,
        })
    }
}

impl ToTokens for DeriveModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let input = self.literal.value();

        let input = &input;
        let mut parser = Parser::prepare(input, ParserOptions::default());
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
                    docs: _
                } => {
                    let method_name = method_name(name, fs);
                    // todo this is probably necessary
                    // let mutable_return = fs.return_type.mutable;
                    let definition = if fs.arguments.is_empty() && fs.self_type.is_none() {
                        match rigz_type_to_return_type(&fs.return_type.rigz_type, true) {
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
                                let ty =
                                    rigz_type_to_return_type(&a.function_type.rigz_type, false)
                                        .unwrap();
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
                                let ty = if t.rigz_type == RigzType::Any {
                                    "ObjectValue".to_string()
                                } else {
                                    rigz_type_to_rust_str(&t.rigz_type, false).unwrap()
                                };
                                let ty =
                                    parse_str::<Type>(ty.as_str()).expect("Failed to read type");
                                let first = if t.mutable {
                                    quote! {
                                        #name: &mut #ty,
                                    }
                                } else {
                                    quote! {
                                        #name: &#ty,
                                    }
                                };
                                let mut r = vec![first];
                                r.extend(args);
                                r
                            }
                        };
                        if is_vm {
                            match rigz_type_to_return_type(&fs.return_type.rigz_type, true) {
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
                            match rigz_type_to_return_type(&fs.return_type.rigz_type, true) {
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
                fn call(&self, function: &str, args: RigzArgs) -> Result<ObjectValue, VMError> {
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
                    this: Rc<RefCell<ObjectValue>>,
                    function: &str,
                    args: RigzArgs,
                ) -> Result<ObjectValue, VMError> {
                    match function {
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
                    function: &str,
                    args: RigzArgs,
                ) -> Result<Option<ObjectValue>, VMError> {
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

        let module_def = quote! {
            trait #module_trait {
                #(#methods)*
            }
        };

        tokens.extend(self.final_definition(module, module_methods, module_def, has_vm))
    }
}

impl DeriveModule {
    fn final_definition(
        &self,
        module: ModuleTraitDefinition,
        module_methods: Vec<Tokens>,
        module_def: Tokens,
        has_vm: bool,
    ) -> TokenStream {
        let name = &module.definition.name;
        let docs = generate_docs(name, &module.definition.functions);

        let module_name = match &self.ident {
            Some(id) => id.clone(),
            None => Ident::new(format!("{name}Module").as_str(), Span::call_site()),
        };

        let input = self.literal.value();
        let input = input.as_str();

        let lifetime_module = if has_vm {
            quote! { #module_name<'_> }
        } else {
            quote! { #module_name }
        };

        let deps = if self.dependencies.is_empty() {
            quote! {}
        } else {
            let d = self
                .dependencies
                .iter()
                .map(|i| quote! { rigz_core::Dependency::new::<#i>() });
            quote! {
                fn deps() -> Vec<rigz_core::Dependency> where Self: Sized {
                    vec![#(#d, )*]
                }
            }
        };

        let parsed_deps = if self.dependencies.is_empty() {
            quote! {}
        } else {
            let d = self
                .dependencies
                .iter()
                .map(|i| quote! { rigz_ast::ParsedDependency::new::<#i>() });
            quote! {
                fn parsed_dependencies() -> Vec<rigz_ast::ParsedDependency> where Self: Sized {
                    vec![#(#d, )*]
                }
            }
        };

        let base = quote! {
            #module_def

            impl Definition for #lifetime_module {
                #[inline]
                fn name() -> &'static str where Self: Sized{
                    #name
                }

                #[inline]
                fn trait_definition() -> &'static str where Self: Sized {
                    #input
                }
            }

            impl Module for #lifetime_module {
                #deps

                #(#module_methods)*
            }

            impl ParsedModule for #lifetime_module {
                #parsed_deps

                #[inline]
                fn module_definition() -> ModuleTraitDefinition where Self: Sized {
                    #module
                }
            }

            #[cfg(feature = "gen_docs")]
            impl GenDocs for #lifetime_module {
                fn generate_docs() -> &'static str where Self: Sized {
                    #docs
                }
            }
        };

        match self.ident {
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
    }
}
