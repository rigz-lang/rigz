use crate::process::Reference;
use crate::VMError;
use indexmap::IndexMap;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::vec::IntoIter;

pub trait Snapshot: Sized {
    fn as_bytes(&self) -> Vec<u8>;

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError>;
}

impl Snapshot for usize {
    fn as_bytes(&self) -> Vec<u8> {
        (*self as u64).to_le_bytes().to_vec()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next_array() {
            None => Err(VMError::RuntimeError(format!("Missing {location} bytes"))),
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
        let mut results = IndexMap::with_capacity(len);
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
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Missing Option byte {location}"
                )))
            }
            Some(b) => b,
        };

        let v = match next {
            0 => None,
            1 => Some(T::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::RuntimeError(format!(
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
            None => Err(VMError::RuntimeError(format!(
                "Missing bool byte {location}"
            ))),
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            Some(b) => Err(VMError::RuntimeError(format!(
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

impl Snapshot for char {
    fn as_bytes(&self) -> Vec<u8> {
        (*self as u32).as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let v = Snapshot::from_bytes(bytes, location)?;
        match char::from_u32(v) {
            None => Err(VMError::RuntimeError(format!(
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
            None => Err(VMError::RuntimeError(format!(
                "{location} - failed to read u32"
            ))),
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

impl<T: Snapshot> Snapshot for Reference<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.as_ref().as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(Reference::new(T::from_bytes(bytes, location)?))
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
            return Err(VMError::RuntimeError(format!(
                "{location} String len {b} != {len}"
            )));
        }
        let s = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                return Err(VMError::RuntimeError(format!(
                    "{location} Failed to create string: {e}"
                )))
            }
        };
        // todo for long running projects this is an issue, need to store strings somewhere on the VM
        Ok(s)
    }
}
