//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    declare_fault_ipc_buffer_newtype, declare_fault_newtype, fault_ipc_buffer_newtype_ref_methods,
    fault_newtype_getter_method, sys, Word,
};

declare_fault_newtype!(NullFault, sys::seL4_Fault_NullFault);
declare_fault_newtype!(CapFault, sys::seL4_Fault_CapFault);
declare_fault_newtype!(UnknownSyscall, sys::seL4_Fault_UnknownSyscall);
declare_fault_newtype!(UserException, sys::seL4_Fault_UserException);
declare_fault_newtype!(VmFault, sys::seL4_Fault_VMFault);

#[sel4_cfg(KERNEL_MCS)]
declare_fault_newtype!(Timeout, sys::seL4_Fault_Timeout);

declare_fault_ipc_buffer_newtype!(
    UnknownSyscallInIpcBuffer,
    sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Length
);

#[sel4_cfg_enum]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    NullFault(NullFault),
    CapFault(CapFault),
    UnknownSyscall(UnknownSyscall),
    UserException(UserException),
    VmFault(VmFault),
    #[sel4_cfg(KERNEL_MCS)]
    Timeout(Timeout),
}

impl Fault {
    pub fn from_sys(raw: sys::seL4_Fault) -> Self {
        sel4_cfg_wrap_match! {
            match raw.splay() {
                sys::seL4_Fault_Splayed::NullFault(inner) => {
                    Self::NullFault(NullFault::from_inner(inner))
                }
                sys::seL4_Fault_Splayed::CapFault(inner) => Self::CapFault(CapFault::from_inner(inner)),
                sys::seL4_Fault_Splayed::UnknownSyscall(inner) => {
                    Self::UnknownSyscall(UnknownSyscall::from_inner(inner))
                }
                sys::seL4_Fault_Splayed::UserException(inner) => {
                    Self::UserException(UserException::from_inner(inner))
                }
                sys::seL4_Fault_Splayed::VMFault(inner) => Self::VmFault(VmFault::from_inner(inner)),
                #[sel4_cfg(KERNEL_MCS)]
                sys::seL4_Fault_Splayed::Timeout(inner) => Self::Timeout(Timeout::from_inner(inner)),
            }
        }
    }
}

impl UnknownSyscall {
    fault_newtype_getter_method!(fault_ip, get_FaultIP);
    fault_newtype_getter_method!(rsp, get_RSP);
    fault_newtype_getter_method!(flags, get_FLAGS);
    fault_newtype_getter_method!(syscall, get_Syscall);
    fault_newtype_getter_method!(rax, get_RAX);
    fault_newtype_getter_method!(rbx, get_RBX);
    fault_newtype_getter_method!(rcx, get_RCX);
    fault_newtype_getter_method!(rdx, get_RDX);
    fault_newtype_getter_method!(rsi, get_RSI);
    fault_newtype_getter_method!(rdi, get_RDI);
    fault_newtype_getter_method!(rbp, get_RBP);
    fault_newtype_getter_method!(r8, get_R8);
    fault_newtype_getter_method!(r9, get_R9);
    fault_newtype_getter_method!(r10, get_R10);
    fault_newtype_getter_method!(r11, get_R11);
    fault_newtype_getter_method!(r12, get_R12);
    fault_newtype_getter_method!(r13, get_R13);
    fault_newtype_getter_method!(r14, get_R14);
    fault_newtype_getter_method!(r15, get_R15);
}

impl<'a> UnknownSyscallInIpcBuffer<'a> {
    fault_ipc_buffer_newtype_ref_methods!(
        fault_ip,
        fault_ip_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP
    );
}
