//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::sync::atomic::{fence, AtomicIsize, Ordering};

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub struct GenericRawMutex<O> {
    sync_ops: O,
    value: AtomicIsize,
}

impl<O> GenericRawMutex<O> {
    pub const fn new(sync_ops: O) -> Self {
        Self {
            sync_ops,
            value: AtomicIsize::new(1),
        }
    }
}

impl<O: MutexSyncOpsWithInteriorMutability> GenericRawMutex<O> {
    pub fn modify(&self, input: O::ModifyInput) -> O::ModifyOutput {
        self.sync_ops.modify(input)
    }
}

unsafe impl<O: MutexSyncOps> lock_api::RawMutex for GenericRawMutex<O> {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    const INIT: Self = unimplemented!();

    fn lock(&self) {
        let old_value = self.value.fetch_sub(1, Ordering::Acquire);
        if old_value <= 0 {
            self.sync_ops.wait();
            fence(Ordering::Acquire);
        }
    }

    fn try_lock(&self) -> bool {
        unimplemented!()
    }

    unsafe fn unlock(&self) {
        let old_value = self.value.fetch_add(1, Ordering::Release);
        if old_value < 0 {
            self.sync_ops.signal();
        }
    }
}

pub trait MutexSyncOps {
    fn signal(&self);
    fn wait(&self);
}

pub trait MutexSyncOpsWithInteriorMutability {
    type ModifyInput;
    type ModifyOutput;

    fn modify(&self, input: Self::ModifyInput) -> Self::ModifyOutput;
}

pub trait MutexSyncOpsWithNotification {
    fn notification(&self) -> sel4::cap::Notification;
}

impl<O: MutexSyncOpsWithNotification> MutexSyncOps for O {
    fn signal(&self) {
        self.notification().signal()
    }

    fn wait(&self) {
        let _badge = self.notification().wait();
    }
}

impl MutexSyncOpsWithNotification for sel4::cap::Notification {
    fn notification(&self) -> sel4::cap::Notification {
        *self
    }
}

pub struct DeferredNotificationMutexSyncOps {
    inner: ImmediateSyncOnceCell<sel4::cap::Notification>,
}

impl DeferredNotificationMutexSyncOps {
    pub const fn new() -> Self {
        Self {
            inner: ImmediateSyncOnceCell::new(),
        }
    }
}

impl Default for DeferredNotificationMutexSyncOps {
    fn default() -> Self {
        Self::new()
    }
}

impl MutexSyncOpsWithNotification for DeferredNotificationMutexSyncOps {
    fn notification(&self) -> sel4::cap::Notification {
        *self.inner.get().unwrap()
    }
}

impl MutexSyncOpsWithInteriorMutability for DeferredNotificationMutexSyncOps {
    type ModifyInput = sel4::cap::Notification;
    type ModifyOutput = ();

    fn modify(&self, input: Self::ModifyInput) -> Self::ModifyOutput {
        self.inner.set(input).unwrap()
    }
}

impl<F: Fn() -> sel4::cap::Notification> MutexSyncOpsWithNotification for F {
    fn notification(&self) -> sel4::cap::Notification {
        (self)()
    }
}

pub struct PanickingMutexSyncOps(());

impl PanickingMutexSyncOps {
    pub const fn new() -> Self {
        Self(())
    }
}

impl Default for PanickingMutexSyncOps {
    fn default() -> Self {
        Self::new()
    }
}

impl MutexSyncOps for PanickingMutexSyncOps {
    fn signal(&self) {
        panic!("unexpected contention: signal")
    }

    fn wait(&self) {
        panic!("unexpected contention: wait")
    }
}
