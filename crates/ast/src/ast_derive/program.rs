use crate::program::{
    ArgType, AssignIndex, Constructor, FunctionExpression, ImportValue, ObjectAttr,
    ObjectDefinition, RigzArguments,
};
use crate::{
    Assign, Each, Element, Exposed, Expression, FunctionArgument, FunctionDeclaration,
    FunctionDefinition, FunctionSignature, FunctionType, MatchVariant, MatchVariantCondition,
    MatchVariantVariable, ModuleTraitDefinition, Scope, Statement, TraitDefinition,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rigz_core::derive::{boxed, csv_vec, option};

impl ToTokens for Element {
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

impl ToTokens for FunctionExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FunctionExpression::FunctionCall(name, args) => {
                quote! {
                    FunctionExpression::FunctionCall(#name.to_string(), #args)
                }
            }
            FunctionExpression::TypeFunctionCall(ty, name, args) => {
                quote! {
                    FunctionExpression::TypeFunctionCall(#ty, #name.to_string(), #args)
                }
            }
            FunctionExpression::TypeConstructor(ty, args) => {
                quote! {
                    FunctionExpression::TypeConstructor(#ty, #args)
                }
            }
            FunctionExpression::InstanceFunctionCall(ex, calls, args) => {
                let ex = boxed(ex);
                quote! {
                    FunctionExpression::InstanceFunctionCall(#ex, vec![#(#calls.to_string())*], #args)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Expression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Expression::This => quote! {
                Expression::This
            },
            Expression::Break => quote! {
                Expression::Break
            },
            Expression::Next => quote! {
                Expression::Next
            },
            Expression::Enum(t, s, exp) => {
                let exp = match exp {
                    None => quote! { None },
                    Some(e) => {
                        let b = boxed(e);
                        quote! { Some(#b) }
                    }
                };
                quote! {
                    Expression::Enum(#t.to_string(), #s.to_string(), #exp)
                }
            }
            Expression::Match {
                condition,
                variants,
            } => {
                let cond = boxed(condition);
                let var = csv_vec(variants);
                quote! {
                    Expression::Match {
                        condition: #cond,
                        variants: #var
                    }
                }
            }
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
            Expression::Set(e) => {
                let values = csv_vec(e);
                quote! {
                    Expression::Set(#values)
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
                    Expression::Identifier(#i.to_string())
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
                Expression::Symbol(#s.to_string())
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
            Expression::Ternary {
                condition,
                then,
                branch,
            } => {
                let c = boxed(condition);
                let then = boxed(then);
                let branch = boxed(branch);
                quote! {
                    Expression::Ternary {
                        condition: #c,
                        then: #then
                        branch: #branch
                    }
                }
            }
            Expression::IfGuard { condition, then } => {
                let c = boxed(condition);
                let then = boxed(then);
                quote! {
                    Expression::IfGuard {
                        condition: #c,
                        then: #then
                    }
                }
            }
            Expression::UnlessGuard { condition, then } => {
                let c = boxed(condition);
                let then = boxed(then);
                quote! {
                    Expression::UnlessGuard {
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
            Expression::Exit(ret) => match ret {
                None => quote! { Expression::Exit(None) },
                Some(b) => {
                    let b = boxed(b);
                    quote! { Expression::Exit(Some(#b)) }
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
                        var: #var.to_string(),
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
                        k_var: #k_var.to_string(),
                        v_var: #v_var.to_string(),
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
            Expression::DoubleBang(b) => {
                let b = boxed(b);
                quote! {
                    Expression::DoubleBang(#b)
                }
            }
            Expression::Try(b) => {
                let b = boxed(b);
                quote! {
                    Expression::Try(#b)
                }
            }
            Expression::Catch { base, var, catch } => {
                let b = boxed(base);
                let v = option(var);
                quote! {
                    Expression::Catch {
                        base: #b,
                        var: #v.map(|s| s.to_string()),
                        catch: #catch
                    }
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for MatchVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            MatchVariant::Enum {
                name,
                condition,
                body,
                variables,
            } => {
                let var = csv_vec(variables);
                quote! {
                    MatchVariant::Enum {
                        name: #name.to_string(),
                        condition: #condition,
                        body: #body,
                        variables: #var
                    }
                }
            }
            MatchVariant::Else(s) => quote! {
                MatchVariant::Else(#s)
            },
        };
        tokens.extend(t);
    }
}

impl ToTokens for MatchVariantCondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            MatchVariantCondition::None => quote! { MatchVariantCondition::None },
            MatchVariantCondition::If(ex) => quote! {
                MatchVariantCondition::If(#ex)
            },
            MatchVariantCondition::Unless(ex) => quote! {
                MatchVariantCondition::Unless(#ex)
            },
        };
        tokens.extend(t);
    }
}

impl ToTokens for MatchVariantVariable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            MatchVariantVariable::Identifier(id) => quote! {
                MatchVariantVariable::Identifier(#id.to_string())
            },
            MatchVariantVariable::Value(v) => quote! {
                MatchVariantVariable::Value(#v)
            },
        };
        tokens.extend(t)
    }
}

impl ToTokens for RigzArguments {
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
                let values: Vec<_> = n
                    .iter()
                    .map(|(a, v)| quote! { (#a.to_string(), #v), })
                    .collect();
                let n = quote! { vec![#(#values)*] };
                quote! {
                    RigzArguments::Mixed(#a, #n)
                }
            }
            RigzArguments::Named(n) => {
                let values: Vec<_> = n
                    .iter()
                    .map(|(a, v)| quote! { (#a.to_string(), #v), })
                    .collect();
                let n = quote! { vec![#(#values)*] };
                quote! {
                    RigzArguments::Named(#n)
                }
            }
        };
        tokens.extend(t);
    }
}

impl ToTokens for Each {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Each::Identifier {
                name,
                mutable,
                shadow,
            } => {
                quote! { Each::Identifier { name: #name.to_string(), mutable: #mutable, shadow: #shadow } }
            }
            Each::TypedIdentifier {
                name,
                mutable,
                shadow,
                rigz_type,
            } => {
                quote! { Each::TypedIdentifier { name: #name.to_string(), mutable: #mutable, shadow: #shadow, rigz_type: #rigz_type} }
            }
            Each::Tuple(t) => {
                let values: Vec<_> = t
                    .iter()
                    .map(|(id, mutable, shadow)| quote! { (#id.to_string(), #mutable, #shadow), })
                    .collect();
                quote! { Each::Tuple(vec![#(#values)*]) }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Assign {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Assign::This => quote! { Assign::This },
            Assign::Identifier {
                name,
                mutable,
                shadow,
            } => {
                quote! { Assign::Identifier { name: #name.to_string(), mutable: #mutable, shadow: #shadow } }
            }
            Assign::TypedIdentifier {
                name,
                mutable,
                shadow,
                rigz_type,
            } => {
                quote! { Assign::TypedIdentifier { name: #name.to_string(), mutable: #mutable, shadow: #shadow, rigz_type: #rigz_type} }
            }
            Assign::Tuple(t) => {
                let values: Vec<_> = t
                    .iter()
                    .map(|(id, mutable, shadow)| quote! { (#id.to_string(), #mutable, #shadow), })
                    .collect();
                quote! { Assign::Tuple(vec![#(#values)*]) }
            }
            Assign::InstanceSet(base, calls) => {
                let c = csv_vec(calls);
                quote! { Assign::InstanceSet(#base, #c) }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for AssignIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            AssignIndex::Identifier(i) => quote! { AssignIndex::Identifier(#i.to_string()) },
            AssignIndex::Index(s) => quote! { AssignIndex::Index(#s) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for ImportValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            ImportValue::TypeValue(s) => quote! {ImportValue::TypeValue(#s.to_string())},
            ImportValue::FilePath(s) => quote! {ImportValue::FilePath(#s.to_string())},
            ImportValue::UrlPath(s) => quote! {ImportValue::UrlPath(#s.to_string())},
        };
        tokens.extend(t)
    }
}

impl ToTokens for Exposed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Exposed::TypeValue(tv) => quote! { Exposed::TypeValue(#tv.to_string()) },
            Exposed::Identifier(id) => quote! { Exposed::Identifier(#id.to_string()) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for Scope {
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

impl ToTokens for Statement {
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
                        lhs: #lhs,
                        op: #op,
                        expression: #expression
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
                    Statement::TypeDefinition(#name.to_string(), #typ)
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
            Statement::ObjectDefinition(o) => {
                quote! {
                    Statement::ObjectDefinition(#o)
                }
            }
            Statement::Enum(e) => {
                quote! {
                    Statement::Enum(#e)
                }
            }
            Statement::Loop(s) => {
                quote! {
                    Statement::Loop(#s)
                }
            }
            Statement::For {
                body,
                each,
                expression,
            } => {
                quote! {
                    Statement::For {
                        body: #body,
                        each: #each,
                        expression: #expression
                    }
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for FunctionDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionDefinition {
            name,
            type_definition,
            body,
            lifecycle,
        } = self;
        let l = option(lifecycle);
        let name = name.as_str();
        tokens.extend(quote! {
            FunctionDefinition {
                name: #name.to_string(),
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

impl ToTokens for FunctionArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionArgument {
            name,
            default,
            function_type,
            var_arg,
            rest,
        } = self;
        let d = option(default);
        let name = name.as_str();
        tokens.extend(quote! {
            FunctionArgument {
                name: #name.to_string(),
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

impl ToTokens for FunctionSignature {
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

impl ToTokens for FunctionDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FunctionDeclaration::Declaration {
                name,
                type_definition,
            } => {
                quote! {
                    FunctionDeclaration::Declaration {
                        name: #name.to_string(),
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

impl ToTokens for ModuleTraitDefinition {
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

impl ToTokens for TraitDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TraitDefinition { name, functions } = self;
        let functions = csv_vec(functions);
        tokens.extend(quote! {
            TraitDefinition {
                name: #name.to_string(),
                functions: #functions
            }
        })
    }
}

impl ToTokens for ObjectDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rt = &self.rigz_type;
        let c = &self.constructor;
        let func = csv_vec(&self.functions);
        let f = csv_vec(&self.fields);
        tokens.extend(quote! {
            ObjectDefinition {
                rigz_type: #rt,
                constructor: #c,
                fields: #f,
                functions: #func
            }
        })
    }
}

impl ToTokens for Constructor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Constructor::Default => {
                quote! {
                    Constructor::Default
                }
            }
            Constructor::Declaration(d, var) => {
                let d = csv_vec(d);
                let v = option(var);
                quote! {
                    Constructor::Declaration(#d, #v)
                }
            }
            Constructor::Definition(s, var, b) => {
                let v = option(var);
                let s = csv_vec(s);
                quote! {
                    Constructor::Definition(#s, #v, #b)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for ObjectAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let n = &self.name;
        let a = &self.attr_type;
        let d = option(&self.default);
        tokens.extend(quote! {
            ObjectAttr {
                name: #n.to_string(),
                attr_type: #a,
                default: #d
            }
        })
    }
}
