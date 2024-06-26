//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(never_type)]
#![cfg_attr(feature = "std", feature(restricted_std))]
#![cfg_attr(feature = "std", feature(lang_items))]
#![cfg_attr(feature = "std", feature(panic_unwind))]
#![cfg_attr(feature = "std", allow(internal_features))]

#[cfg(all(feature = "std", any(feature = "unwinding", feature = "sel4-panicking")))]
compile_error!("");

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
extern crate unwind;

pub use sel4_panicking_env::abort;
pub use sel4_root_task_macros::root_task;

#[doc(inline)]
#[cfg(not(feature = "std"))]
pub use sel4_panicking as panicking;

mod entry;
mod heap;
mod printing;
mod termination;

pub use heap::set_global_allocator_mutex_notification;
pub use termination::{Never, Termination};

#[sel4::sel4_cfg(PRINTING)]
pub use sel4_panicking_env::{debug_print, debug_println};

#[macro_export]
macro_rules! declare_root_task {
    {
        main = $main:expr $(,)?
    } => {
        $crate::_private::declare_root_task! {
            main = $main,
            stack_size = $crate::_private::DEFAULT_STACK_SIZE,
        }
    };
    {
        main = $main:expr,
        stack_size = $stack_size:expr $(,)?
    } => {
        $crate::_private::declare_main!($main);
        $crate::_private::declare_stack!($stack_size);
    };
    {
        main = $main:expr,
        $(stack_size = $stack_size:expr,)?
        heap_size = $heap_size:expr $(,)?
    } => {
        $crate::_private::declare_heap!($heap_size);
        $crate::_private::declare_root_task! {
            main = $main,
            $(stack_size = $stack_size,)?
        }
    };
}

#[doc(hidden)]
pub const DEFAULT_STACK_SIZE: usize = 0x10000;

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4::BootInfoPtr;
    pub use sel4_runtime_common::declare_stack;

    pub use crate::heap::_private as heap;

    pub use crate::{
        declare_heap, declare_main, declare_root_task, entry::run_main, DEFAULT_STACK_SIZE,
    };
}
