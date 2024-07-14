//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

macro_rules! newtype_methods {
    ($inner_vis:vis $inner:ty) => {
        $inner_vis const fn from_inner(inner: $inner) -> Self {
            Self(inner)
        }

        $inner_vis const fn into_inner(self) -> $inner {
            self.0
        }

        $inner_vis const fn inner(&self) -> &$inner {
            &self.0
        }

        $inner_vis fn inner_mut(&mut self) -> &mut $inner {
            &mut self.0
        }
    };
}

macro_rules! declare_cap_type {
    (
        $(#[$outer:meta])*
        $t:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $t;

        impl $crate::CapType for $t {
            const NAME: &'static str = stringify!($t);
        }
    };
}

macro_rules! declare_cap_type_for_object {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident }
    ) => {
        $crate::declare_cap_type! {
            $(#[$outer])*
            $t
        }

        impl $crate::CapTypeForObject for $t {
            fn object_type() -> $crate::ObjectType {
                $crate::$object_type::$t.into()
            }
        }
    };
}

macro_rules! declare_cap_type_for_object_of_fixed_size {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident, $object_blueprint:ident }
    ) => {
        $crate::declare_cap_type_for_object! {
            $(#[$outer])*
            $t { $object_type }
        }

        impl $crate::CapTypeForObjectOfFixedSize for $t {
            fn object_blueprint() -> $crate::ObjectBlueprint {
                $crate::$object_blueprint::$t.into()
            }
        }
    };
}

macro_rules! declare_cap_type_for_object_of_variable_size {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident, $object_blueprint:ident }
    ) => {
        $crate::declare_cap_type_for_object! {
            $(#[$outer])*
            $t { $object_type }
        }

        impl $crate::CapTypeForObjectOfVariableSize for $t {
            fn object_blueprint(size_bits: usize) -> $crate::ObjectBlueprint {
                ($crate::$object_blueprint::$t { size_bits }).into()
            }
        }
    };
}

macro_rules! declare_cap_alias {
    (
        $(#[$outer:meta])*
        $t:ident
    ) => {
        $(#[$outer])*
        pub type $t<C = $crate::NoExplicitInvocationContext> =
            $crate::Cap<$crate::cap_type::$t, C>;
    };
}

macro_rules! declare_fault_newtype {
    ($t:ident, $sys:path) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $t($sys);

        impl $t {
            $crate::newtype_methods!(pub $sys);
        }
    };
}

macro_rules! fault_newtype_getter_method {
    ($outer:ident, $inner:ident) => {
        pub fn $outer(&self) -> Word {
            self.inner().$inner()
        }
    };
}

macro_rules! declare_fault_ipc_buffer_newtype {
    ($t:ident, $length:expr) => {
        pub struct $t<'a>(&'a mut $crate::IpcBuffer);

        impl<'a> $t<'a> {
            pub fn from_inner(inner: &'a mut $crate::IpcBuffer) -> Self {
                Self(inner)
            }

            pub fn into_inner(self) -> &'a mut $crate::IpcBuffer {
                self.0
            }

            pub fn inner(&self) -> &$crate::IpcBuffer {
                &self.0
            }

            pub fn inner_mut(&mut self) -> &mut $crate::IpcBuffer {
                &mut self.0
            }

            // TODO
            #[$crate::config::sel4_cfg(not(KERNEL_MCS))]
            pub fn reply(&mut self) {
                $crate::reply(
                    self.inner_mut(),
                    $crate::MessageInfoBuilder::default()
                        .length($length as usize)
                        .build(),
                )
            }
        }
    };
}

macro_rules! fault_ipc_buffer_newtype_ref_methods {
    ($outer:ident, $outer_mut:ident, $ix:expr) => {
        pub fn $outer(&self) -> &$crate::Word {
            &self.inner().msg_regs()[$ix as usize]
        }

        pub fn $outer_mut(&mut self) -> &mut $crate::Word {
            &mut self.inner_mut().msg_regs_mut()[$ix as usize]
        }
    };
}

macro_rules! user_context_newtype_ref_methods {
    ($outer:ident, $outer_mut:ident) => {
        pub fn $outer(&self) -> &$crate::Word {
            &self.inner().$outer
        }

        pub fn $outer_mut(&mut self) -> &mut $crate::Word {
            &mut self.inner_mut().$outer
        }
    };
}

pub(crate) use declare_cap_alias;
pub(crate) use declare_cap_type;
pub(crate) use declare_cap_type_for_object;
pub(crate) use declare_cap_type_for_object_of_fixed_size;
pub(crate) use declare_cap_type_for_object_of_variable_size;
pub(crate) use declare_fault_ipc_buffer_newtype;
pub(crate) use declare_fault_newtype;
pub(crate) use fault_ipc_buffer_newtype_ref_methods;
pub(crate) use fault_newtype_getter_method;
pub(crate) use newtype_methods;
pub(crate) use user_context_newtype_ref_methods;
