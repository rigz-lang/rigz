use crate::{FastHashMap, IndexMap, IndexSet, VMError, ValueRange};
use fxhash::FxBuildHasher;
use itertools::Itertools;
use log::Level;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::IntoIter;
// todo make snapshot a feature

pub trait Snapshot: Sized {
    fn as_bytes(&self) -> Vec<u8>;

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError>;
}

impl Snapshot for Level {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Level::Error => vec![0],
            Level::Warn => vec![1],
            Level::Info => vec![2],
            Level::Debug => vec![3],
            Level::Trace => vec![4],
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => return Err(VMError::runtime(format!("Missing Level byte {location}"))),
            Some(b) => b,
        };

        let l = match next {
            0 => Level::Error,
            1 => Level::Warn,
            2 => Level::Info,
            3 => Level::Debug,
            4 => Level::Trace,
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal Level byte {b} {location}"
                )))
            }
        };
        Ok(l)
    }
}

impl Snapshot for usize {
    fn as_bytes(&self) -> Vec<u8> {
        (*self as u64).to_le_bytes().to_vec()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next_array() {
            None => Err(VMError::runtime(format!("Missing {location} bytes"))),
            Some(d) => Ok(u64::from_le_bytes(d) as usize),
        }
    }
}

impl<T: Snapshot> Snapshot for Vec<T> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.len());
        for v in self {
            res.extend(v.as_bytes());
        }
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = Snapshot::from_bytes(bytes, &format!("{location} len"))?;
        let mut results = Vec::with_capacity(len);
        for _ in 0..len {
            results.push(T::from_bytes(bytes, location)?);
        }
        Ok(results)
    }
}

impl<V: Snapshot + Hash + Eq> Snapshot for IndexSet<V> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.len());
        for v in self {
            res.extend(v.as_bytes());
        }
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = Snapshot::from_bytes(bytes, &format!("{location} len"))?;
        let mut results = IndexSet::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let v = V::from_bytes(bytes, location)?;
            results.insert(v);
        }
        Ok(results)
    }
}

impl<K: Snapshot + Hash + Eq, V: Snapshot> Snapshot for IndexMap<K, V> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.len());
        for (k, v) in self {
            res.extend(k.as_bytes());
            res.extend(v.as_bytes());
        }
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = Snapshot::from_bytes(bytes, &format!("{location} len"))?;
        let mut results = IndexMap::with_capacity_and_hasher(len, Default::default());
        for _ in 0..len {
            let k = K::from_bytes(bytes, location)?;
            let v = V::from_bytes(bytes, location)?;
            results.insert(k, v);
        }
        Ok(results)
    }
}

impl<K: Snapshot + Hash + Eq, V: Snapshot> Snapshot for FastHashMap<K, V> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.len());
        for (k, v) in self {
            res.extend(k.as_bytes());
            res.extend(v.as_bytes());
        }
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = Snapshot::from_bytes(bytes, &format!("{location} len"))?;
        let mut results = FastHashMap::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let k = K::from_bytes(bytes, location)?;
            let v = V::from_bytes(bytes, location)?;
            results.insert(k, v);
        }
        Ok(results)
    }
}

impl<K: Snapshot + Hash + Eq, V: Snapshot> Snapshot for HashMap<K, V> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.len());
        for (k, v) in self {
            res.extend(k.as_bytes());
            res.extend(v.as_bytes());
        }
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = Snapshot::from_bytes(bytes, &format!("{location} len"))?;
        let mut results = HashMap::with_capacity(len);
        for _ in 0..len {
            let k = K::from_bytes(bytes, location)?;
            let v = V::from_bytes(bytes, location)?;
            results.insert(k, v);
        }
        Ok(results)
    }
}

impl<T: Snapshot> Snapshot for Option<T> {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            None => vec![0],
            Some(v) => {
                let mut res = vec![1];
                res.extend(v.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => return Err(VMError::runtime(format!("Missing Option byte {location}"))),
            Some(b) => b,
        };

        let v = match next {
            0 => None,
            1 => Some(T::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal Option byte {b} - {location}"
                )))
            }
        };
        Ok(v)
    }
}

impl Snapshot for bool {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next() {
            None => Err(VMError::runtime(format!("Missing bool byte {location}"))),
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            Some(b) => Err(VMError::runtime(format!(
                "Illegal bool byte {b} - {location}"
            ))),
        }
    }
}

impl<A: Snapshot, B: Snapshot> Snapshot for (A, B) {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.0.as_bytes();
        res.extend(self.1.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let a = A::from_bytes(bytes, location)?;
        let b = B::from_bytes(bytes, location)?;
        Ok((a, b))
    }
}

