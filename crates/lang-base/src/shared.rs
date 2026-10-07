// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Short-cut definition of `Rc<std::cell::RefCell<T>>` and `Rc<T>`

use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// An immutable, ref-counted object shared safely across contexts and threads.
#[repr(transparent)]
#[derive(Debug)]
pub struct Shared<T>(Arc<T>);

impl<T> Shared<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }

    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        Arc::get_mut(&mut this.0)
    }

    #[inline]
    pub fn make_mut(this: &mut Self) -> &mut T
    where
        T: Clone,
    {
        Arc::make_mut(&mut this.0)
    }
}

impl<T> std::ops::Deref for Shared<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> From<T> for Shared<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

/// An object that can be safely shared and mutated across contexts and threads.
#[repr(transparent)]
#[derive(Debug)]
pub struct SharedMut<T>(Arc<RwLock<T>>);

impl<T> SharedMut<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(RwLock::new(val)))
    }

    #[inline]
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.0.read()
    }

    #[inline]
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.0.write()
    }

    #[inline]
    pub fn read_unwrap(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().expect("Shared lock poisoned on read")
    }

    #[inline]
    pub fn write_unwrap(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().expect("Shared lock poisoned on write")
    }

    #[inline]
    pub fn with_read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.read_unwrap();
        f(&*guard)
    }

    #[inline]
    pub fn with_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.write_unwrap();
        f(&mut *guard)
    }
}

impl<T> Clone for SharedMut<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> From<T> for SharedMut<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T: Default> Default for SharedMut<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<Arc<RwLock<T>>> for SharedMut<T> {
    fn from(arc: Arc<RwLock<T>>) -> Self {
        Self(arc)
    }
}

impl<T> From<SharedMut<T>> for Arc<RwLock<T>> {
    fn from(shared: SharedMut<T>) -> Self {
        shared.0
    }
}

impl<T> SharedMut<T> {
    /// Returns true if two `Shared` pointers point to the same allocation.
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Arc::ptr_eq(&this.0, &other.0)
    }
}
