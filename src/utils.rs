use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;

const HALF_SECOND: Duration = Duration::from_millis(500);

// TODO: It is generally a bad idea to use spin locks.
// These will need to be replaced at some point, but this will get us off the ground.
pub fn get_read_spin_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<T> {
    loop {
        if let Ok(inner) = lock.try_read() {
            return inner;
        }
        sleep(HALF_SECOND)
    }
}

// TODO: See the issues with `get_read_spin_lock`.
pub fn get_write_spin_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<T> {
    loop {
        if let Ok(inner) = lock.try_write() {
            return inner;
        }
        sleep(HALF_SECOND)
    }
}
