//! Pulley registers.

use core::{fmt, ops::Range};

macro_rules! define_registers {
    (
        $(
            $( #[$attr:meta] )*
            pub struct $name:ident = $range:expr;
        )*
) => {
        $(
            $( #[ $attr ] )*
            #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name(u8);

            impl fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
                    fmt::Display::fmt(self, f)
                }
            }

            impl $name {
                /// The valid register range for this register class.
                pub const RANGE: Range<u8> = $range;

                /// Construct a new register of this class.
                #[inline]
                pub fn new(index: u8) -> Option<Self> {
                    if Self::RANGE.start <= index && index < Self::RANGE.end {
                        Some(unsafe { Self::unchecked_new(index) })
                    } else {
                        None
                    }
                }

                /// Construct a new register of this class without checking that
                /// `index` is a valid register index.
                #[inline]
                pub unsafe fn unchecked_new(index: u8) -> Self {
                    debug_assert!(Self::RANGE.start <= index && index < Self::RANGE.end);
                    Self(index)
                }

                /// Get this register's index.
                #[inline]
                pub fn index(&self) -> usize {
                    usize::from(self.0)
                }
            }

            #[cfg(feature = "arbitrary")]
            impl<'a> arbitrary::Arbitrary<'a> for $name {
                fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                    let index = u.int_in_range(Self::RANGE.start..=Self::RANGE.end - 1)?;
                    Ok(Self(index))
                }
            }
        )*
    }
}

define_registers! {
    /// An `x` register: integers.
    pub struct XReg = 0..32;

    /// An `f` register: floats.
    pub struct FReg = 0..32;

    /// A `v` register: vectors.
    pub struct VReg = 0..32;
}

/// Any register, regardless of class.
///
/// Never appears inside an instruction -- instructions always name a particular
/// class of register -- but this is useful for testing and things like that.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnyReg {
    X(XReg),
    F(FReg),
    V(VReg),
}

impl From<XReg> for AnyReg {
    fn from(x: XReg) -> Self {
        Self::X(x)
    }
}

impl From<FReg> for AnyReg {
    fn from(f: FReg) -> Self {
        Self::F(f)
    }
}

impl From<VReg> for AnyReg {
    fn from(v: VReg) -> Self {
        Self::V(v)
    }
}

impl fmt::Display for AnyReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyReg::X(r) => fmt::Display::fmt(r, f),
            AnyReg::F(r) => fmt::Display::fmt(r, f),
            AnyReg::V(r) => fmt::Display::fmt(r, f),
        }
    }
}

impl fmt::Debug for AnyReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for AnyReg {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        match u.int_in_range(0..=2)? {
            0 => Ok(AnyReg::X(u.arbitrary()?)),
            1 => Ok(AnyReg::F(u.arbitrary()?)),
            2 => Ok(AnyReg::V(u.arbitrary()?)),
            _ => unreachable!(),
        }
    }
}

impl XReg {
    /// The valid special register range.
    pub const SPECIAL_RANGE: Range<u8> = 27..32;

    /// The special `sp` stack pointer register.
    pub const SP: Self = Self(27);

    /// The special `lr` link register.
    pub const LR: Self = Self(28);

    /// The special `fp` frame pointer register.
    pub const FP: Self = Self(29);

    /// The special `spilltmp0` scratch register.
    pub const SPILL_TMP_0: Self = Self(30);

    /// The special `spilltmp1` scratch register.
    pub const SPILL_TMP_1: Self = Self(31);

    /// Is this `x` register a special register?
    pub fn is_special(&self) -> bool {
        self.0 >= Self::SPECIAL_RANGE.start
    }
}

impl fmt::Display for XReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SP => write!(f, "sp"),
            Self::LR => write!(f, "lr"),
            Self::FP => write!(f, "fp"),
            Self::SPILL_TMP_0 => write!(f, "spilltmp0"),
            Self::SPILL_TMP_1 => write!(f, "spilltmp1"),
            Self(x) => write!(f, "x{x}"),
        }
    }
}

impl fmt::Display for FReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "f{}", self.0)
    }
}

impl fmt::Display for VReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// TODO: docs
pub trait Reg: Copy + Sized {
    /// TODO: docs
    unsafe fn from_u8_unchecked(it: u8) -> Self;

    /// TODO: docs
    fn to_u8(self) -> u8;
}

impl Reg for XReg {
    unsafe fn from_u8_unchecked(it: u8) -> Self {
        Self::unchecked_new(it)
    }

    fn to_u8(self) -> u8 {
        self.0
    }
}

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
            let dst = R::from_u8_unchecked(dst as u8);
            let src1 = R::from_u8_unchecked(src1 as u8);
            let src2 = R::from_u8_unchecked(src2 as u8);

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
    fn special_x_regs() {
        assert!(XReg::SP.is_special());
        assert!(XReg::LR.is_special());
        assert!(XReg::FP.is_special());
        assert!(XReg::SPILL_TMP_0.is_special());
        assert!(XReg::SPILL_TMP_1.is_special());
    }

    #[test]
    fn not_special_x_regs() {
        for i in 0..27 {
            assert!(!XReg::new(i).unwrap().is_special());
        }
    }

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
