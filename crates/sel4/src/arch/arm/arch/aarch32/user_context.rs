//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{newtype_methods, sys, user_context_newtype_ref_methods};

/// Corresponds to `seL4_UserContext`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(pub sys::seL4_UserContext);

    user_context_newtype_ref_methods!(pc, pc_mut);
    user_context_newtype_ref_methods!(sp, sp_mut);
    user_context_newtype_ref_methods!(cpsr, cpsr_mut);
    user_context_newtype_ref_methods!(tpidrurw, tpidrurw_mut);
    user_context_newtype_ref_methods!(tpidruro, tpidruro_mut);

    user_context_newtype_ref_methods!(r0, r0_mut);
    user_context_newtype_ref_methods!(r1, r1_mut);
    user_context_newtype_ref_methods!(r2, r2_mut);
    user_context_newtype_ref_methods!(r3, r3_mut);
    user_context_newtype_ref_methods!(r4, r4_mut);
    user_context_newtype_ref_methods!(r5, r5_mut);
    user_context_newtype_ref_methods!(r6, r6_mut);
    user_context_newtype_ref_methods!(r7, r7_mut);
    user_context_newtype_ref_methods!(r8, r8_mut);
    user_context_newtype_ref_methods!(r9, r9_mut);
    user_context_newtype_ref_methods!(r10, r10_mut);
    user_context_newtype_ref_methods!(r11, r11_mut);
    user_context_newtype_ref_methods!(r12, r12_mut);
    user_context_newtype_ref_methods!(r14, r14_mut);
}
