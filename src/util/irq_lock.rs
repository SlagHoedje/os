use core::ops::{Deref, DerefMut};

use spin::{Mutex, MutexGuard};

use x86_64::instructions::interrupts;

/// A lock struct that disables interrupts before it locks the contents.
pub struct IrqLock<T: ?Sized> {
    data: Mutex<T>,
}

impl<T> IrqLock<T> {
    /// Creates a new 'IrqLock' with contents 'data'
    pub const fn new(data: T) -> IrqLock<T> {
        IrqLock {
            data: Mutex::new(data),
        }
    }
}

impl<T: ?Sized> IrqLock<T> {
    /// Locks the data and disables the interrupts
    pub fn lock(&self) -> IrqLockGuard<T> {
        let guard = IrqLockGuard {
            interrupts_enabled: interrupts::are_enabled(),
            data: self.data.lock(),
        };
        interrupts::disable();
        guard
    }

    /// Force unlock the data
    ///
    /// # Safety
    /// If used in an uncontrolled environment, this operation will violate rust data safety by
    /// allowing more than one mutable reference to point at the data at one time!
    pub unsafe fn force_unlock(&self) {
        self.data.force_unlock();
    }
}

/// A guard around the data obtained from 'IrqLock'. When dropped, the data is unlocked.
pub struct IrqLockGuard<'a, T: ?Sized + 'a> {
    interrupts_enabled: bool,
    data: MutexGuard<'a, T>,
}

impl<'a, T: ?Sized> Drop for IrqLockGuard<'a, T> {
    fn drop(&mut self) {
        if self.interrupts_enabled {
            interrupts::enable();
        }
    }
}

impl<'a, T: ?Sized> Deref for IrqLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.data
    }
}

impl <'a, T: ?Sized> DerefMut for IrqLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}