use crate::vm::Snapshot;
use crate::VMError;
use itertools::Itertools;
use std::fmt::Display;
use std::vec::IntoIter;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VMOptions {
    pub enable_logging: bool,
    pub disable_modules: bool,
    pub disable_variable_cleanup: bool,
    pub max_depth: usize,
}

impl Default for VMOptions {
    fn default() -> Self {
        VMOptions {
            enable_logging: true,
            disable_modules: false,
            disable_variable_cleanup: false,
            max_depth: 1024,
        }
    }
}

impl Snapshot for VMOptions {
    fn as_bytes(&self) -> Vec<u8> {
        let mut options = 0;
        options |= self.enable_logging as u8;
        options |= (self.disable_modules as u8) << 1;
        options |= (self.disable_variable_cleanup as u8) << 2;
        let mut result = vec![options];
        result.extend((self.max_depth as u64).to_le_bytes());
        result
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let byte = match bytes.next() {
            Some(b) => b,
            None => return Err(VMError::RuntimeError(format!("Missing {location} byte"))),
        };
        let max_depth = Snapshot::from_bytes(bytes, &format!("{location} max_depth"))?;
        Ok(VMOptions {
            enable_logging: (byte & 1) == 1,
            disable_modules: (byte & 1 << 1) == 2,
            disable_variable_cleanup: (byte & 1 << 2) == 4,
            max_depth,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::vm::VMOptions;
    use crate::Snapshot;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn options_snapshot() {
        let options = VMOptions {
            enable_logging: true,
            disable_modules: true,
            disable_variable_cleanup: true,
            ..Default::default()
        };
        let byte = options.as_bytes();
        assert_eq!(
            VMOptions::from_bytes(&mut byte.into_iter(), &"options_snapshot_test"),
            Ok(options)
        )
    }
}
