//! Encoding support for pulley bytecode.

use crate::imms::*;
use crate::opcode::{ExtendedOpcode, Opcode};
use crate::regs::*;

/// Trait for types that can be encoded into pulley bytecode.
pub trait Encode {
    /// Number of bytes this type serializes to.
    const ENCODED_SIZE: usize;

    /// Write `self` to `sink`.
    fn encode<E>(&self, sink: &mut E)
    where
        E: Extend<u8>;
}

macro_rules! encode_prims {
    ($($tys:ty),*) => {
       $(
        impl Encode for $tys {
            const ENCODED_SIZE: usize = core::mem::size_of::<Self>();

            fn encode<E>(&self, sink: &mut E)
            where
                E: Extend<u8>,
            {
                sink.extend(self.to_le_bytes());
            }
        }
       )*
    };
}

encode_prims!(u8, u16, u32, u64, i8, i16, i32, i64);

macro_rules! encode_via {
    ($ty:ty => $underlying:ty) => {
        impl Encode for $ty {
            const ENCODED_SIZE: usize = <$underlying as Encode>::ENCODED_SIZE;

            fn encode<E>(&self, sink: &mut E)
            where
                E: Extend<u8>,
            {
                (<$underlying>::from(*self)).encode(sink);
            }
        }
    };
}

encode_via!(XReg => u8);
encode_via!(FReg => u8);
encode_via!(VReg => u8);
encode_via!(PcRelOffset => i32);

impl<R: Reg> Encode for BinaryOperands<R> {
    const ENCODED_SIZE: usize = <u16 as Encode>::ENCODED_SIZE;

    fn encode<E>(&self, sink: &mut E)
    where
        E: Extend<u8>,
    {
        u16::encode(&self.to_bits(), sink);
    }
}

impl<R: Reg + Encode> Encode for RegSet<R> {
    const ENCODED_SIZE: usize = <u32 as Encode>::ENCODED_SIZE;

    fn encode<E>(&self, sink: &mut E)
    where
        E: Extend<u8>,
    {
        u32::encode(&self.to_bitset().0, sink)
    }
}

macro_rules! impl_encoders {
    (
        $(
            $( #[$attr:meta] )*
                $snake_name:ident = $name:ident $( {
                $(
                    $( #[$field_attr:meta] )*
                    $field:ident : $field_ty:ty
                ),*
            } )? ;
        )*
    ) => {
        $(
            $( #[$attr] )*
            pub fn $snake_name<E>(into: &mut E $( $( , $field : impl Into<$field_ty> )* )? )
            where
                E: Extend<u8>,
            {
                into.extend(core::iter::once(Opcode::$name as u8));
                $(
                    $(
                        $field.into().encode(into);
                    )*
                )?
            }

            impl Encode for crate::$name {
                const ENCODED_SIZE: usize = 0 $( $( + <$field_ty as Encode>::ENCODED_SIZE )* )?;

                #[allow(unused_variables)] // `sink` is unused if there are no fields.
                fn encode<E>(&self, sink: &mut E)
                where
                    E: Extend<u8>,
                {
                    $(
                        $(
                            self.$field.encode(sink);
                        )*
                    )?
                }
            }
        )*
    };
}
for_each_op!(impl_encoders);

macro_rules! impl_extended_encoders {
    (
        $(
            $( #[$attr:meta] )*
                $snake_name:ident = $name:ident $( {
                    $(
                        $( #[$field_attr:meta] )*
                        $field:ident : $field_ty:ty
                    ),*
                } )? ;
        )*
    ) => {
        $(
            $( #[$attr] )*
            pub fn $snake_name<E>(into: &mut E $( $( , $field : impl Into<$field_ty> )* )? )
            where
                E: Extend<u8>,
            {
                into.extend(core::iter::once(Opcode::ExtendedOp as u8));
                into.extend((ExtendedOpcode::$name as u16).to_le_bytes());
                $(
                    $(
                        $field.into().encode(into);
                    )*
                )?
            }
        )*
    };
}
for_each_extended_op!(impl_extended_encoders);
