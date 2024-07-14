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

impl CapFault {
    // TODO
}

impl UnknownSyscall {
    fault_newtype_getter_method!(fault_ip, get_FaultIP);
    fault_newtype_getter_method!(sp, get_SP);
    fault_newtype_getter_method!(ra, get_RA);
    fault_newtype_getter_method!(syscall, get_Syscall);
    fault_newtype_getter_method!(a0, get_A0);
    fault_newtype_getter_method!(a1, get_A1);
    fault_newtype_getter_method!(a2, get_A2);
    fault_newtype_getter_method!(a3, get_A3);
    fault_newtype_getter_method!(a4, get_A4);
    fault_newtype_getter_method!(a5, get_A5);
    fault_newtype_getter_method!(a6, get_A6);
}

impl<'a> UnknownSyscallInIpcBuffer<'a> {
    fault_ipc_buffer_newtype_ref_methods!(
        fault_ip,
        fault_ip_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a0,
        a0_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A0
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a1,
        a1_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A1
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a2,
        a2_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A2
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a3,
        a3_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A3
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a4,
        a4_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A4
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a5,
        a5_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A5
    );

    fault_ipc_buffer_newtype_ref_methods!(
        a6,
        a6_mut,
        sys::seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A6
    );
}

impl UserException {
    // TODO
}

impl VmFault {
    fault_newtype_getter_method!(ip, get_IP);
    fault_newtype_getter_method!(addr, get_Addr);
    fault_newtype_getter_method!(fsr, get_FSR);

    pub fn is_prefetch(&self) -> bool {
        self.inner().get_PrefetchFault() != 0
    }
}
