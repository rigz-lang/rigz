use crate::{AsPrimitive, ObjectValue, VMError};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct RigzArgs(pub Vec<Rc<RefCell<ObjectValue>>>);

impl Debug for RigzArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<Vec<Rc<RefCell<ObjectValue>>>> for RigzArgs {
    #[inline]
    fn from(value: Vec<Rc<RefCell<ObjectValue>>>) -> Self {
        RigzArgs(value)
    }
}

impl From<RigzArgs> for Vec<Rc<RefCell<ObjectValue>>> {
    #[inline]
    fn from(value: RigzArgs) -> Self {
        value.0
    }
}

impl From<RigzArgs> for Vec<ObjectValue> {
    #[inline]
    fn from(value: RigzArgs) -> Self {
        value.0.into_iter().map(|v| v.borrow().clone()).collect()
    }
}

pub type VarArgs<const START: usize, const COUNT: usize> = (
    [Rc<RefCell<ObjectValue>>; START],
    [Vec<Rc<RefCell<ObjectValue>>>; COUNT],
);

pub type VarArgsRc<const START: usize, const COUNT: usize> = (
    [Rc<RefCell<ObjectValue>>; START],
    [Rc<RefCell<ObjectValue>>; COUNT],
);

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
    pub fn first(self) -> Result<Rc<RefCell<ObjectValue>>, VMError> {
        if self.is_empty() {
            return Err(VMError::runtime(
                "Invalid args, expected 1 argument".to_string(),
            ));
        }
        let mut args = self.0;
        Ok(args.remove(0))
    }

    #[inline]
    pub fn take<const N: usize>(self) -> Result<[Rc<RefCell<ObjectValue>>; N], VMError> {
        if self.len() < N {
            return Err(VMError::runtime(format!(
                "Invalid args, expected {N} argument{}",
                if N > 1 { "s" } else { "" }
            )));
        }

        let mut results = [(); N].map(|_| Rc::new(ObjectValue::default().into()));
        for (i, v) in self.0.into_iter().take(N).rev().enumerate() {
            results[i] = v;
        }
        Ok(results)
    }

    #[inline]
    pub fn var_args<const START: usize, const COUNT: usize>(
        self,
    ) -> Result<VarArgs<START, COUNT>, VMError> {
        if self.len() < START {
            return Err(VMError::runtime(format!(
                "Invalid args, expected {START} argument{}",
                if START > 1 { "s" } else { "" }
            )));
        }

        let mut results = [(); START].map(|_| Rc::new(ObjectValue::default().into()));
        let mut var = [(); COUNT].map(|_| Vec::new());
        for (i, v) in self.0.into_iter().rev().enumerate() {
            if i < START {
                results[i] = v;
                continue;
            }

            match v.borrow().to_list() {
                Ok(l) => {
                    var[i - START] = l;
                }
                Err(e) => {
                    return Err(VMError::runtime(format!(
                        "Invalid Var Args at {i} - {v:?}: {e}"
                    )));
                }
            };
        }
        let min = var[0].len();
        if var.iter().any(|v| v.len() != min) {
            Err(VMError::runtime(format!(
                "Invalid var args, expected all args to contain {min}"
            )))
        } else {
            Ok((results, var))
        }
    }

    #[inline]
    pub fn var_args_rc<const START: usize, const COUNT: usize>(
        self,
    ) -> Result<VarArgsRc<START, COUNT>, VMError> {
        if self.len() < START {
            return Err(VMError::runtime(format!(
                "Invalid args, expected {START} argument{}",
                if START > 1 { "s" } else { "" }
            )));
        }

        let mut results = [(); START].map(|_| Rc::new(ObjectValue::default().into()));
        let mut var_len = [0; COUNT];
        let mut var = [(); COUNT].map(|_| Rc::new(RefCell::new(ObjectValue::default())));
        for (i, v) in self.0.into_iter().rev().enumerate() {
            if i < START {
                results[i] = v;
                continue;
            }

            let rc = v.clone();
            match v.borrow().to_list() {
                Ok(l) => {
                    var_len[i - START] = l.len();
                    var[i - START] = rc;
                }
                Err(e) => {
                    return Err(VMError::runtime(format!(
                        "Invalid Var Args at {i} - {v:?}: {e}"
                    )));
                }
            };
        }
        let min = var_len[0];
        if var_len.iter().any(|v| *v != min) {
            Err(VMError::runtime(format!(
                "Invalid var args, expected all args to contain {min}"
            )))
        } else {
            Ok((results, var))
        }
    }
}

#[cfg(test)]
pub mod rigz_args {
    use crate::{ObjectValue, PrimitiveValue, RigzArgs};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn take() {
        let args: RigzArgs = RigzArgs(vec![
            Rc::new(RefCell::new(1.into())),
            Rc::new(RefCell::new(2.into())),
        ]);
        let [first] = args.take().expect("Failed to take first");
        assert_eq!(first, Rc::new(RefCell::new(1.into())));
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn var_args_one() {
        let a: ObjectValue = 2.into();
        let b: ObjectValue = 3.into();
        let args: RigzArgs = RigzArgs(vec![
            Rc::new(RefCell::new(vec![a, b].into())),
            Rc::new(RefCell::new(1.into())),
        ]);
        let ([first], [var]) = args.var_args().expect("Failed to get var_args");
        assert_eq!(first, Rc::new(RefCell::new(1.into())));
        assert_eq!(
            var,
            vec![
                Rc::new(RefCell::new(2.into())),
                Rc::new(RefCell::new(3.into()))
            ]
        );
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn var_args_skip_first() {
        let args: RigzArgs = RigzArgs(vec![Rc::new(RefCell::new(
            ObjectValue::List(vec![
                Rc::new(RefCell::new(1.into())),
                Rc::new(RefCell::new(2.into())),
                Rc::new(RefCell::new(3.into())),
            ])
            .into(),
        ))]);
        let ([], [var]) = args.var_args().expect("Failed to get var_args");
        assert_eq!(
            var,
            vec![
                Rc::new(RefCell::new(1.into())),
                Rc::new(RefCell::new(2.into())),
                Rc::new(RefCell::new(3.into()))
            ]
        );
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn var_args_two() {
        let args: RigzArgs = RigzArgs(vec![
            Rc::new(RefCell::new(vec![PrimitiveValue::Number(3.into())].into())),
            Rc::new(RefCell::new(
                ObjectValue::List(vec![Rc::new(RefCell::new(2.into()))]).into(),
            )),
            Rc::new(RefCell::new(1.into())),
        ]);
        let ([first], [var1, var2]) = args.var_args().expect("Failed to get var_args");
        let v: ObjectValue = 1.into();
        assert_eq!(first, v.into());
        assert_eq!(var1, vec![Rc::new(RefCell::new(2.into()))]);
        assert_eq!(var2, vec![Rc::new(RefCell::new(3.into()))]);
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn var_args_error() {
        let args: RigzArgs = RigzArgs(vec![
            Rc::new(RefCell::new(1.into())),
            Rc::new(RefCell::new(
                ObjectValue::List(vec![Rc::new(RefCell::new(2.into()))]).into(),
            )),
            Rc::new(RefCell::new(
                ObjectValue::List(vec![
                    Rc::new(RefCell::new(3.into())),
                    Rc::new(RefCell::new(3.into())),
                ])
                .into(),
            )),
        ]);
        assert!(
            args.var_args::<1, 2>().is_err(),
            "different lengths of var args were permitted"
        );
    }
}
