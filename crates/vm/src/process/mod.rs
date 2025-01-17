use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "threaded")]
mod threaded;

#[cfg(feature = "threaded")]
pub type ModulesMap =
    std::sync::Arc<dashmap::DashMap<&'static str, std::sync::Arc<dyn Module + Send + Sync>>>;

#[cfg(feature = "threaded")]
pub type Reference<T> = std::sync::Arc<T>;

#[cfg(not(feature = "threaded"))]
pub type Reference<T> = std::rc::Rc<T>;

#[cfg(feature = "threaded")]
pub(crate) type Process = threaded::Process;

#[cfg(not(feature = "threaded"))]
pub type ModulesMap = std::collections::HashMap<&'static str, std::rc::Rc<dyn Module>>;

mod process_manager;
#[cfg(not(feature = "threaded"))]
mod single;

pub(crate) use process_manager::ProcessManager;
use rigz_core::Module;

#[cfg(not(feature = "threaded"))]
pub type Process = single::Process;

#[cfg(not(feature = "threaded"))]
pub type Reference<T> = std::rc::Rc<T>;

#[derive(Debug)]
pub struct MutableReference<T: Debug>(
    #[cfg(feature = "threaded")] std::sync::Arc<std::sync::RwLock<T>>,
    #[cfg(not(feature = "threaded"))] std::rc::Rc<std::cell::RefCell<T>>,
);

impl<T: Debug> Clone for MutableReference<T> {
    fn clone(&self) -> Self {
        MutableReference(self.0.clone())
    }
}

impl<T: Debug> From<T> for MutableReference<T> {
    fn from(t: T) -> Self {
        #[cfg(feature = "threaded")]
        {
            MutableReference(std::sync::Arc::new(std::sync::RwLock::new(t)))
        }
        #[cfg(not(feature = "threaded"))]
        {
            MutableReference(std::rc::Rc::new(std::cell::RefCell::new(t)))
        }
    }
}

impl<T: Debug> MutableReference<T> {
    #[cfg(feature = "threaded")]
    pub fn apply<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self.0.read().expect("failed to obtain RwLock").deref())
    }

    #[cfg(feature = "threaded")]
    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self.0.write().expect("failed to obtain RwLock").deref_mut())
    }

    #[cfg(feature = "threaded")]
    pub fn update_with_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T, MutableReference<T>) -> R,
    {
        let s = self.clone();
        f(
            self.0.write().expect("failed to obtain RwLock").deref_mut(),
            s,
        )
    }

    #[cfg(not(feature = "threaded"))]
    pub fn apply<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self.0.borrow().deref())
    }

    #[cfg(not(feature = "threaded"))]
    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self.0.borrow_mut().deref_mut())
    }

    #[cfg(not(feature = "threaded"))]
    pub fn update_with_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T, MutableReference<T>) -> R,
    {
        let s = self.clone();
        f(self.0.borrow_mut().deref_mut(), s)
    }
}
