//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

//! This crate provides straightforward, pure-Rust bindings to the [seL4
//! API](https://sel4.systems/Info/Docs/seL4-manual-latest.pdf).
//!
//! Most items in this crate correspond to types, constants, and functions in
//! [libsel4](https://docs.sel4.systems/projects/sel4/api-doc.html). Notably, when applicable,
//! `seL4_CPtr` is agumented in [`Cap`] with a marker specifying the type of capability it points
//! to.
//!
//! This crate's implementation is based on the lower-level [`sel4-sys`](::sel4_sys) crate, which is
//! generated from the libsel4 headers and interface definition files.
//!
//! ### Features
//!
//! #### `"state"`
//!
//! Functions in the C libsel4 use the thread-local variable `__sel4_ipc_buffer` to obtain a pointer
//! to the current thread's IPC buffer:
//!
//! ```C
//! extern __thread seL4_IPCBuffer *__sel4_ipc_buffer;
//! ```
//!
//! [libmicrokit](https://github.com/seL4/microkit/tree/main/libmicrokit), which does not support
//! thread-local storage uses the following snippet to force `__sel4_ipc_buffer` to global rather
//! than thread-local:
//!
//! ```C
//! #define __thread
//! #include <sel4/sel4.h>
//! ```
//!
//! For the sake of flexibility and applicability, this crate can be configured to use no state at
//! all. Users can opt out of state and explicitly pass around references to the active IPC buffer
//! instead of relying on the implementation to obtain such a reference using thread-local or global
//! state. Such a paradigm is useful in certain uncommon circumstances, but most users will benefit
//! from the convenience of an implicit IPC buffer. The `"state"` feature, enabled by default, uses
//! state to allow one to make seL4 API calls without having to explicitly specify an IPC buffer.
//!
//! For the sake of interoperability with C, the state looks something like: `static mut
//! __sel4_ipc_buffer: *mut IpcBuffer`. If the `"state-exposed"` feature is enabled, it is exposed
//! with `#![no_mangle]`. If the `"state-extern"` feature is enabled, it is wrapped in an `extern
//! "C"` block. Whether it is thread-local is determined by the following pseudocode:
//!
//! ```rust
//! cfg_if! {
//!     if #[cfg(all(any(target_thread_local, feature = "tls"), not(feature = "non-thread-local-state")))] {
//!         // thread-local
//!     } else if #[cfg(not(feature = "thread-local-state"))] {
//!         // not thread-local
//!     } else {
//!         compile_error!(r#"invalid configuration"#);
//!     }
//! }
//! ```
//!
//! The non-thread-local configuration should only be used in cases where the language runtime does
//! not support thread-local storage. In those cases without thread-local storage where this crate
//! will only ever run in a single thread, use the `"single-threaded"` feature to enable a more
//! efficient implementation. Note that enabling the `"single-threaded"` feature in a case where
//! this crate runs in more than one thread is unsafe.
//!
//! At the API level, the `"state"` feature causes [`NoExplicitInvocationContext`] to be an alias
//! for [`ImplicitInvocationContext`], which implements [`InvocationContext`] by accessing the
//! thread-local pointer to an IPC buffer. When the `"state"` feature is not enabled,
//! [`NoExplicitInvocationContext`] is an alias for [`NoInvocationContext`], which does not
//! implement [`InvocationContext`]. The thread-local IPC buffer pointer is modified and accessed by
//! [`set_ipc_buffer`], [`with_ipc_buffer`], and [`with_ipc_buffer_mut`]. The lower-level
//! [`try_with_ipc_buffer_slot`] and [`try_with_ipc_buffer_slot_mut`] are provided as well.
//!
//! ### Building
//!
//! This crate and its dependencies depend, at build time, on the libsel4 headers. The location of
//! these headers is provided to this crate at build time by environment variables. If
//! `SEL4_INCLUDE_DIRS` is set, then its value is interpreted as a colon-separated list of include
//! paths for the libsel4 headers. Otherwise, if `SEL4_PREFIX` is set, then
//! `$SEL4_PREFIX/libsel4/include` is used.

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]
#![allow(clippy::unit_arg)]

pub use sel4_config::{
    self as config, sel4_cfg, sel4_cfg_attr, sel4_cfg_bool, sel4_cfg_enum, sel4_cfg_if,
    sel4_cfg_match, sel4_cfg_str, sel4_cfg_struct, sel4_cfg_usize, sel4_cfg_word,
    sel4_cfg_wrap_match,
};

