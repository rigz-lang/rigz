use crate::{VMError, Value, VM};
use derive_more::IntoIterator;
use dyn_clone::DynClone;
use std::fmt::{Debug, Formatter};

#[derive(Clone, IntoIterator)]
pub struct RigzArgs(pub Vec<Value>);

impl Debug for RigzArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<Vec<Value>> for RigzArgs {
    #[inline]
    fn from(value: Vec<Value>) -> Self {
        RigzArgs(value)
    }
}

impl From<RigzArgs> for Vec<Value> {
    #[inline]
    fn from(value: RigzArgs) -> Self {
        value.0
    }
}

impl RigzArgs {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn first(self) -> Result<Value, VMError> {
        if self.is_empty() {
            return Err(VMError::RuntimeError(
                "Invalid args, expected 1 argument".to_string(),
            ));
        }
        let mut args = self.0;
        Ok(args.remove(0))
    }

    #[inline]
    pub fn take<const N: usize>(self) -> Result<[Value; N], VMError> {
        if self.len() < N {
            return Err(VMError::RuntimeError(format!(
                "Invalid args, expected {N} argument{}",
                if N > 1 { "s" } else { "" }
            )));
        }

        let mut results = [(); N].map(|_| Value::None);
        for (i, v) in self.0.into_iter().enumerate().take(N) {
            results[i] = v;
        }
        Ok(results)
    }

    #[inline]
    pub fn var_args<const START: usize, const COUNT: usize>(
        self,
    ) -> Result<([Value; START], [Vec<Value>; COUNT]), VMError> {
        if self.len() < START {
            return Err(VMError::RuntimeError(format!(
                "Invalid args, expected {START} argument{}",
                if START > 1 { "s" } else { "" }
            )));
        }

        let mut results = [(); START].map(|_| Value::None);
        let mut var = [(); COUNT].map(|_| Vec::new());
        for (i, v) in self.0.into_iter().enumerate() {
            if i < START {
                results[i] = v;
                continue;
            }

            let Value::List(l) = v else {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Var Args at {i} - {v}"
                )));
            };
            var[i - START] = l;
        }
        let min = var[0].len();
        if var.iter().any(|v| v.len() != min) {
            Err(VMError::RuntimeError(format!(
                "Invalid var args, expected all args to contain {min}"
            )))
        } else {
            Ok((results, var))
        }
    }
}

#[cfg(test)]
mod rigz_args {
    use crate::{RigzArgs, Value};

    #[test]
    fn take() {
        let args = RigzArgs(vec![1.into(), 2.into()]);
        let [first] = args.take().expect("Failed to take first");
        assert_eq!(first, 1.into());
    }

    #[test]
    fn var_args_one() {
        let args = RigzArgs(vec![
            1.into(),
            vec![Value::Number(2.into()), 3.into()].into(),
        ]);
        let ([first], [var]) = args.var_args().expect("Failed to get var_args");
        assert_eq!(first, 1.into());
        assert_eq!(var, vec![2.into(), 3.into()]);
    }

    #[test]
    fn var_args_skip_first() {
        let args = RigzArgs(vec![Value::List(vec![1.into(), 2.into(), 3.into()])]);
        let ([], [var]) = args.var_args().expect("Failed to get var_args");
        assert_eq!(var, vec![1.into(), 2.into(), 3.into()]);
    }

    #[test]
    fn var_args_two() {
        let args = RigzArgs(vec![
            1.into(),
            Value::List(vec![2.into()]),
            vec![Value::Number(3.into())].into(),
        ]);
        let ([first], [var1, var2]) = args.var_args().expect("Failed to get var_args");
        assert_eq!(first, 1.into());
        assert_eq!(var1, vec![2.into()]);
        assert_eq!(var2, vec![3.into()]);
    }

    #[test]
    fn var_args_error() {
        let args = RigzArgs(vec![
            1.into(),
            Value::List(vec![2.into()]),
            Value::List(vec![3.into(), 3.into()]),
        ]);
        assert!(
            args.var_args::<1, 2>().is_err(),
            "different lengths of var args were permitted"
        );
    }
}

#[allow(unused_variables)]
pub trait Module<'vm>: Debug + DynClone {
    fn name(&self) -> &'static str;

    fn call(&self, function: &'vm str, args: RigzArgs) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call`",
            self.name()
        )))
    }

    fn call_extension(
        &self,
        this: Value,
        function: &'vm str,
        args: RigzArgs,
    ) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `call_extension`",
            self.name()
        )))
    }

    fn call_mutable_extension(
        &self,
        this: &mut Value,
        function: &'vm str,
        args: RigzArgs,
    ) -> Result<Option<Value>, VMError> {
        Ok(Some(
            VMError::UnsupportedOperation(format!(
                "{} does not implement `call_mutable_extension`",
                self.name()
            ))
            .to_value(),
        ))
    }

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: RigzArgs,
    ) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "{} does not implement `vm_extension`",
            self.name()
        )))
    }

    // todo create proc_macro that uses tree-sitter-rigz for syntax highlighting and compile time syntax validation
    fn trait_definition(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(Module<'_>);
