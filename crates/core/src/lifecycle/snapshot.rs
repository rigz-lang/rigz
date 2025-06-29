use crate::{
    EventLifecycle, Lifecycle, MemoizedLifecycle, Snapshot, Stage, StatefulLifecycle,
    TestLifecycle, VMError,
};
use std::fmt::Display;
use std::vec::IntoIter;

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
                return Err(VMError::runtime(format!(
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
                return Err(VMError::runtime(format!(
                    "Illegal Lifecycle byte {b} - {location}"
                )))
            }
        };
        Ok(l)
    }
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
            None => return Err(VMError::runtime(format!("Missing Stage byte {location}"))),
        };

        let s = match next {
            0 => Stage::Parse,
            1 => Stage::Run,
            2 => Stage::Halt,
            3 => Stage::Custom(Snapshot::from_bytes(bytes, location)?),
            b => return Err(VMError::runtime(format!("Illegal Stage {b} - {location}"))),
        };
        Ok(s)
    }
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
