use crate::program::{ArgType, FunctionExpression, ImportValue, RigzArguments};
use crate::{
    Assign, Element, Exposed, Expression, FunctionArgument, FunctionDeclaration,
    FunctionDefinition, FunctionSignature, FunctionType, ModuleTraitDefinition, Scope, Statement,
    TraitDefinition,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rigz_vm::derive::{boxed, csv_tuple_vec, csv_vec, option};

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

impl ToTokens for FunctionExpression<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FunctionExpression::FunctionCall(name, args) => {
                quote! {
                    FunctionExpression::FunctionCall(#name, #args)
                }
            }
            FunctionExpression::TypeFunctionCall(ty, name, args) => {
                quote! {
                    FunctionExpression::TypeFunctionCall(#ty, #name, #args)
                }
            }
            FunctionExpression::InstanceFunctionCall(ex, calls, args) => {
                let ex = boxed(ex);
                quote! {
                    FunctionExpression::InstanceFunctionCall(#ex, vec![#(#calls)*], #args)
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
            Expression::Tuple(e) => {
                let values = csv_vec(e);
                quote! {
                    Expression::Tuple(#values)
                }
            }
            Expression::Map(m) => {
                let values: Vec<_> = m
                    .iter()
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
                let ex = boxed(ex);
                quote! {
                    Expression::UnaryExp(#op, #ex)
                }
            }
            Expression::Function(f) => {
                quote! {
                    Expression::Function(#f)
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
            Expression::Return(ret) => match ret {
                None => quote! { Expression::Return(None) },
                Some(b) => {
                    let b = boxed(b);
                    quote! { Expression::Return(Some(#b)) }
                }
            },
            Expression::Lambda {
                arguments,
                var_args_start,
                body,
            } => {
                let arguments = csv_vec(arguments);
                let body = boxed(body);
                let var_args_start = option(var_args_start);
                quote! {
                    Expression::Lambda {
                        arguments: #arguments,
                        var_args_start: #var_args_start,
                        body: #body
                    }
                }
            }
            Expression::ForList {
                var,
                expression,
                body,
            } => {
                let e = boxed(expression);
                let b = boxed(body);
                quote! {
                    Expression::ForList {
                        var: #var,
                        expression: #e,
                        body: #b,
                    }
                }
            }
            Expression::ForMap {
                k_var,
                v_var,
                expression,
                key,
                value,
            } => {
                let expression = boxed(expression);
                let key = boxed(key);
                let value = match value {
                    None => quote! { None },
                    Some(v) => {
                        let v = boxed(v);
                        quote! { Some(#v) }
                    }
                };
                quote! {
                    Expression::ForMap {
                        k_var: #k_var,
                        v_var: #v_var,
                        expression: #expression,
                        key: #key,
                        value: #value,
                    }
                }
            }
            Expression::Index(lhs, index) => {
                let lhs = boxed(lhs);
                let index = boxed(index);
                quote! {
                    Expression::Index(#lhs, #index)
                }
            }
            Expression::Error(err) => {
                let err = boxed(err);
                quote! {
                    Expression::Error(#err)
                }
            }
            Expression::Into { base, next } => {
                let base = boxed(base);
                quote! {
                    Expression::Into {
                        base: #base,
                        next: #next
                    }
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for RigzArguments<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            RigzArguments::Positional(v) => {
                let v = csv_vec(v);
                quote! {
                    RigzArguments::Positional(#v)
                }
            }
            RigzArguments::Mixed(a, n) => {
                let a = csv_vec(a);
                let n = csv_tuple_vec(n);
                quote! {
                    RigzArguments::Mixed(#a, #n)
                }
            }
            RigzArguments::Named(n) => {
                let n = csv_tuple_vec(n);
                quote! {
                    RigzArguments::Named(#n)
                }
            }
        };
        tokens.extend(t);
    }
}

impl ToTokens for Assign<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Assign::This => quote! { Assign::This },
            Assign::Identifier(name, mutable) => quote! { Assign::Identifier(#name, #mutable) },
            Assign::TypedIdentifier(n, mutable, rt) => {
                quote! { Assign::TypedIdentifier(#n, #mutable, #rt) }
            }
            Assign::Tuple(t) => {
                let values: Vec<_> = t
                    .iter()
                    .map(|(id, mutable)| quote! { (#id, #mutable), })
                    .collect();
                quote! { Assign::Tuple(vec![#(#values)*]) }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for ImportValue<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            ImportValue::TypeValue(s) => quote! {ImportValue::TypeValue(#s)},
            ImportValue::FilePath(s) => quote! {ImportValue::FilePath(#s)},
            ImportValue::UrlPath(s) => quote! {ImportValue::UrlPath(#s)},
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
            Statement::TypeDefinition(name, typ) => {
                quote! {
                    Statement::TypeDefinition(#name, #typ)
                }
            }
            Statement::TraitImpl {
                base_trait,
                concrete,
                definitions,
            } => {
                let definitions = csv_vec(definitions);
                quote! {
                    Statement::TraitImpl {
                        base_trait: #base_trait,
                        concrete: #concrete,
                        definition: #definitions
                    }
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
        let FunctionArgument {
            name,
            default,
            function_type,
            var_arg,
            rest,
        } = self;
        let d = option(default);
        tokens.extend(quote! {
            FunctionArgument {
                name: #name,
                default: #d,
                function_type: #function_type,
                var_arg: #var_arg,
                rest: #rest
            }
        })
    }
}

impl ToTokens for ArgType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            ArgType::Positional => quote! { ArgType::Positional },
            ArgType::List => quote! { ArgType::List },
            ArgType::Map => quote! { ArgType::Map },
        };
        tokens.extend(t)
    }
}

impl ToTokens for FunctionSignature<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionSignature {
            arguments,
            return_type,
            self_type,
            arg_type,
            var_args_start,
        } = self;
        let args = csv_vec(arguments);
        let s = option(self_type);
        let v = option(var_args_start);
        tokens.extend(quote! {
            FunctionSignature {
                arguments: #args,
                return_type: #return_type,
                self_type: #s,
                var_args_start: #v,
                arg_type: #arg_type
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
        let ModuleTraitDefinition {
            auto_import,
            definition,
        } = self;
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
