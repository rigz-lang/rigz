use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

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
