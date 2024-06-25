//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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

                    static STATIC_HEAP: StaticHeap<{ $size }> = StaticHeap::new();

                    #[global_allocator]
                    static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
                        PanickingRawMutex,
                        &'static StaticHeap<{ $size }>,
                    > = StaticDlmallocGlobalAlloc::new(PanickingRawMutex::new(), &STATIC_HEAP);
                }
            }
        };
    };
}

pub mod _private {
    pub use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
    pub use sel4_sync::PanickingRawMutex;
}
