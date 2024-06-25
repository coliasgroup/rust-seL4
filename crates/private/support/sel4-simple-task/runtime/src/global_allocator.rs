//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeapBounds};
use sel4_sync::{GenericRawMutex, MutexSyncOps};

use crate::{get_static_heap_bounds, get_static_heap_mutex_notification};

#[global_allocator]
#[allow(clippy::type_complexity)]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    GenericRawMutex<MutexSyncOpsImpl>,
    fn() -> StaticHeapBounds,
> = StaticDlmallocGlobalAlloc::new(
    GenericRawMutex::new(MutexSyncOpsImpl),
    get_static_heap_bounds,
);

struct MutexSyncOpsImpl;

impl MutexSyncOps for MutexSyncOpsImpl {
    fn signal(&self) {
        get_static_heap_mutex_notification().signal();
    }

    fn wait(&self) {
        get_static_heap_mutex_notification().wait();
    }
}
