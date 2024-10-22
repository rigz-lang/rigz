use std::collections::HashMap;
use crate::Value;

#[derive(Clone, Debug)]
pub enum Lifecycle {
    On(EventLifecycle),
    After(StatefulLifecycle),
    Memo(MemoizedLifecycle),
}

#[derive(Clone, Debug)]
pub struct EventLifecycle {
    pub event: String,
    pub scope_id: usize,
}

#[derive(Clone, Debug)]
pub enum Stage {
    Parse,
    Run,
    Halt,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct StatefulLifecycle {
    pub stage: Stage,
    pub scope_id: usize,
}

#[derive(Clone, Debug)]
pub struct MemoizedLifecycle {
    pub scope_id: usize,
    pub results: HashMap<Vec<Value>, Value>
}