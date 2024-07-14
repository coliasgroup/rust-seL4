//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::VmFault;

use crate::{inner_decls, self_impl};

self_impl!(VmFaultExt, VmFault);

pub trait VmFaultExt {
    inner_decls!(VmFault);
}
