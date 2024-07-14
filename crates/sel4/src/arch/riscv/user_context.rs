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
    user_context_newtype_ref_methods!(ra, ra_mut);
    user_context_newtype_ref_methods!(sp, sp_mut);
    user_context_newtype_ref_methods!(gp, gp_mut);
    user_context_newtype_ref_methods!(tp, tp_mut);
    user_context_newtype_ref_methods!(s0, s0_mut);
    user_context_newtype_ref_methods!(s1, s1_mut);
    user_context_newtype_ref_methods!(s2, s2_mut);
    user_context_newtype_ref_methods!(s3, s3_mut);
    user_context_newtype_ref_methods!(s4, s4_mut);
    user_context_newtype_ref_methods!(s5, s5_mut);
    user_context_newtype_ref_methods!(s6, s6_mut);
    user_context_newtype_ref_methods!(s7, s7_mut);
    user_context_newtype_ref_methods!(s8, s8_mut);
    user_context_newtype_ref_methods!(s9, s9_mut);
    user_context_newtype_ref_methods!(s10, s10_mut);
    user_context_newtype_ref_methods!(s11, s11_mut);
    user_context_newtype_ref_methods!(a0, a0_mut);
    user_context_newtype_ref_methods!(a1, a1_mut);
    user_context_newtype_ref_methods!(a2, a2_mut);
    user_context_newtype_ref_methods!(a3, a3_mut);
    user_context_newtype_ref_methods!(a4, a4_mut);
    user_context_newtype_ref_methods!(a5, a5_mut);
    user_context_newtype_ref_methods!(a6, a6_mut);
    user_context_newtype_ref_methods!(a7, a7_mut);
    user_context_newtype_ref_methods!(t0, t0_mut);
    user_context_newtype_ref_methods!(t1, t1_mut);
    user_context_newtype_ref_methods!(t2, t2_mut);
    user_context_newtype_ref_methods!(t3, t3_mut);
    user_context_newtype_ref_methods!(t4, t4_mut);
    user_context_newtype_ref_methods!(t5, t5_mut);
    user_context_newtype_ref_methods!(t6, t6_mut);
}
