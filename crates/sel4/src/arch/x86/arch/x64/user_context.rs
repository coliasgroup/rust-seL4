//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{newtype_methods, sys, user_context_newtype_ref_methods};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(pub sys::seL4_UserContext);

    user_context_newtype_ref_methods!(rip, rip_mut);
    user_context_newtype_ref_methods!(rsp, rsp_mut);
    user_context_newtype_ref_methods!(rflags, rflags_mut);
    user_context_newtype_ref_methods!(fs_base, fs_base_mut);
    user_context_newtype_ref_methods!(gs_base, gs_base_mut);
    user_context_newtype_ref_methods!(rax, rax_mut);
    user_context_newtype_ref_methods!(rbx, rbx_mut);
    user_context_newtype_ref_methods!(rcx, rcx_mut);
    user_context_newtype_ref_methods!(rdx, rdx_mut);
    user_context_newtype_ref_methods!(rsi, rsi_mut);
    user_context_newtype_ref_methods!(rdi, rdi_mut);
    user_context_newtype_ref_methods!(rbp, rbp_mut);
    user_context_newtype_ref_methods!(r8, r8_mut);
    user_context_newtype_ref_methods!(r9, r9_mut);
    user_context_newtype_ref_methods!(r10, r10_mut);
    user_context_newtype_ref_methods!(r11, r11_mut);
    user_context_newtype_ref_methods!(r12, r12_mut);
    user_context_newtype_ref_methods!(r13, r13_mut);
    user_context_newtype_ref_methods!(r14, r14_mut);
    user_context_newtype_ref_methods!(r15, r15_mut);
}
