use core::ffi::c_uint;

use crate::{sys, ObjectType};

/// Alias for [`ObjectTypeAArch64`].
pub type ObjectTypeSeL4Arch = ObjectTypeAArch64;

/// Alias for [`ObjectBlueprintAArch64`].
pub type ObjectBlueprintSeL4Arch = ObjectBlueprintAArch64;

/// Corresponds to `seL4_ModeObjectType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeAArch64 {
    HugePage,
    PUD,
    PGD,
}

impl ObjectTypeAArch64 {
    pub const fn into_sys(self) -> c_uint {
        match self {
            Self::HugePage => sys::_mode_object::seL4_ARM_HugePageObject,
            Self::PUD => sys::_mode_object::seL4_ARM_PageUpperDirectoryObject,
            Self::PGD => sys::_mode_object::seL4_ARM_PageGlobalDirectoryObject,
        }
    }
}

/// AArch64-specific variants of [`ObjectBlueprint`](crate::ObjectBlueprint).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintAArch64 {
    HugePage,
    PUD,
    PGD,
}

impl ObjectBlueprintAArch64 {
    pub const fn ty(self) -> ObjectType {
        match self {
            Self::HugePage => ObjectTypeAArch64::HugePage.into(),
            Self::PUD => ObjectTypeAArch64::PUD.into(),
            Self::PGD => ObjectTypeAArch64::PGD.into(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => sys::seL4_HugePageBits.try_into().ok().unwrap(),
            Self::PUD => sys::seL4_PUDBits.try_into().ok().unwrap(),
            Self::PGD => sys::seL4_PGDBits.try_into().ok().unwrap(),
        }
    }
}
