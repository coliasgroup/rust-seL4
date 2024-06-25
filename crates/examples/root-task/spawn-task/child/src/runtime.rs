//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;

use sel4::CapTypeForFrameObjectOfFixedSize;
use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
use sel4_panicking::catch_unwind;
use sel4_panicking_env::abort;
use sel4_sync::PanickingRawMutex;

use crate::main;

const STACK_SIZE: usize = 1024 * 64;

sel4_runtime_common::declare_stack!(STACK_SIZE);

const HEAP_SIZE: usize = 1024 * 64;

static STATIC_HEAP: StaticHeap<HEAP_SIZE> = StaticHeap::new();

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    PanickingRawMutex,
    &'static StaticHeap<HEAP_SIZE>,
> = StaticDlmallocGlobalAlloc::new(PanickingRawMutex::new(), &STATIC_HEAP);

sel4_panicking_env::register_debug_put_char!(sel4::debug_put_char);

#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    unsafe extern "C" fn cont_fn(_cont_arg: *mut sel4_runtime_common::ContArg) -> ! {
        inner_entry()
    }

    sel4_runtime_common::initialize_tls_on_stack_and_continue(cont_fn, ptr::null_mut())
}

fn inner_entry() -> ! {
    #[cfg(panic = "unwind")]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    unsafe {
        sel4::set_ipc_buffer(get_ipc_buffer().as_mut().unwrap());
        sel4_runtime_common::run_ctors();
    }

    match catch_unwind(main) {
        Ok(never) => never,
        Err(_) => abort!("main() panicked"),
    }
}

fn get_ipc_buffer() -> *mut sel4::IpcBuffer {
    extern "C" {
        static _end: usize;
    }
    unsafe {
        (ptr::addr_of!(_end) as usize)
            .next_multiple_of(sel4::cap_type::Granule::FRAME_OBJECT_TYPE.bytes())
            as *mut sel4::IpcBuffer
    }
}
