use crate::prepare::{CallSignature, FunctionCallSignatures, ProgramParser};
use crate::{RigzBuilder, RigzType, UnaryOperation};
use rigz_ast::{Element, Expression, FunctionExpression, Scope, ValidationError};
use std::cmp::Ordering;
use std::collections::HashSet;

impl<T: RigzBuilder> ProgramParser<'_, T> {
    fn scope_type(&mut self, scope: &Scope) -> Result<RigzType, ValidationError> {
        let e = scope.elements.last().expect("Invalid scope");
        match e {
            Element::Statement(_) => Ok(RigzType::None),
            Element::Expression(e) => self.rigz_type(e),
        }
    }

    pub(crate) fn rigz_type(
        &mut self,
        expression: &Expression,
    ) -> Result<RigzType, ValidationError> {
        // todo arguments should be checked for function calls here for best match
        let t = match expression {
            Expression::This => self.identifiers["self"].clone().rigz_type,
            Expression::Value(v) => v.rigz_type(),
            Expression::Error(_) => RigzType::Error,
            Expression::Identifier(a) => match self.identifiers.get(a) {
                None => {
                    self.check_module_exists(a)?;
                    match self.function_scopes.get(a) {
                        None => {
                            return Err(ValidationError::MissingExpression(format!(
                                "identifier {a} does not exist"
                            )))
                        }
                        Some(f) => match Self::function_call_return_type(a, f) {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(ValidationError::MissingExpression(format!(
                                    "{a} does not match existing functions"
                                )))
                            }
                        },
                    }
                }
                Some(v) => v.clone().rigz_type,
            },
            Expression::BinExp(lhs, _, rhs) => {
                let rhs = self.rigz_type(rhs)?;
                let lhs = self.rigz_type(lhs)?;

                match lhs.partial_cmp(&rhs) {
                    None => RigzType::Any,
                    Some(ord) => match ord {
                        Ordering::Less => rhs,
                        Ordering::Equal => lhs,
                        Ordering::Greater => lhs,
                    },
                }
            }
            Expression::UnaryExp(o, e) => match o {
                UnaryOperation::Not => match self.rigz_type(e)? {
                    RigzType::Error => RigzType::Error,
                    _ => RigzType::Bool,
                },
                UnaryOperation::Neg | UnaryOperation::Reverse => self.rigz_type(e)?,
                UnaryOperation::Print
                | UnaryOperation::EPrint
                | UnaryOperation::PrintLn
                | UnaryOperation::EPrintLn => RigzType::None,
            },
            Expression::Cast(_, r) => r.clone(),
            Expression::Scope(s) => self.scope_type(s)?,
            Expression::Function(fe) => self.function_type(fe)?,
            Expression::Symbol(_) => RigzType::String,
            Expression::If { then, branch, .. } => match branch {
                None => self.scope_type(then)?,
                Some(branch) => {
                    let then = self.scope_type(then)?;
                    let branch = self.scope_type(branch)?;
                    if then == branch {
                        then
                    } else {
                        RigzType::Composite(vec![then, branch])
                    }
                }
            },
            Expression::Unless { then, .. } => self.scope_type(then)?,
            Expression::Return(e) => match e {
                None => RigzType::None,
                Some(e) => self.rigz_type(e)?,
            },
            Expression::Lambda { body, .. } => self.rigz_type(body)?,
            Expression::ForList { body, .. } => RigzType::List(self.rigz_type(body)?.into()),
            Expression::ForMap { key, value, .. } => match value {
                None => {
                    let key = self.rigz_type(key)?;
                    let value = match &key {
                        RigzType::Tuple(t) => t[1].clone(),
                        _ => {
                            return Err(ValidationError::MissingExpression(format!(
                                "Invalid key in for-map expression {key}"
                            )))
                        }
                    };
                    RigzType::Map(Box::new(key), value.into())
                }
                Some(value) => {
                    RigzType::Map(self.rigz_type(key)?.into(), self.rigz_type(value)?.into())
                }
            },
            Expression::Tuple(e) => {
                let mut result = Vec::with_capacity(e.len());
                for ex in e {
                    result.push(self.rigz_type(ex)?);
                }
                RigzType::Tuple(result)
            }
            // todo more accurate typing
            Expression::List(_) => RigzType::List(Box::new(RigzType::Any)),
            Expression::Map(_) => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            Expression::Index(base, _index) => {
                let base = self.rigz_type(base)?;
                // todo confirm index can be used
                match base {
                    RigzType::None | RigzType::Bool | RigzType::Error => RigzType::Error,
                    RigzType::Any => RigzType::Any,
                    RigzType::Int | RigzType::Float | RigzType::Number => RigzType::Bool,
                    RigzType::String => RigzType::String,
                    RigzType::List(l) | RigzType::Map(_, l) => *l,
                    RigzType::Type => RigzType::Error,
                    _ => todo!(),
                    // todo need to know whether int range, char range, or dynamic
                    /*
                    RigzType::Range => RigzType::Any,
                    RigzType::Function(_, _) => {}
                    RigzType::This => {
                        // todo improve this logic, move most of this to a function
                        match self.identifiers.get("self") {
                            None => RigzType::This,
                            Some(v) => v.rigz_type.clone(),
                        }
                    }
                    RigzType::Tuple(_) => {}
                    RigzType::Union(_) => {}
                    RigzType::Composite(_) => {}
                    RigzType::Custom(_) => {}
                     */
                }
            }
            Expression::Into { next, .. } => {
                // todo check base arg
                self.function_type(next)?
            }
        };
        Ok(t)
    }

    fn function_type(&mut self, fe: &FunctionExpression) -> Result<RigzType, ValidationError> {
        let e = match fe {
            FunctionExpression::FunctionCall(name, _) => {
                if matches!(name.as_str(), "puts" | "log" | "sleep") {
                    return Ok(RigzType::None);
                }

                if matches!(name.as_str(), "send" | "spawn") {
                    return Ok(RigzType::Int);
                }

                if name == "broadcast" {
                    return Ok(RigzType::List(RigzType::Int.into()));
                }

                if name == "receive" {
                    return Ok(RigzType::Any);
                }

                self.check_module_exists(name)?;
                match self.function_scopes.get(name) {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "function {name} does not exist"
                        )))
                    }
                    Some(f) => Self::function_call_return_type(name, f)?,
                }
            }
            FunctionExpression::TypeFunctionCall(r, name, _) => {
                self.check_module_exists(name)?;
                match self.function_scopes.get(name) {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "typed extension function {r}.{name} does not exist"
                        )))
                    }
                    Some(f) => {
                        // todo ignore extension functions here
                        if f.len() > 1 {
                            // todo support union types
                            let matched: HashSet<_> = f
                                .iter()
                                .filter_map(|cs| match cs {
                                    CallSignature::Function(f, _) => match &f.self_type {
                                        None => None,
                                        Some(ft) => {
                                            if &ft.rigz_type == r {
                                                Some(f.return_type.rigz_type.clone())
                                            } else {
                                                None
                                            }
                                        }
                                    },
                                    CallSignature::Lambda(_, _, ret) => Some(ret.clone()),
                                })
                                .collect();
                            match matched.len() {
                                0 => {
                                    return Err(ValidationError::InvalidFunction(format!(
                                        "typed extension function {r}.{name} does not exist"
                                    )))
                                }
                                1 => matched.iter().next().cloned().unwrap(),
                                _ => r.clone(),
                            }
                        } else {
                            f[0].rigz_type()
                        }
                    }
                }
            }
            FunctionExpression::InstanceFunctionCall(ex, calls, _) => {
                let this = self.rigz_type(ex)?;
                let this = match this {
                    RigzType::This => match self.identifiers.get("self") {
                        None => RigzType::This,
                        Some(v) => v.rigz_type.clone(),
                    },
                    _ => this,
                };
                let name = calls.last().expect("Invalid instance function call");
                // todo need to handle call chaining
                self.check_module_exists(name)?;
                match self.function_scopes.get(name) {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "extension function {this}.{name} does not exist",
                        )))
                    }
                    Some(f) => {
                        // todo ignore extension functions here
                        if f.len() > 1 {
                            // todo support union types
                            let matched: HashSet<_> = f
                                .iter()
                                .filter_map(|cs| match cs {
                                    CallSignature::Function(f, _) => f
                                        .self_type
                                        .as_ref()
                                        .filter(|t| t.rigz_type == this)
                                        .map(|_| f.return_type.rigz_type.clone()),
                                    CallSignature::Lambda(_, _, ret) => Some(ret.clone()),
                                })
                                .collect();
                            match matched.len() {
                                0 => {
                                    return Err(ValidationError::InvalidFunction(format!(
                                        "extension {name} does not exist"
                                    )))
                                }
                                1 => matched.iter().next().cloned().unwrap(),
                                _ => {
                                    dbg!(f);
                                    this
                                }
                            }
                        } else {
                            f[0].rigz_type()
                        }
                    }
                }
            }
        };
        Ok(e)
    }

    fn function_call_return_type(
        name: &str,
        f: &FunctionCallSignatures,
    ) -> Result<RigzType, ValidationError> {
        let matched: HashSet<_> = f
            .iter()
            .filter_map(|cs| match cs {
                CallSignature::Function(f, _) => match &f.self_type {
                    None => Some(f.return_type.rigz_type.clone()),
                    Some(_) => None,
                },
                CallSignature::Lambda(_, _, ret) => Some(ret.clone()),
            })
            .collect();

        Ok(match matched.len() {
            0 => {
                return Err(ValidationError::InvalidFunction(format!(
                    "function {name} does not exist"
                )))
            }
            1 => matched.iter().next().cloned().unwrap(),
            _ => RigzType::Any,
        })
    }
}
