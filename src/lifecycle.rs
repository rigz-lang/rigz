#[derive(Clone, Debug)]
pub enum Lifecycle {
    On(EventLifecycle),
    After(StatefulLifecycle),
}

#[derive(Clone, Debug)]
pub struct EventLifecycle {
    pub event: String,
    pub scope_id: usize
}

#[derive(Clone, Debug)]
pub enum Stage {
    Parse,
    Run,
    Halt,
    Custom(String)
}

#[derive(Clone, Debug)]
pub struct StatefulLifecycle {
    pub stage: Stage,
    pub scope_id: usize
}