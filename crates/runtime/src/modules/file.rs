use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait File
        fn read(path: String, encoding = "utf-8") -> String!
        fn write(path: String, contents: String, encoding = "utf-8") -> None!
    end"#
);

use std::fs::{read_to_string, File};
use std::io::Write;

impl RigzFile for FileModule {
    fn read(&self, path: String, encoding: String) -> Result<String, VMError> {
        let path = path.to_string();
        let encoding = encoding.to_string();
        if encoding.as_str().to_lowercase() != "utf-8" {
            return Err(VMError::RuntimeError(format!(
                "Non utf-8 files are not supported yet, received {encoding}"
            )));
        }

        Ok(read_to_string(&path).map_err(|e| VMError::RuntimeError(format!(
            "Failed to read {path} - {e}"
        )))?)
    }

    fn write(&self, path: String, contents: String, encoding: String) -> Result<(), VMError> {
        let path = path.to_string();
        let contents = contents.to_string();
        let encoding = encoding.to_string();
        if encoding.as_str().to_lowercase() != "utf-8" {
            return Err(VMError::RuntimeError(format!(
                "Non utf-8 files are not supported yet, received {encoding}"
            )));
        }
        let mut file = File::open(&path)
            .map_err(|e| VMError::RuntimeError(format!("Failed to open {path} - {e}")))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| VMError::RuntimeError(format!("Failed to write {path} - {e}")))?;
        Ok(())
    }
}