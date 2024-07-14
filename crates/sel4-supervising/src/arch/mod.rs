//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(ARCH_AARCH64)] {
        #[path = "aarch64.rs"]
        mod imp;
    } else if #[sel4_cfg(ARCH_AARCH32)] {
        #[path = "aarch32.rs"]
        mod imp;
    } else if #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        #[path = "riscv.rs"]
        mod imp;
    } else if #[sel4_cfg(ARCH_X86_64)] {
        #[path = "x86_64.rs"]
        mod imp;
    }
}

#[cfg(any())]
mod aarch32;
#[cfg(any())]
mod aarch64;
#[cfg(any())]
mod riscv;
#[cfg(any())]
mod x86_64;

pub use imp::*;
