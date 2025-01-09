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

impl VMOptions {
    // todo use bytes instead of byte
    pub(crate) fn as_byte(&self) -> u8 {
        let mut result = 0;
        result |= self.enable_logging as u8;
        result |= (self.disable_modules as u8) << 1;
        result |= (self.disable_variable_cleanup as u8) << 2;
        result
    }

    pub(crate) fn from_byte(byte: u8) -> Self {
        VMOptions {
            enable_logging: (byte & 1) == 1,
            disable_modules: (byte & 1 << 1) == 2,
            disable_variable_cleanup: (byte & 1 << 2) == 4,
            max_depth: 1024,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::vm::VMOptions;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn options_snapshot() {
        let options = VMOptions {
            enable_logging: true,
            disable_modules: true,
            disable_variable_cleanup: true,
            ..Default::default()
        };
        let byte = options.as_byte();
        assert_eq!(VMOptions::from_byte(byte), options)
    }
}
