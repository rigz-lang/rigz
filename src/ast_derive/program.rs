use crate::{Assign, Element, Exposed, Expression, FunctionArgument, FunctionDeclaration, FunctionDefinition, FunctionSignature, FunctionType, ModuleTraitDefinition, Scope, Statement, TraitDefinition};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rigz_vm::derive::{boxed, csv_vec, option};

impl ToTokens for Element<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Element::Expression(e) => {
                quote! {
                    Element::Expression(#e)
                }
            }
            Element::Statement(s) => {
                quote! {
                    Element::Statement(#s)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Expression<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Expression::This => quote! {
                Expression::This
            },
            Expression::Value(v) => {
                quote! {
                    Expression::Value(#v)
                }
            }
            Expression::List(e) => {
                let values = csv_vec(e);
                quote! {
                    Expression::List(#values)
                }
            }
            Expression::Map(m) => {
                let values: Vec<_> = m
                    .into_iter()
                    .map(|(k, v)| {
                        quote! { (#k, #v), }
                    })
                    .collect();
                quote! {
                    Expression::Map(vec![#(#values)*])
                }
            }
            Expression::Identifier(i) => {
                quote! {
                    Expression::Identifier(#i)
                }
            }
            Expression::BinExp(lhs, op, rhs) => {
                quote! {
                    Expression::BinExp(Box::new(#lhs), #op, Box::new(#rhs))
                }
            }
            Expression::UnaryExp(op, ex) => {
                quote! {
                    Expression::UnaryExp(#op, #ex)
                }
            }
            Expression::FunctionCall(name, args) => {
                let args = csv_vec(args);
                quote! {
                    Expression::FunctionCall(#name, #args)
                }
            }
            Expression::TypeFunctionCall(ty, name, args) => {
                let args = csv_vec(args);
                quote! {
                    Expression::TypeFunctionCall(#ty, #name, #args)
                }
            }
            Expression::InstanceFunctionCall(ex, calls, args) => {
                let args = csv_vec(args);
                quote! {
                    Expression::InstanceFunctionCall(#ex, vec![#(#calls)*], #args)
                }
            }
            Expression::Scope(s) => {
                quote! {
                    Expression::Scope(#s)
                }
            }
            Expression::Cast(e, t) => {
                let e = boxed(e);
                quote! {
                    Expression::Cast(#e, #t)
                }
            }
            Expression::Symbol(s) => quote! {
                Expression::Symbol(#s)
            },
            Expression::If {
                condition,
                then,
                branch,
            } => {
                let c = boxed(condition);
                let b = option(branch);
                quote! {
                    Expression::If {
                        condition: #c,
                        then: #then,
                        branch: #b
                    }
                }
            }
            Expression::Unless { condition, then } => {
                let c = boxed(condition);
                quote! {
                    Expression::Unless {
                        condition: #c,
                        then: #then
                    }
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Assign<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Assign::This => quote! { Assign::This },
            Assign::Identifier(name, mutable) => quote! { Assign::Identifier(#name, #mutable) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for Exposed<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Exposed::TypeValue(tv) => quote! { Exposed::TypeValue(#tv) },
            Exposed::Identifier(id) => quote! { Exposed::Identifier(#id) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for Scope<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Scope { elements } = self;
        let elements = csv_vec(elements);
        tokens.extend(quote! {
            Scope {
                elements: #elements
            }
        })
    }
}

impl ToTokens for Statement<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Statement::Assignment { lhs, expression } => {
                quote! {
                    Statement::Assignment {
                        lhs: #lhs,
                        expression: #expression
                    }
                }
            }
            Statement::BinaryAssignment {
                lhs,
                op,
                expression,
            } => {
                quote! {
                    Statement::BinaryAssignment {
                        lhs = #lhs,
                        op = #op,
                        expression = #expression
                    }
                }
            }
            Statement::FunctionDefinition(fd) => {
                quote! {
                    Statement::FunctionDefinition(#fd)
                }
            }
            Statement::Trait(tr) => {
                quote! {
                    Statement::Trait(#tr)
                }
            }
            Statement::Import(im) => {
                quote! {
                    Statement::Import(#im)
                }
            }
            Statement::Export(ex) => {
                quote! {
                    Statement::Export(#ex)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for FunctionDefinition<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionDefinition {
            name,
            type_definition,
            body,
            lifecycle,
        } = self;
        let l = option(lifecycle);
        tokens.extend(quote! {
            FunctionDefinition {
                name: #name,
                lifecycle: #l,
                type_definition: #type_definition,
                body: #body
            }
        })
    }
}

impl ToTokens for FunctionType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let m = self.mutable;
        let r = &self.rigz_type;
        tokens.extend(quote! {
            FunctionType {
                mutable: #m,
                rigz_type: #r
            }
        })
    }
}

impl ToTokens for FunctionArgument<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionArgument { name, default, function_type, var_arg } = self;
        let d = option(default);
        tokens.extend(quote! {
            FunctionArgument {
                name: #name,
                default: #d,
                function_type: #function_type,
                var_arg: #var_arg
            }
        })
    }
}

impl ToTokens for FunctionSignature<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionSignature { arguments, return_type, self_type, positional } = self;
        let args = csv_vec(arguments);
        let s = option(self_type);
        tokens.extend(quote! {
            FunctionSignature {
                arguments: #args,
                return_type: #return_type,
                self_type: #s,
                positional: #positional
            }
        })
    }
}

impl ToTokens for FunctionDeclaration<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FunctionDeclaration::Declaration {
                name,
                type_definition,
            } => {
                quote! {
                    FunctionDeclaration::Declaration {
                        name: #name,
                        type_definition: #type_definition
                    }
                }
            }
            FunctionDeclaration::Definition(fd) => {
                quote! {
                    FunctionDeclaration::Definition(#fd)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for ModuleTraitDefinition<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ModuleTraitDefinition { auto_import, definition } = self;
        tokens.extend(quote! {
            ModuleTraitDefinition {
                auto_import: #auto_import,
                definition: #definition
            }
        })
    }
}

impl ToTokens for TraitDefinition<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TraitDefinition { name, functions } = self;
        let functions = csv_vec(functions);
        tokens.extend(quote! {
            TraitDefinition {
                name: #name,
                functions: #functions
            }
        })
    }
}
