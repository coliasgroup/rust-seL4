//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(used_with_arg)]

mod channel;
mod defer;
mod handler;
mod message;
mod symbols;

#[doc(hidden)]
pub mod ipc;

pub use channel::{Channel, IrqAckError, ProtectionDomain};
pub use defer::{DeferredAction, DeferredActionInterface, DeferredActionSlot};
pub use handler::{Handler, Infallible, Never, NullHandler};
pub use message::{
    get_mr, set_mr, with_msg_bytes, with_msg_bytes_mut, with_msg_regs, with_msg_regs_mut,
    MessageInfo, MessageLabel, MessageRegisterValue,
};
pub use symbols::{ipc_buffer_ptr, pd_is_passive, pd_name};

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4_immutable_cell::ImmutableCell;
}
