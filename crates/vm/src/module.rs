use crate::{Reference, VM};
use rigz_core::Object;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

// #[allow(unused_variables)]
// pub trait Module: Debug {
//     fn name(&self) -> &'static str;
//
//     fn call(&self, function: String, args: RigzArgs) -> Result<PrimitiveValue, VMError> {
//         Err(VMError::UnsupportedOperation(format!(
//             "{} does not implement `call`",
//             self.name()
//         )))
//     }
//
//     fn call_extension(
//         &self,
//         this: Rc<RefCell<PrimitiveValue>>,
//         function: String,
//         args: RigzArgs,
//     ) -> Result<PrimitiveValue, VMError> {
//         Err(VMError::UnsupportedOperation(format!(
//             "{} does not implement `call_extension`",
//             self.name()
//         )))
//     }
//
//     fn call_mutable_extension(
//         &self,
//         this: Rc<RefCell<PrimitiveValue>>,
//         function: String,
//         args: RigzArgs,
//     ) -> Result<Option<PrimitiveValue>, VMError> {
//         Ok(Some(
//             VMError::UnsupportedOperation(format!(
//                 "{} does not implement `call_mutable_extension`",
//                 self.name()
//             ))
//             .to_value(),
//         ))
//     }
//
//     fn vm_extension(
//         &self,
//         vm: &mut VM,
//         function: String,
//         args: RigzArgs,
//     ) -> Result<PrimitiveValue, VMError> {
//         Err(VMError::UnsupportedOperation(format!(
//             "{} does not implement `vm_extension`",
//             self.name()
//         )))
//     }
//
//     // todo create proc_macro that uses tree-sitter-rigz for syntax highlighting and compile time syntax validation
//     fn trait_definition(&self) -> &'static str;
// }
