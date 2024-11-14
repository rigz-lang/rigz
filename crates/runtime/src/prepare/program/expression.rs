use crate::prepare::{FunctionCallSignatures, ProgramParser};
use crate::{RigzBuilder, RigzType, UnaryOperation};
use rigz_ast::{Element, Expression, ValidationError};
use std::cmp::Ordering;
use std::collections::HashSet;

impl<'vm, T: RigzBuilder<'vm>> ProgramParser<'vm, T> {
    pub(crate) fn rigz_type(
        &self,
        expression: &Expression<'vm>,
    ) -> Result<RigzType, ValidationError> {
        let t = match expression {
            Expression::This => RigzType::This,
            Expression::Value(v) => v.rigz_type(),
            Expression::List(_) => RigzType::List(Box::new(RigzType::Any)),
            Expression::Map(_) => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            Expression::Identifier(a) => match self.identifiers.get(a) {
                None => match self.function_scopes.get(a) {
                    None => {
                        return Err(ValidationError::MissingExpression(format!(
                            "variable {a} does not exist"
                        )))
                    }
                    Some(f) => match Self::function_call_return_type(a, f) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(ValidationError::MissingExpression(format!(
                                "variable {a} does not exist"
                            )))
                        }
                    },
                },
                Some(v) => v.clone(),
            },
            Expression::BinExp(lhs, _, rhs) => {
                let lhs = self.rigz_type(lhs)?;
                let rhs = self.rigz_type(rhs)?;

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
            Expression::Scope(s) => {
                let e = s.elements.last().expect("Invalid scope");
                match e {
                    Element::Statement(_) => RigzType::None,
                    Element::Expression(e) => self.rigz_type(e)?,
                }
            }
            Expression::FunctionCall(name, _) => match self.function_scopes.get(name) {
                None => {
                    return Err(ValidationError::InvalidFunction(format!(
                        "function {name} does not exist"
                    )))
                }
                Some(f) => Self::function_call_return_type(name, f)?,
            },
            Expression::TypeFunctionCall(r, name, _) => {
                match self.function_scopes.get(name) {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "extension function {r}.{name} does not exist"
                        )))
                    }
                    Some(f) => {
                        // todo ignore extension functions here
                        if f.len() > 1 {
                            // todo support union types
                            let matched: HashSet<_> = f
                                .iter()
                                .filter_map(|(f, _)| match &f.self_type {
                                    None => None,
                                    Some((ft, _)) => {
                                        if &ft.rigz_type == r {
                                            Some(f.return_type.0.rigz_type.clone())
                                        } else {
                                            None
                                        }
                                    }
                                })
                                .collect();
                            match matched.len() {
                                0 => {
                                    return Err(ValidationError::InvalidFunction(format!(
                                        "extension function {r}.{name} does not exist"
                                    )))
                                }
                                1 => matched.iter().next().cloned().unwrap(),
                                _ => RigzType::Any,
                            }
                        } else {
                            f[0].0.return_type.0.rigz_type.clone()
                        }
                    }
                }
            }
            Expression::InstanceFunctionCall(_, calls, _) => {
                let name = calls.last().expect("Invalid instance function call");
                // todo need to handle call chaining
                match self.function_scopes.get(name) {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "extension function {name} does not exist"
                        )))
                    }
                    Some(f) => {
                        // todo ignore extension functions here
                        if f.len() > 1 {
                            // todo support union types
                            let matched: HashSet<_> = f
                                .iter()
                                .filter_map(|(f, _)| match &f.self_type {
                                    None => None,
                                    Some(_) => Some(f.return_type.0.rigz_type.clone()),
                                })
                                .collect();
                            match matched.len() {
                                0 => {
                                    return Err(ValidationError::InvalidFunction(format!(
                                        "extension {name} does not exist"
                                    )))
                                }
                                1 => matched.iter().next().cloned().unwrap(),
                                _ => RigzType::Any,
                            }
                        } else {
                            f[0].0.return_type.0.rigz_type.clone()
                        }
                    }
                }
            }
            _ => RigzType::Any,
        };
        Ok(t)
    }

    fn function_call_return_type(
        name: &&str,
        f: &FunctionCallSignatures,
    ) -> Result<RigzType, ValidationError> {
        let matched: HashSet<_> = f
            .iter()
            .filter_map(|(f, _)| match &f.self_type {
                None => Some(f.return_type.0.rigz_type.clone()),
                Some(_) => None,
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
