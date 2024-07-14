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
    fault_newtype_getter_method!(cpsr, get_CPSR);
    fault_newtype_getter_method!(r0, get_R0);
    fault_newtype_getter_method!(r1, get_R1);
    fault_newtype_getter_method!(r2, get_R2);
    fault_newtype_getter_method!(r3, get_R3);
    fault_newtype_getter_method!(r4, get_R4);
    fault_newtype_getter_method!(r5, get_R5);
    fault_newtype_getter_method!(r6, get_R6);
    fault_newtype_getter_method!(r7, get_R7);
}

impl<'a> UnknownSyscallInIpcBuffer<'a> {
    fault_ipc_buffer_newtype_ref_methods!(
        r0,
        r0_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R0
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r1,
        r1_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R1
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r2,
        r2_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R2
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r3,
        r3_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R3
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r4,
        r4_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R4
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r5,
        r5_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R5
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r6,
        r6_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R6
    );

    fault_ipc_buffer_newtype_ref_methods!(
        r7,
        r7_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R7
    );
}
