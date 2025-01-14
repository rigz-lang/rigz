use crate::{Snapshot, VMError, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use std::time::Duration;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    On(EventLifecycle),
    After(StatefulLifecycle),
    Memo(MemoizedLifecycle),
    Test(TestLifecycle),
    Composite(Vec<Lifecycle>),
}

impl Snapshot for Lifecycle {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Lifecycle::On(l) => {
                let mut res = vec![0];
                res.extend(l.as_bytes());
                res
            }
            Lifecycle::After(l) => {
                let mut res = vec![1];
                res.extend(l.as_bytes());
                res
            }
            Lifecycle::Memo(l) => {
                let mut res = vec![2];
                res.extend(l.as_bytes());
                res
            }
            Lifecycle::Test(_) => vec![3],
            Lifecycle::Composite(l) => {
                let mut res = vec![4];
                res.extend(l.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Missing Lifecycle byte {location}"
                )))
            }
        };

        let l = match next {
            0 => Lifecycle::On(Snapshot::from_bytes(bytes, location)?),
            1 => Lifecycle::After(Snapshot::from_bytes(bytes, location)?),
            2 => Lifecycle::Memo(Snapshot::from_bytes(bytes, location)?),
            3 => Lifecycle::Test(TestLifecycle),
            4 => Lifecycle::Composite(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal Lifecycle byte {b} - {location}"
                )))
            }
        };
        Ok(l)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventLifecycle {
    pub event: String,
}

impl Snapshot for EventLifecycle {
    fn as_bytes(&self) -> Vec<u8> {
        Snapshot::as_bytes(&self.event)
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(EventLifecycle {
            event: Snapshot::from_bytes(bytes, location)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    Parse,
    Run,
    Halt,
    Custom(String),
}

impl Snapshot for Stage {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Stage::Parse => vec![0],
            Stage::Run => vec![1],
            Stage::Halt => vec![2],
            Stage::Custom(c) => {
                let mut res = vec![3];
                res.extend(Snapshot::as_bytes(c));
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Missing Stage byte {location}"
                )))
            }
        };

        let s = match next {
            0 => Stage::Parse,
            1 => Stage::Run,
            2 => Stage::Halt,
            3 => Stage::Custom(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal Stage {b} - {location}"
                )))
            }
        };
        Ok(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatefulLifecycle {
    pub stage: Stage,
}

impl Snapshot for StatefulLifecycle {
    fn as_bytes(&self) -> Vec<u8> {
        Snapshot::as_bytes(&self.stage)
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(StatefulLifecycle {
            stage: Snapshot::from_bytes(bytes, location)?,
        })
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct MemoizedLifecycle {
    pub results: HashMap<Vec<Value>, Value>,
}

impl Snapshot for MemoizedLifecycle {
    fn as_bytes(&self) -> Vec<u8> {
        Snapshot::as_bytes(&self.results)
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(MemoizedLifecycle {
            results: Snapshot::from_bytes(bytes, location)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestLifecycle;

#[derive(Clone, Debug, Eq, Default)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub failure_messages: Vec<(String, VMError)>,
    pub duration: Duration,
}

impl AddAssign for TestResults {
    fn add_assign(&mut self, rhs: Self) {
        self.passed += rhs.passed;
        self.failed += rhs.failed;
        self.failure_messages.extend(rhs.failure_messages);
        self.duration += rhs.duration;
    }
}
impl PartialEq for TestResults {
    fn eq(&self, other: &Self) -> bool {
        self.passed == other.passed
            && self.failed == other.failed
            && self.failure_messages == other.failure_messages
    }
}

impl Display for TestResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let success = self.failed == 0;

        let preamble = if success {
            if cfg!(feature = "js") {
                "test result: ok".to_string()
            } else {
                "test result: \x1b[32mok\x1b[0m".to_string()
            }
        } else {
            let mut result = "\nfailures:\n".to_string();
            for (name, reason) in &self.failure_messages {
                result.push_str(format!("\t{name}: {reason}\n").as_str())
            }
            let res = if cfg!(feature = "js") {
                "\ntest result: FAILED"
            } else {
                "\ntest result: \x1b[31mFAILED\x1b[0m"
            };
            result.push_str(res);
            result
        };

        write!(
            f,
            "{preamble}. passed: {}, failed: {}, finished in {:?}",
            self.passed, self.failed, self.duration
        )
    }
}