impl<A: Snapshot, B: Snapshot, C: Snapshot> Snapshot for (A, B, C) {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.0.as_bytes();
        res.extend(self.1.as_bytes());
        res.extend(self.2.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let a = A::from_bytes(bytes, location)?;
        let b = B::from_bytes(bytes, location)?;
        let c = C::from_bytes(bytes, location)?;
        Ok((a, b, c))
    }
}

impl<T: Snapshot> Snapshot for Range<T> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.start.as_bytes();
        res.extend(self.end.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let start = T::from_bytes(bytes, location)?;
        let end = T::from_bytes(bytes, location)?;
        Ok(start..end)
    }
}

impl Snapshot for i64 {
    fn as_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next_array() {
            None => Err(VMError::runtime(format!("Missing i64 byte {location}"))),
            Some(n) => Ok(i64::from_be_bytes(n)),
        }
    }
}

impl Snapshot for char {
    fn as_bytes(&self) -> Vec<u8> {
        (*self as u32).as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let v = Snapshot::from_bytes(bytes, location)?;
        match char::from_u32(v) {
            None => Err(VMError::runtime(format!(
                "{location} - Failed to convert {v} to char"
            ))),
            Some(c) => Ok(c),
        }
    }
}

impl Snapshot for u32 {
    fn as_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next_array() {
            None => Err(VMError::runtime(format!("{location} - failed to read u32"))),
            Some(v) => Ok(u32::from_le_bytes(v)),
        }
    }
}

impl<T: Snapshot> Snapshot for Box<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.as_ref().as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(Box::new(T::from_bytes(bytes, location)?))
    }
}

impl<T: Snapshot> Snapshot for RefCell<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.borrow().as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(RefCell::new(T::from_bytes(bytes, location)?))
    }
}

impl<T: Snapshot> Snapshot for Rc<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.as_ref().as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(T::from_bytes(bytes, location)?.into())
    }
}

impl<T: Snapshot> Snapshot for Arc<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.as_ref().as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(T::from_bytes(bytes, location)?.into())
    }
}

impl Snapshot for String {
    fn as_bytes(&self) -> Vec<u8> {
        let mut l = self.len().as_bytes();
        l.extend(self.bytes());
        l
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let len = usize::from_bytes(bytes, location)?;
        let bytes: Vec<_> = bytes.take(len).collect();
        let b = bytes.len();
        if b != len {
            return Err(VMError::runtime(format!(
                "{location} String len {b} != {len}"
            )));
        }
        let s = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                return Err(VMError::runtime(format!(
                    "{location} Failed to create string: {e}"
                )))
            }
        };
        // todo for long running projects this is an issue, need to store strings somewhere on the VM
        Ok(s)
    }
}

impl Snapshot for ValueRange {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            ValueRange::Int(r) => {
                let mut res = vec![0];
                res.extend(r.as_bytes());
                res
            }
            ValueRange::Char(r) => {
                let mut res = vec![1];
                res.extend(r.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => {
                return Err(VMError::runtime(format!(
                    "Missing ValueRange byte {location}"
                )))
            }
        };

        let v = match next {
            0 => ValueRange::Int(Snapshot::from_bytes(bytes, location)?),
            1 => ValueRange::Char(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal ValueRange byte {b} - {location}"
                )))
            }
        };
        Ok(v)
    }
}

impl Snapshot for VMError {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            VMError::TimeoutError(m) => {
                let mut res = vec![0];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::RuntimeError(m) => {
                let mut res = vec![1];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::EmptyStack(m) => {
                let mut res = vec![2];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::ConversionError(m) => {
                let mut res = vec![3];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::ScopeDoesNotExist(m) => {
                let mut res = vec![4];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::UnsupportedOperation(m) => {
                let mut res = vec![5];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::VariableDoesNotExist(m) => {
                let mut res = vec![6];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::InvalidModule(m) => {
                let mut res = vec![7];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::InvalidModuleFunction(m) => {
                let mut res = vec![8];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::LifecycleError(m) => {
                let mut res = vec![9];
                res.extend(Snapshot::as_bytes(m));
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(s) => s,
            None => return Err(VMError::runtime(format!("Missing VMError {location}"))),
        };
        if next == 1 {
            return Ok(VMError::RuntimeError(Snapshot::from_bytes(
                bytes,
                &format!("VMError::Runtime - {location}"),
            )?));
        }
        let message = String::from_bytes(bytes, &format!("VMError - {location}"))?;
        let e = match next {
            0 => VMError::TimeoutError(message),
            2 => VMError::EmptyStack(message),
            3 => VMError::ConversionError(message),
            4 => VMError::ScopeDoesNotExist(message),
            5 => VMError::UnsupportedOperation(message),
            6 => VMError::VariableDoesNotExist(message),
            7 => VMError::InvalidModule(message),
            8 => VMError::InvalidModuleFunction(message),
            9 => VMError::LifecycleError(message),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal VMError byte {b} {location}"
                )))
            }
        };
        Ok(e)
    }
}
