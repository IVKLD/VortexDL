use std::sync::{Mutex, MutexGuard};

pub trait MutexExt<T> {
    fn lock_unpoisoned(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    #[inline]
    fn lock_unpoisoned(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}
