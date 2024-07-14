//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    fault_ipc_buffer_newtype_ref_methods, fault_newtype_getter_method, sys, UnknownSyscall,
    UnknownSyscallInIpcBuffer, Word,
};

impl UnknownSyscall {
    fault_newtype_getter_method!(spsr, get_SPSR);
    fault_newtype_getter_method!(x0, get_X0);
    fault_newtype_getter_method!(x1, get_X1);
    fault_newtype_getter_method!(x2, get_X2);
    fault_newtype_getter_method!(x3, get_X3);
    fault_newtype_getter_method!(x4, get_X4);
    fault_newtype_getter_method!(x5, get_X5);
    fault_newtype_getter_method!(x6, get_X6);
    fault_newtype_getter_method!(x7, get_X7);
}

impl<'a> UnknownSyscallInIpcBuffer<'a> {
    fault_ipc_buffer_newtype_ref_methods!(
        x0,
        x0_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X0
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x1,
        x1_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X1
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x2,
        x2_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X2
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x3,
        x3_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X3
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x4,
        x4_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X4
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x5,
        x5_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X5
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x6,
        x6_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X6
    );

    fault_ipc_buffer_newtype_ref_methods!(
        x7,
        x7_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X7
    );
}
