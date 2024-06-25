//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::abort;

static GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION: ImmediateSyncOnceCell<sel4::cap::Notification> =
    ImmediateSyncOnceCell::new();

pub fn set_global_allocator_mutex_notification(nfn: sel4::cap::Notification) {
    GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
        .set(nfn)
        .unwrap_or_else(|_| abort!("global allocator mutex notification already initialized"))
}

#[doc(hidden)]
pub fn get_global_allocator_mutex_notification() -> sel4::cap::Notification {
    *GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
        .get()
        .unwrap_or_else(|| {
            abort!("global allocator contention before mutex notification initialization")
        })
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_heap {
    ($size:expr) => {
        const _: () = {
            mod outer_scope {
                use super::*;

                const _SIZE: usize = $size;

                mod inner_scope {
                    use $crate::_private::heap::*;

                    use super::_SIZE as SIZE;

                    static STATIC_HEAP: StaticHeap<{ SIZE }> = StaticHeap::new();

                    #[global_allocator]
                    static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
                        GenericRawMutex<fn() -> sel4::cap::Notification>,
                        &'static StaticHeap<{ SIZE }>,
                    > = StaticDlmallocGlobalAlloc::new(
                        GenericRawMutex::new(get_global_allocator_mutex_notification),
                        &STATIC_HEAP,
                    );
                }
            }
        };
    };
}

pub mod _private {
    pub use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
    pub use sel4_sync::GenericRawMutex;

    pub use super::get_global_allocator_mutex_notification;
}
