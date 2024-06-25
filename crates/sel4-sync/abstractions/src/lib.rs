//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};

pub use lock_api;

pub struct PanickingRawMutex {
    locked: AtomicBool,
}

unsafe impl lock_api::RawMutex for PanickingRawMutex {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };

    fn lock(&self) {
        if !self.try_lock() {
            panic!("lock contention")
        }
    }

    fn try_lock(&self) -> bool {
        let was_locked = self.locked.swap(true, Ordering::Acquire);
        !was_locked
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release)
    }
}
