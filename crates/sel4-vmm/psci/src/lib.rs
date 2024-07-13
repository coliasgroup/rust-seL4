//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub const VERSION: i32 = 0x0001_0001;

pub mod ret {
    pub const SUCCESS: i32 = 0;
    pub const NOT_SUPPORTED: i32 = -1;
    pub const INVALID_PARAMETERS: i32 = -2;
    pub const DEFINED: i32 = -3;
    pub const ALREADY_ON: i32 = -4;
    pub const ON_PENDING: i32 = -5;
    pub const INTERNAL_FAILURE: i32 = -6;
    pub const NOT_PRESENT: i32 = -7;
    pub const DISABLED: i32 = -8;
    pub const INVALID_ADDRESS: i32 = -9;
}

pub mod fid {
    use super::fid_for_native_bit_width;

    pub const PSCI_VERSION: u32 = 0x8400_0000;
    pub const CPU_ON: u32 = fid_for_native_bit_width(0x8400_0003);
    pub const MIGRATE_INFO_TYPE: u32 = 0x8400_0006;
    pub const PSCI_FEATURES: u32 = 0x8400_000a;
}

const fn fid_for_native_bit_width(fid_32: u32) -> u32 {
    let mask = if cfg!(target_pointer_width = "64") {
        0x4000_0000
    } else {
        0
    };
    fid_32 | mask
}

pub enum Call {
    Version,
    Features {
        qfid: u32,
    },
    CpuOn {
        target_cpu: usize,
        entry_point_address: usize,
        context_id: usize,
    },
    MigrateInfoType,
}

impl Call {
    pub const MAX_NUM_ARGS: usize = 5;

    pub fn parse(args: &[usize; Self::MAX_NUM_ARGS]) -> Result<Self, CallParseError> {
        let fid = args[0] as u32;
        Ok(match fid {
            fid::PSCI_VERSION => Self::Version,
            fid::PSCI_FEATURES => Self::Features {
                qfid: args[1] as u32,
            },
            fid::CPU_ON => Self::CpuOn {
                target_cpu: args[1],
                entry_point_address: args[2],
                context_id: args[3],
            },
            fid::MIGRATE_INFO_TYPE => Self::MigrateInfoType,
            _ => return Err(CallParseError::UnrecognizedFid { fid }),
        })
    }
}

#[derive(Debug)]
pub enum CallParseError {
    UnrecognizedFid { fid: u32 },
}