pub use sel4_sys as sys;

mod arch;
mod bootinfo;
mod cap_rights;
mod cnode_cap_data;
mod const_helpers;
mod cptr;
mod error;
mod fault;
mod helper_macros;
mod invocation_context;
mod invocations;
mod ipc_buffer;
mod message_info;
mod object;
mod reply_authority;
mod syscalls;
mod vspace;

pub mod init_thread;

pub use bootinfo::{
    BootInfo, BootInfoExtra, BootInfoExtraId, BootInfoExtraIter, BootInfoPtr, UntypedDesc,
};
pub use cap_rights::{CapRights, CapRightsBuilder};
pub use cnode_cap_data::CNodeCapData;
pub use cptr::{
    cap, cap_type, AbsoluteCPtr, CPtr, CPtrBits, CPtrWithDepth, Cap, CapType, HasCPtrWithDepth,
};
pub use error::{Error, Result};
pub use fault::*;
pub use invocation_context::{InvocationContext, NoExplicitInvocationContext, NoInvocationContext};
pub use ipc_buffer::IpcBuffer;
pub use message_info::{MessageInfo, MessageInfoBuilder};
pub use object::{
    CapTypeForObject, CapTypeForObjectOfFixedSize, CapTypeForObjectOfVariableSize, ObjectBlueprint,
    ObjectType,
};
pub use reply_authority::{ConveysReplyAuthority, ReplyAuthority};
pub use syscalls::{
    r#yield, Badge, CallWithMRs, FastMessages, IpcCapType, RecvWithMRs, NUM_MESSAGE_REGISTERS,
};
pub use vspace::{
    vspace_levels, CapTypeForFrameObject, CapTypeForFrameObjectOfFixedSize,
    CapTypeForTranslationTableObject,
};

sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        pub use invocations::Time;
    } else {
        pub use syscalls::reply;
        pub use reply_authority::ImplicitReplyAuthority;
    }
}

#[sel4_cfg(SET_TLS_BASE_SELF)]
pub use syscalls::set_tls_base;

pub use arch::top_level::*;

pub(crate) use helper_macros::{
    declare_cap_alias, declare_cap_type, declare_cap_type_for_object,
    declare_cap_type_for_object_of_fixed_size, declare_cap_type_for_object_of_variable_size,
    declare_fault_ipc_buffer_newtype, declare_fault_newtype, fault_ipc_buffer_newtype_ref_methods,
    fault_newtype_getter_method, newtype_methods, user_context_newtype_ref_methods,
};

sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        mod printing;

        pub use printing::{DebugWrite, debug_put_char};
    }
}

sel4_cfg_if! {
    if #[sel4_cfg(DEBUG_BUILD)] {
        mod debug;

        pub use debug::debug_snapshot;
    }
}

sel4_cfg_if! {
    if #[sel4_cfg(ENABLE_BENCHMARKS)] {
        mod benchmark;

        pub use benchmark::{
            benchmark_reset_log,
            benchmark_finalize_log,
            benchmark_set_log_buffer,
        };

        sel4_cfg_if! {
            if #[sel4_cfg(BENCHMARK_TRACK_UTILISATION)] {
                pub use benchmark::{
                    benchmark_get_thread_utilisation,
                    benchmark_reset_thread_utilisation,
                };

                #[sel4_cfg(DEBUG_BUILD)]
                pub use benchmark::{
                    benchmark_dump_all_thread_utilisation,
                    benchmark_reset_all_thread_utilisation,
                };
            }
        }
    }
}

#[cfg(feature = "state")]
mod state;

#[cfg(feature = "state")]
pub use state::{
    ipc_buffer_is_thread_local, set_ipc_buffer, try_with_ipc_buffer_slot,
    try_with_ipc_buffer_slot_mut, with_ipc_buffer, with_ipc_buffer_mut, ImplicitInvocationContext,
};

/// Corresponds to `seL4_Word`.
pub type Word = sys::seL4_Word;

/// The size of [`Word`] in bits.
pub const WORD_SIZE: usize = sel4_cfg_usize!(WORD_SIZE);

#[doc(hidden)]
pub mod _private {
    #[sel4_config::sel4_cfg(PRINTING)]
    pub use super::printing::_private as printing;
}
