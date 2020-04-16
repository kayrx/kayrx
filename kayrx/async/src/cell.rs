use std::cell::UnsafeCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct CausalCell<T>(UnsafeCell<T>);

#[derive(Default)]
pub(crate) struct CausalCheck(());

impl<T> CausalCell<T> {
    pub(crate) fn new(data: T) -> CausalCell<T> {
        CausalCell(UnsafeCell::new(data))
    }

    pub(crate) fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*const T) -> R,
    {
        f(self.0.get())
    }

    pub(crate) fn with_unchecked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*const T) -> R,
    {
        f(self.0.get())
    }

    pub(crate) fn check(&self) {}

    pub(crate) fn with_deferred<F, R>(&self, f: F) -> (R, CausalCheck)
    where
        F: FnOnce(*const T) -> R,
    {
        (f(self.0.get()), CausalCheck::default())
    }

    pub(crate) fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut T) -> R,
    {
        f(self.0.get())
    }
}

impl CausalCheck {
    pub(crate) fn check(self) {}

    pub(crate) fn join(&mut self, _other: CausalCheck) {}
}


/// Custom cell impl

pub(crate) struct Cell<T> {
    pub(crate) inner: Rc<UnsafeCell<T>>,
}

impl<T> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Cell<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(UnsafeCell::new(inner)),
        }
    }

    pub(crate) fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    pub(crate) fn get_ref(&self) -> &T {
        unsafe { &*self.inner.as_ref().get() }
    }

    pub(crate) fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.as_ref().get() }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut_unsafe(&self) -> &mut T {
        &mut *self.inner.as_ref().get()
    }
}