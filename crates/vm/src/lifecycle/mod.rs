use crate::{VMError, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    On(EventLifecycle),
    After(StatefulLifecycle),
    Memo(MemoizedLifecycle),
    Test(TestLifecycle),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventLifecycle {
    pub event: String,
    pub scope_id: usize,
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
    pub scope_id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoizedLifecycle {
    pub scope_id: usize,
    pub results: HashMap<Vec<Value>, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestLifecycle;

#[derive(Clone, Debug, Eq)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub failure_messages: Vec<(String, VMError)>,
    pub duration: Duration,
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
            "test result: \x1b[32mok\x1b[0m".to_string()
        } else {
            let mut result = "\nfailures:\n".to_string();
            for (name, reason) in &self.failure_messages {
                result.push_str(format!("\t{name}: {reason}\n").as_str())
            }
            result.push_str("\ntest result: \x1b[31mFAILED\x1b[0m");
            result
        };

        write!(
            f,
            "{preamble}. passed: {}, failed: {}, finished in {:?}",
            self.passed, self.failed, self.duration
        )
    }
}
