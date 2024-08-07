//! Pulley registers.

use core::fmt;
use std::ops::Range;

/// An `x` register: integers.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[rustfmt::skip]
#[allow(non_camel_case_types, missing_docs)]
pub enum XReg {
    x0,  x1,  x2,  x3,  x4,  x5,  x6,  x7,  x8,  x9,
    x10, x11, x12, x13, x14, x15, x16, x17, x18, x19,
    x20, x21, x22, x23, x24, x25, x26, x27, x28, x29,
    x30, x31
}

/// An `f` register: floats.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[rustfmt::skip]
#[allow(non_camel_case_types, missing_docs)]
pub enum FReg {
    f0,  f1,  f2,  f3,  f4,  f5,  f6,  f7,  f8,  f9,
    f10, f11, f12, f13, f14, f15, f16, f17, f18, f19,
    f20, f21, f22, f23, f24, f25, f26, f27, f28, f29,
    f30, f31
}

/// A `v` register: vectors.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[rustfmt::skip]
#[allow(non_camel_case_types, missing_docs)]
pub enum VReg {
    v0,  v1,  v2,  v3,  v4,  v5,  v6,  v7,  v8,  v9,
    v10, v11, v12, v13, v14, v15, v16, v17, v18, v19,
    v20, v21, v22, v23, v24, v25, v26, v27, v28, v29,
    v30, v31
}

/// An `s` register: special machine state.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[allow(non_camel_case_types, missing_docs)]
pub enum SReg {
    SP,
    LR,
    FP,
    SPILL_TMP_0,
    SPILL_TMP_1,
}

impl fmt::Debug for SReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SP => write!(f, "sp"),
            Self::LR => write!(f, "lr"),
            Self::FP => write!(f, "fp"),
            Self::SPILL_TMP_0 => write!(f, "spilltmp0"),
            Self::SPILL_TMP_1 => write!(f, "spilltmp1"),
        }
    }
}

/// Any register, regardless of class.
///
/// Never appears inside an instruction -- instructions always name a particular
/// class of register -- but this is useful for testing and things like that.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum AnyReg {
    X(XReg),
    F(FReg),
    V(VReg),
    S(SReg),
}

impl fmt::Display for AnyReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyReg::X(r) => fmt::Display::fmt(r, f),
            AnyReg::F(r) => fmt::Display::fmt(r, f),
            AnyReg::V(r) => fmt::Display::fmt(r, f),
            AnyReg::S(r) => fmt::Display::fmt(r, f),
        }
    }
}

impl fmt::Debug for AnyReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// TODO: docs
pub trait Reg: Copy + Sized {
    /// Bounds of valid underluing `u8` values.
    const RANGE: Range<u8>;

    /// Convert this register from the underlying `u8`, without bounds checks.
    unsafe fn new_unchecked(it: u8) -> Self;

    /// Convert this register from the underlying `u8`, with bounds checks.
    #[inline]
    fn new(it: u8) -> Option<Self> {
        if Self::RANGE.contains(&it) {
            unsafe { Some(Self::new_unchecked(it)) }
        } else {
            None
        }
    }

    /// Convert this register to the underlying `u8`.
    fn to_u8(self) -> u8;

    /// Get this register's index.
    #[inline]
    fn index(&self) -> usize {
        usize::from(self.to_u8())
    }
}

macro_rules! impl_reg {
    ($ty:ty, $any:ident, $range:expr ) => {
        impl Reg for $ty {
            const RANGE: Range<u8> = $range;

            #[inline]
            unsafe fn new_unchecked(it: u8) -> Self {
                debug_assert!(Self::RANGE.contains(&it));
                std::mem::transmute(it)
            }

            #[inline]
            fn to_u8(self) -> u8 {
                self as u8
            }
        }

        impl From<$ty> for AnyReg {
            fn from(r: $ty) -> Self {
                Self::$any(r)
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    };
}

impl_reg!(XReg, X, 0..32);
impl_reg!(FReg, F, 0..32);
impl_reg!(VReg, V, 0..32);
impl_reg!(SReg, S, 0..(Self::SPILL_TMP_1 as u8 + 1));

/// TODO: docs
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinaryOperands<R> {
    /// Destination register. Packed into bits 0..5
    pub dst: R,
    /// First source register. Packed into bits 5..10
    pub src1: R,
    /// Second source register. Packed into bits 10..15
    pub src2: R,
}

impl<R: Reg> BinaryOperands<R> {
    /// TODO: docs
    pub fn to_bits(self) -> u16 {
        let dst = self.dst.to_u8() as u16;
        let src1 = self.src1.to_u8() as u16;
        let src2 = self.src2.to_u8() as u16;
        dst | (src1 << 5) | (src2 << 10)
    }

    /// TODO: docs
    pub fn from_bits(bits: u16) -> Self {
        let dst = bits & 0b0_00000_00000_11111;
        let src1 = (bits & 0b0_00000_11111_00000) >> 5;
        let src2 = (bits & 0b0_11111_00000_00000) >> 10;

        // SAFETY: each of `dst`, `src1` and `src2` are 5 bits, so cannot be out
        // of range.
        unsafe {
            let dst = R::new_unchecked(dst as u8);
            let src1 = R::new_unchecked(src1 as u8);
            let src2 = R::new_unchecked(src2 as u8);

            Self { dst, src1, src2 }
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, R: Reg> arbitrary::Arbitrary<'a> for BinaryOperands<R> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary().map(|bits| Self::from_bits(bits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_operands() {
        let mut i = 0;
        for src2 in 0..32 {
            for src1 in 0..32 {
                for dst in 0..32 {
                    let operands = BinaryOperands {
                        dst: XReg::new(dst).unwrap(),
                        src1: XReg::new(src1).unwrap(),
                        src2: XReg::new(src2).unwrap(),
                    };

                    assert_eq!(operands.to_bits(), i);
                    assert_eq!(BinaryOperands::from_bits(i), operands);
                    assert_eq!(BinaryOperands::from_bits(0x8000 | i), operands);
                    i += 1;
                }
            }
        }
    }
}
