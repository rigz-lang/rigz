use crate::{Number, RigzType, VMError};
use indexmap::IndexMap;
use std::fmt::{Debug, Display};

pub trait WithTypeInfo {
    fn rigz_type(&self) -> RigzType;
}

pub trait AsPrimitive<T: Clone + AsPrimitive<T> + Default + Sized>:
    Display + Debug + WithTypeInfo
{
    fn reverse(&self) -> Result<T, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot reverse {self}"
        )))
    }

    fn as_list(&mut self) -> Result<&mut Vec<T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut List"
        )))
    }

    fn to_list(&self) -> Result<Vec<T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to List"
        )))
    }

    fn to_map(&self) -> Result<IndexMap<T, T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to Map"
        )))
    }

    fn as_map(&mut self) -> Result<&mut IndexMap<T, T>, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Map"
        )))
    }

    fn to_number(&self) -> Result<Number, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to Number"
        )))
    }

    fn as_number(&mut self) -> Result<&mut Number, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Number"
        )))
    }

    fn to_bool(&self) -> bool {
        true
    }

    fn as_bool(&mut self) -> Result<&mut bool, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Bool"
        )))
    }

    fn as_string(&mut self) -> Result<&mut String, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut String"
        )))
    }

    fn to_float(&self) -> Result<f64, VMError> {
        Ok(self.to_number()?.to_float())
    }

    fn to_usize(&self) -> Result<usize, VMError> {
        self.to_number()?.to_usize()
    }

    fn to_int(&self) -> Result<i64, VMError> {
        Ok(self.to_number()?.to_int())
    }

    fn as_float(&mut self) -> Result<&mut f64, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Float"
        )))
    }

    fn as_int(&mut self) -> Result<&mut i64, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot convert {self:?} to mut Int"
        )))
    }

    fn get(&self, attr: &T) -> Result<T, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot get {attr} from {self:?}"
        )))
    }

    fn set(&mut self, attr: &T, value: T) -> Result<(), VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot update {attr} on {self:?} - {value}"
        )))
    }

    fn get_mut(&self, attr: &T) -> Result<&mut T, VMError> {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot get_mut {attr} from {self:?}"
        )))
    }
}
