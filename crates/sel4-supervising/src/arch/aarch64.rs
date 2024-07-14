//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::{UserContext, VCpuFault, VmFault, Word};

use crate::{inner_decls, self_impl, BitField, VmFaultData, VmFaultWidth};

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
mod esr {
    use super::BitField;

    pub(crate) const EC: BitField = BitField::from_range_inclusive(26..=31);
    pub(crate) const ISV: BitField = BitField::bit(24);
    pub(crate) const SAS: BitField = BitField::from_range_inclusive(22..=23);
    pub(crate) const SSE: BitField = BitField::bit(21);
    pub(crate) const SRT: BitField = BitField::from_range_inclusive(16..=20);
    pub(crate) const SF: BitField = BitField::bit(15);
    pub(crate) const WnR: BitField = BitField::bit(6);
}

self_impl!(VmFaultExt, VmFault);

pub trait VmFaultExt {
    inner_decls!(VmFault);

    fn esr_is_valid(&self) -> bool {
        esr::ISV.get(self.inner().fsr()) == 1
    }

    fn valid_esr(&self) -> Word {
        assert!(self.esr_is_valid());
        self.inner().fsr()
    }

    fn is_write(&self) -> bool {
        esr::WnR.get(self.valid_esr()) == 1
    }

    fn is_read(&self) -> bool {
        !self.is_write()
    }

    fn width(&self) -> VmFaultWidth {
        match esr::SAS.get(self.valid_esr()) {
            0b00 => VmFaultWidth::U8,
            0b01 => VmFaultWidth::U16,
            0b10 => VmFaultWidth::U32,
            0b11 => VmFaultWidth::U64,
            _ => unreachable!(),
        }
    }

    fn is_aligned(&self) -> bool {
        let mask = match self.width() {
            VmFaultWidth::U8 => 0x0,
            VmFaultWidth::U16 => 0x1,
            VmFaultWidth::U32 => 0x3,
            VmFaultWidth::U64 => 0x7,
        };
        self.inner().addr() & mask == 0
    }

    fn data(&self, ctx: &UserContext) -> VmFaultData {
        assert!(self.is_write());
        self.width().truncate(self.register_value(ctx))
    }

    fn emulate_read(&self, ctx: &mut UserContext, val: VmFaultData) {
        assert!(self.is_read());
        assert_eq!(self.width(), val.width());
        let reg = self.register_mut(ctx);
        *reg = val.set(*reg);
    }

    fn register_value(&self, ctx: &UserContext) -> Word {
        match register_index(self.inner()) {
            0 => *ctx.x0(),
            1 => *ctx.x1(),
            2 => *ctx.x2(),
            3 => *ctx.x3(),
            4 => *ctx.x4(),
            5 => *ctx.x5(),
            6 => *ctx.x6(),
            7 => *ctx.x7(),
            8 => *ctx.x8(),
            9 => *ctx.x9(),
            10 => *ctx.x10(),
            11 => *ctx.x11(),
            12 => *ctx.x12(),
            13 => *ctx.x13(),
            14 => *ctx.x14(),
            15 => *ctx.x15(),
            16 => *ctx.x16(),
            17 => *ctx.x17(),
            18 => *ctx.x18(),
            19 => *ctx.x19(),
            20 => *ctx.x20(),
            21 => *ctx.x21(),
            22 => *ctx.x22(),
            23 => *ctx.x23(),
            24 => *ctx.x24(),
            25 => *ctx.x25(),
            26 => *ctx.x26(),
            27 => *ctx.x27(),
            28 => *ctx.x28(),
            29 => *ctx.x29(),
            30 => *ctx.x30(),
            31 => 0,
            _ => panic!(),
        }
    }

    fn register_mut<'a>(&self, ctx: &'a mut UserContext) -> &'a mut Word {
        match register_index(self.inner()) {
            0 => ctx.x0_mut(),
            1 => ctx.x1_mut(),
            2 => ctx.x2_mut(),
            3 => ctx.x3_mut(),
            4 => ctx.x4_mut(),
            5 => ctx.x5_mut(),
            6 => ctx.x6_mut(),
            7 => ctx.x7_mut(),
            8 => ctx.x8_mut(),
            9 => ctx.x9_mut(),
            10 => ctx.x10_mut(),
            11 => ctx.x11_mut(),
            12 => ctx.x12_mut(),
            13 => ctx.x13_mut(),
            14 => ctx.x14_mut(),
            15 => ctx.x15_mut(),
            16 => ctx.x16_mut(),
            17 => ctx.x17_mut(),
            18 => ctx.x18_mut(),
            19 => ctx.x19_mut(),
            20 => ctx.x20_mut(),
            21 => ctx.x21_mut(),
            22 => ctx.x22_mut(),
            23 => ctx.x23_mut(),
            24 => ctx.x24_mut(),
            25 => ctx.x25_mut(),
            26 => ctx.x26_mut(),
            27 => ctx.x27_mut(),
            28 => ctx.x28_mut(),
            29 => ctx.x29_mut(),
            30 => ctx.x30_mut(),
            _ => panic!(),
        }
    }
}

fn register_index(vm_fault: &VmFault) -> Word {
    esr::SRT.get(vm_fault.valid_esr())
}

self_impl!(VCpuFaultExt, VCpuFault);

pub trait VCpuFaultExt {
    inner_decls!(VCpuFault);

    fn is_wfx(&self) -> bool {
        esr::EC.get(self.inner().hsr()) == 0b000001
    }
}
