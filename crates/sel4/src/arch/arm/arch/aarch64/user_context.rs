//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
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
    user_context_newtype_ref_methods!(spsr, spsr_mut);
    user_context_newtype_ref_methods!(tpidr_el0, tpidr_el0_mut);
    user_context_newtype_ref_methods!(tpidrro_el0, tpidrro_el0_mut);

    user_context_newtype_ref_methods!(x0, x0_mut);
    user_context_newtype_ref_methods!(x1, x1_mut);
    user_context_newtype_ref_methods!(x2, x2_mut);
    user_context_newtype_ref_methods!(x3, x3_mut);
    user_context_newtype_ref_methods!(x4, x4_mut);
    user_context_newtype_ref_methods!(x5, x5_mut);
    user_context_newtype_ref_methods!(x6, x6_mut);
    user_context_newtype_ref_methods!(x7, x7_mut);
    user_context_newtype_ref_methods!(x8, x8_mut);
    user_context_newtype_ref_methods!(x9, x9_mut);
    user_context_newtype_ref_methods!(x10, x10_mut);
    user_context_newtype_ref_methods!(x11, x11_mut);
    user_context_newtype_ref_methods!(x12, x12_mut);
    user_context_newtype_ref_methods!(x13, x13_mut);
    user_context_newtype_ref_methods!(x14, x14_mut);
    user_context_newtype_ref_methods!(x15, x15_mut);
    user_context_newtype_ref_methods!(x16, x16_mut);
    user_context_newtype_ref_methods!(x17, x17_mut);
    user_context_newtype_ref_methods!(x18, x18_mut);
    user_context_newtype_ref_methods!(x19, x19_mut);
    user_context_newtype_ref_methods!(x20, x20_mut);
    user_context_newtype_ref_methods!(x21, x21_mut);
    user_context_newtype_ref_methods!(x22, x22_mut);
    user_context_newtype_ref_methods!(x23, x23_mut);
    user_context_newtype_ref_methods!(x24, x24_mut);
    user_context_newtype_ref_methods!(x25, x25_mut);
    user_context_newtype_ref_methods!(x26, x26_mut);
    user_context_newtype_ref_methods!(x27, x27_mut);
    user_context_newtype_ref_methods!(x28, x28_mut);
    user_context_newtype_ref_methods!(x29, x29_mut);
    user_context_newtype_ref_methods!(x30, x30_mut);
}
