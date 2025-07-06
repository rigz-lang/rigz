#[cfg(feature = "snapshot")]
mod snapshot;

use crate::{ObjectValue, VMError};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    On(EventLifecycle),
    After(StatefulLifecycle),
    Memo(MemoizedLifecycle),
    Test(TestLifecycle),
    Composite(Vec<Lifecycle>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventLifecycle {
    pub event: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    Parse,
    Run,
    Halt,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatefulLifecycle {
    pub stage: Stage,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct MemoizedLifecycle {
    pub results: HashMap<Vec<ObjectValue>, ObjectValue>,
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

impl TestResults {
    pub fn success(&self) -> bool {
        self.failed == 0 && self.failure_messages.is_empty()
    }
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
        let success = self.success();

        let preamble = if success {
            if cfg!(feature = "colors") {
                "test result: \x1b[32mok\x1b[0m".to_string()
            } else {
                "test result: ok".to_string()
            }
        } else {
            let mut result = "\nfailures:\n".to_string();
            for (name, reason) in &self.failure_messages {
                result.push_str(format!("\t{name}: {reason}\n").as_str())
            }
            let res = if cfg!(feature = "colors") {
                "\ntest result: \x1b[31mFAILED\x1b[0m"
            } else {
                "\ntest result: FAILED"
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
