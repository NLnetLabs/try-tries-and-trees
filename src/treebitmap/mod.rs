use crate::common::{AddressFamily, NoMeta, Prefix};
use num::PrimInt;
use std::cmp::Ordering;
use std::fmt::{Binary, Debug};

// pub struct BitMap8stride(u8, 8);
#[derive(Copy, Clone)]
pub struct U256(u128, u128);

impl Debug for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0128b}\n             {:0128b}",
            self.0, self.1
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct U512(u128, u128, u128, u128);

pub type Stride3 = u16;
pub type Stride4 = u32;
pub type Stride5 = u64;
pub type Stride6 = u128;
pub type Stride7 = U256;
pub type Stride8 = U512;

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0, &other.0) {
            (a, b) if &a > b => Some(self.0.cmp(&other.0)),
            _ => Some(self.1.cmp(&other.1)),
        }
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0, &other.0) {
            (a, b) if &a > b => self.0.cmp(&other.0),
            _ => self.1.cmp(&other.1),
        }
    }
}

impl PartialOrd for U512 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0, &other.0) {
            (a, b) if &a > b => Some(self.0.cmp(&other.0)),
            _ => match (self.1, &other.1) {
                (a, b) if &a > b => Some(self.1.cmp(&other.1)),
                _ => match (self.2, &other.2) {
                    (a, b) if &a > b => Some(self.2.cmp(&other.2)),
                    _ => Some(self.3.cmp(&other.3)),
                },
            },
        }
    }
}

pub trait Stride: Sized + Debug + Binary + Eq + PartialOrd + PartialEq + Debug + Copy {
    type PtrSize;
    const BITS: u8;
    const STRIDE_LEN: u8;

    // Get the bit position of the start of the given nibble.
    // The nibble is defined as a `len` number of bits set from the right.

    // `<Self as Stride>::BITS`
    // is the whole length of the bitmap, since we are shifting to the left,
    // we have to start at the end of the bitmap.
    // `((1 << len) - 1)`
    // is the offset for this nibble length in the bitmap.
    // `nibble`
    // shifts to the right position withing the bit range for this nibble
    // length, this follows from the fact that the `nibble` value represents
    // *both* the bitmap part, we're considering here *and* the position
    // relative to the nibble length offset in the bitmap.
    fn get_bit_pos(nibble: u32, len: u8) -> Self;

    // Clear the bitmap to the right of the pointer and count the number of ones.
    // This numbder represents the index to the corresponding prefix in the pfx_vec.

    // Clearing is performed by shifting to the right until we have the nibble
    // all the way at the right.

    // `(<Self as Stride>::BITS >> 1)`
    // The end of the bitmap (this bitmap is half the size of the pfx bitmap)

    // `nibble`
    // The bit position relative to the offset for the nibble length, this index
    // is only used at the last (relevant) stride, so the offset is always 0.
    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize;

    // Clear the bitmap to the right of the pointer and count the number of ones.
    // This number represents the index to the corresponding child node in the ptr_vec.

    // Clearing is performed by shifting to the right until we have the nibble
    // all the way at the right.

    // For ptrbitarr the only index we want is the one for a full-length nibble
    // (stride length) at the last stride, so we don't need the length of the nibble

    // `(<Self as Stride>::BITS >> 1)`
    // The end of the bitmap (this bitmap is half the size of the pfx bitmap),
    // ::BITS is the size of the pfx bitmap.

    // `nibble`
    // The bit position relative to the offset for the nibble length, this index
    // is only used at the last (relevant) stride, so the offset is always 0.
    fn get_ptr_index(bitmap: Self::PtrSize, nibble: u32) -> usize;

    // Convert a ptrbitarr into a pfxbitarr sized bitmap,
    // so we can do bitwise operations with a pfxbitarr sized
    // bitmap on them.
    // Since the last bit in the pfxbitarr isn't used, but the
    // full ptrbitarr *is* used, the prtbitarr should be shifted
    // one bit to the left.
    fn into_stride_size(bitmap: Self::PtrSize) -> Self;

    // Convert a pfxbitarr sized bitmap into a ptrbitarr sized
    // Note that bitwise operators align bits of unsigend types with different
    // sizes to the right, so we don't have to do anything to pad the smaller sized
    // type. We do have to shift one bit to the left, to accomodate the unused pfxbitarr's
    // last bit.
    fn into_ptrbitarr_size(bitmap: Self) -> Self::PtrSize;

    fn zero() -> Self;
    fn one() -> Self;
    fn leading_zeros(self) -> u32;
}

trait Zero {
    fn zero() -> Self;
}

trait One {
    fn one() -> Self;
}

impl Zero for u8 {
    fn zero() -> u8 {
        0
    }
}

impl One for u8 {
    fn one() -> u8 {
        1
    }
}

impl Stride for Stride3 {
    type PtrSize = u8;
    const BITS: u8 = 16;
    const STRIDE_LEN: u8 = 3;

    fn get_bit_pos(nibble: u32, len: u8) -> Self {
        1 << (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1)
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        (bitmap >> ((<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1) as usize))
            .count_ones() as usize
            - 1
    }
    fn get_ptr_index(bitmap: Self::PtrSize, nibble: u32) -> usize {
        (bitmap >> ((<Self as Stride>::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
            as usize
            - 1
    }

    fn into_stride_size(bitmap: u8) -> u16 {
        (bitmap as u16) << 1
    }

    fn into_ptrbitarr_size(bitmap: Self) -> u8 {
        (bitmap >> 1) as u8
    }

    #[inline]
    fn zero() -> Self {
        0
    }

    #[inline]
    fn one() -> Self {
        1
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
}

impl Stride for Stride4 {
    type PtrSize = u16;
    const BITS: u8 = 32;
    const STRIDE_LEN: u8 = 4;

    fn get_bit_pos(nibble: u32, len: u8) -> u32 {
        1 << (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1)
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        (bitmap >> ((<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1) as usize))
            .count_ones() as usize
            - 1
    }

    fn get_ptr_index(bitmap: u16, nibble: u32) -> usize {
        (bitmap >> ((<Self as Stride>::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
            as usize
            - 1
    }

    fn into_stride_size(bitmap: u16) -> u32 {
        (bitmap as u32) << 1
    }

    fn into_ptrbitarr_size(bitmap: u32) -> u16 {
        (bitmap >> 1) as u16
    }

    #[inline]
    fn zero() -> Self {
        0
    }

    #[inline]
    fn one() -> Self {
        1
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
}

impl Stride for Stride5 {
    type PtrSize = u32;
    const BITS: u8 = 64;
    const STRIDE_LEN: u8 = 5;

    fn get_bit_pos(nibble: u32, len: u8) -> u64 {
        1 << (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1)
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        (bitmap >> ((<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1) as usize))
            .count_ones() as usize
            - 1
    }
    fn get_ptr_index(bitmap: u32, nibble: u32) -> usize {
        (bitmap >> ((<Self as Stride>::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
            as usize
            - 1
    }

    fn into_stride_size(bitmap: u32) -> u64 {
        (bitmap as u64) << 1
    }

    fn into_ptrbitarr_size(bitmap: u64) -> u32 {
        (bitmap >> 1) as u32
    }

    #[inline]
    fn zero() -> Self {
        0
    }

    #[inline]
    fn one() -> Self {
        1
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
}

impl Stride for Stride6 {
    type PtrSize = u64;
    const BITS: u8 = 128;
    const STRIDE_LEN: u8 = 6;

    fn get_bit_pos(nibble: u32, len: u8) -> u128 {
        1 << (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1)
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        (bitmap >> ((<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1) as usize))
            .count_ones() as usize
            - 1
    }
    fn get_ptr_index(bitmap: u64, nibble: u32) -> usize {
        (bitmap >> ((<Self as Stride>::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
            as usize
            - 1
    }

    fn into_stride_size(bitmap: u64) -> u128 {
        (bitmap as u128) << 1
    }

    fn into_ptrbitarr_size(bitmap: u128) -> u64 {
        (bitmap >> 1) as u64
    }

    #[inline]
    fn zero() -> Self {
        0
    }

    #[inline]
    fn one() -> Self {
        1
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
}

impl Stride for Stride7 {
    type PtrSize = u128;
    const BITS: u8 = 255;
    const STRIDE_LEN: u8 = 7;

    fn get_bit_pos(nibble: u32, len: u8) -> Self {
        match 256 - ((1 << len) - 1) as u16 - nibble as u16 - 1 {
            n if n < 128 => {
                // println!("n {}", n);
                U256(0, 1 << n)
            }
            n => {
                // println!("nn {}", n);
                U256(1 << (n as u16 - 128), 0)
            }
        }
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        let n = 256 - ((1 << len) - 1) as u16 - nibble as u16 - 1;
        match n {
            // if we move less than 128 bits to the right,
            // all of bitmap.0 and a part of bitmap.1 will be used for counting zeros
            // ex.
            // ...1011_1010... >> 2 => ...0010_111010...
            //    ____ ====                 -- --====
            n if n < 128 => {
                bitmap.0.count_ones() as usize + (bitmap.1 >> n).count_ones() as usize - 1
            }
            // if we move more than 128 bits to the right,
            // all of bitmap.1 wil be shifted out of sight,
            // so we only have to count bitmap.0 zeroes than (after) shifting of course).
            n => (bitmap.0 >> n).count_ones() as usize - 1,
        }
    }

    fn get_ptr_index(bitmap: Self::PtrSize, nibble: u32) -> usize {
        (bitmap >> ((256 >> 1) - nibble as u16 - 1) as usize).count_ones() as usize - 1
    }

    fn into_stride_size(bitmap: Self::PtrSize) -> Self {
        // One bit needs to move into the self.0 u128,
        // since the last bit of the *whole* bitmap isn't used.
        U256(bitmap >> 127, bitmap << 1)
    }

    fn into_ptrbitarr_size(bitmap: Self) -> Self::PtrSize {
        // TODO expand:
        // self.ptrbitarr =
        // S::into_ptrbitarr_size(bit_pos | S::into_stride_size(self.ptrbitarr));
        (bitmap.0 << 127 | bitmap.1 >> 1) as u128
    }

    #[inline]
    fn zero() -> Self {
        U256(0, 0)
    }

    #[inline]
    fn one() -> Self {
        U256(0, 1)
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        let lz = self.0.leading_zeros();
        let r = if lz == 128 {
            lz + self.1.leading_zeros()
        } else {
            lz
        };
        // println!("lz {}", r);
        r
    }
}

impl PartialEq for Stride7 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Stride7 {}

impl Binary for Stride7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self, f)
    }
}

impl std::ops::BitOr<Self> for Stride7 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1)
    }
}

impl std::ops::BitAnd<Self> for Stride7 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output
    where
        Self: Eq,
    {
        Self(self.0 & rhs.0, self.1 & rhs.1)
    }
}

impl Stride for Stride8 {
    type PtrSize = U256;
    const BITS: u8 = 255; // bogus
    const STRIDE_LEN: u8 = 8;

    fn get_bit_pos(nibble: u32, len: u8) -> Self {
        match 512 - ((1 << len) - 1) as u16 - nibble as u16 - 1 {
            n if n < 128 => {
                // println!("n {}", n);
                U512(0, 0, 0, 1 << n)
            }
            n if n < 256 => U512(0, 0, 1 << (n as u16 - 128), 0),
            n if n < 384 => {
                // println!("nn {}", n);
                U512(0, 1 << (n as u16 - 256), 0, 0)
            }
            n => U512(1 << (n as u16 - 384), 0, 0, 0),
        }
    }

    fn get_pfx_index(bitmap: Self, nibble: u32, len: u8) -> usize {
        let n = 512 - ((1 << len) - 1) as u16 - nibble as u16 - 1;
        match n {
            // if we move less than 128 bits to the right,
            // all of bitmap.2 and a part of bitmap.3 will be used for counting zeros
            // ex.
            // ...1011_1010... >> 2 => ...0010_111010...
            //    ____ ====                 -- --====
            n if n < 128 => {
                bitmap.0.count_ones() as usize
                    + bitmap.1.count_ones() as usize
                    + bitmap.2.count_ones() as usize
                    + (bitmap.3 >> n).count_ones() as usize
                    - 1
            }

            n if n < 256 => {
                bitmap.0.count_ones() as usize
                    + bitmap.1.count_ones() as usize
                    + (bitmap.2 >> n).count_ones() as usize
                    - 1
            }

            n if n < 384 => {
                bitmap.0.count_ones() as usize + (bitmap.1 >> n).count_ones() as usize - 1
            }

            // if we move more than 384 bits to the right,
            // all of bitmap.[1,2,3] will be shifted out of sight,
            // so we only have to count bitmap.0 zeroes than (after) shifting of course).
            n => (bitmap.0 >> n).count_ones() as usize - 1,
        }

    }

    fn get_ptr_index(bitmap: Self::PtrSize, nibble: u32) -> usize {
        let n = (512 >> 1) - nibble as u16 - 1;
        match n {
            // if we move less than 256 bits to the right,
            // all of bitmap.0 and a part of bitmap.1 will be used for counting zeros
            // ex.
            // ...1011_1010... >> 2 => ...0010_111010...
            //    ____ ====                 -- --====
            n if n < 128 => {
                bitmap.0.count_ones() as usize + (bitmap.1 >> n).count_ones() as usize - 1
            }
            // if we move more than 256 bits to the right,
            // all of bitmap.1 wil be shifted out of sight,
            // so we only have to count bitmap.0 zeroes than (after) shifting of course).
            n => (bitmap.0 >> n).count_ones() as usize - 1,
        }
    }

    fn into_stride_size(bitmap: Self::PtrSize) -> Self {
        // One bit needs to move into the self.0 u128,
        // since the last bit of the *whole* bitmap isn't used.
        U512(
            0,
            bitmap.0 >> 127,
            (bitmap.0 << 1) | (bitmap.1 >> 127),
            bitmap.1 << 1,
        )
        // U256(bitmap >> 127, bitmap << 1)

    }

    fn into_ptrbitarr_size(bitmap: Self) -> Self::PtrSize {
        // TODO expand:
        // self.ptrbitarr =
        // S::into_ptrbitarr_size(bit_pos | S::into_stride_size(self.ptrbitarr));
        U256(
            (bitmap.1 << 127 | bitmap.2 >> 1) as u128,
            (bitmap.2 << 127 | bitmap.3 >> 1) as u128,
        )
    }

    #[inline]
    fn zero() -> Self {
        U512(0, 0, 0, 0)
    }

    #[inline]
    fn one() -> Self {
        U512(0, 0, 0, 1)
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        let mut lz = self.0.leading_zeros();
        if lz == 128 {
            lz += self.1.leading_zeros();
            if lz == 256 {
                lz += self.2.leading_zeros();
                if lz == 384 {
                    lz += self.3.leading_zeros();
                }
            }
        }
        lz
    }
}

impl PartialEq for Stride8 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
}

impl Eq for Stride8 {}

impl Binary for Stride8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self, f)
    }
}

impl std::ops::BitOr<Self> for Stride8 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(
            self.0 | rhs.0,
            self.1 | rhs.1,
            self.2 | rhs.2,
            self.3 | rhs.3,
        )
    }
}

impl std::ops::BitAnd<Self> for Stride8 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output
    where
        Self: Eq,
    {
        Self(
            self.0 & rhs.0,
            self.1 & rhs.1,
            self.2 & rhs.2,
            self.3 & rhs.3,
        )
    }
}

#[derive(Debug)]
pub enum SizedStrideNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
    Stride3(TreeBitMapNode<'a, AF, T, Stride3>),
    Stride4(TreeBitMapNode<'a, AF, T, Stride4>),
    Stride5(TreeBitMapNode<'a, AF, T, Stride5>),
    Stride6(TreeBitMapNode<'a, AF, T, Stride6>),
    Stride7(TreeBitMapNode<'a, AF, T, Stride7>),
    Stride8(TreeBitMapNode<'a, AF, T, Stride8>),
}

// pub enum SizeStrideVec<'a, AF, T, S>
// where
//     T: Debug,
//     AF: AddressFamily + Debug + PrimInt,
//     S: Debug + Stride + Binary,
//     <S as Stride>::PtrSize: Debug + Binary,
// {
//     Stride(Vec<TreeBitMapNode<'a, AF, T, S>>),
// }

pub struct TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
    S: Stride,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
{
    bit_id: u16,
    ptrbitarr: <S as Stride>::PtrSize,
    pfxbitarr: S,
    pfx_vec: Vec<&'a Prefix<AF, T>>,
    ptr_vec: Vec<SizedStrideNode<'a, AF, T>>,
}

impl<'a, AF, T> Eq for SizedStrideNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
}

impl<'a, AF, T> Ord for SizedStrideNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            SizedStrideNode::Stride3(nn) => {
                if let SizedStrideNode::Stride3(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
            SizedStrideNode::Stride4(nn) => {
                if let SizedStrideNode::Stride4(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
            SizedStrideNode::Stride5(nn) => {
                if let SizedStrideNode::Stride5(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
            SizedStrideNode::Stride6(nn) => {
                if let SizedStrideNode::Stride6(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
            SizedStrideNode::Stride7(nn) => {
                if let SizedStrideNode::Stride7(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
            SizedStrideNode::Stride8(nn) => {
                if let SizedStrideNode::Stride8(mm) = other {
                    nn.ptrbitarr.cmp(&mm.ptrbitarr)
                } else {
                    0.cmp(&1)
                }
            }
        }
    }
}

impl<'a, AF, T> PartialOrd for SizedStrideNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            SizedStrideNode::Stride3(nn) => {
                if let SizedStrideNode::Stride3(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&1))
                }
            }
            SizedStrideNode::Stride4(nn) => {
                if let SizedStrideNode::Stride4(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&1))
                }
            }
            SizedStrideNode::Stride5(nn) => {
                if let SizedStrideNode::Stride5(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&10))
                }
            }
            SizedStrideNode::Stride6(nn) => {
                if let SizedStrideNode::Stride6(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&10))
                }
            }
            SizedStrideNode::Stride7(nn) => {
                if let SizedStrideNode::Stride7(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&10))
                }
            }
            SizedStrideNode::Stride8(nn) => {
                if let SizedStrideNode::Stride8(mm) = other {
                    Some(nn.bit_id.cmp(&mm.bit_id))
                } else {
                    Some(0.cmp(&10))
                }
            }
        }
    }
}

impl<'a, AF, T> PartialEq for SizedStrideNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
    fn eq(&self, other: &Self) -> bool {
        match self {
            SizedStrideNode::Stride3(n) => {
                if let SizedStrideNode::Stride3(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
            SizedStrideNode::Stride4(n) => {
                if let SizedStrideNode::Stride4(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
            SizedStrideNode::Stride5(n) => {
                if let SizedStrideNode::Stride5(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
            SizedStrideNode::Stride6(n) => {
                if let SizedStrideNode::Stride6(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
            SizedStrideNode::Stride7(n) => {
                if let SizedStrideNode::Stride7(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
            SizedStrideNode::Stride8(n) => {
                if let SizedStrideNode::Stride7(m) = other {
                    n.bit_id == m.bit_id
                } else {
                    true
                }
            }
        }
        // let SizedStrideNode::Stride4(n) = self;
        // let SizedStrideNode::Stride4(m) = other;
        // n.bit_id == m.bit_id
    }
}

impl<'a, AF, T, S> Debug for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
    S: Stride,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeBitMapNode")
            .field("id", &self.bit_id)
            .field("ptrbitarr", &self.ptrbitarr)
            .field("pfxbitarr", &self.pfxbitarr)
            .field("ptr_vec", &self.ptr_vec)
            .finish()
    }
}

impl<'a, AF, T, S> TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
    S: Stride + std::ops::BitAnd<Output = S> + std::ops::BitOr<Output = S>,
    // + std::ops::Shr<Output = S>,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
{
    fn traverse(
        self: &mut Self,
        nibble: u32,
        nibble_len: u8,
        pfx: &'a Prefix<AF, T>,
        next_stride: Option<&&u8>,
        is_last_stride: bool,
    ) -> Option<&mut SizedStrideNode<'a, AF, T>> {
        let bit_pos = S::get_bit_pos(nibble, nibble_len);

        // println!("n {:b} nl {}", nibble, nibble_len);

        // Check that we're not at the last stride (pfx.len <= stride_end),
        // Note that next_stride may have a value, but we still don't want to
        // continue, because we've exceeded the length of the prefix to
        // be inserted.
        // Also note that a nibble_len < S::BITS (a smaller than full nibble)
        // does indeed indicate the last stride has been reached, but the
        // reverse is *not* true, i.e. a full nibble can also be the last
        // stride. Hence the `is_last_stride` argument
        if !is_last_stride {
            // We are not at the last stride
            // Check it the ptr bit is already set in this position
            if (S::into_stride_size(self.ptrbitarr) & bit_pos)
                == <S as std::ops::BitAnd>::Output::zero()
            {
                // Nope, set it and create a child node
                self.ptrbitarr =
                    S::into_ptrbitarr_size(bit_pos | S::into_stride_size(self.ptrbitarr));

                // println!(
                //     "ptr[{:?}]: xxxxxxxxxxxxxxx{:0128b}x",
                //     next_stride, self.ptrbitarr
                // );
                // println!("bit_id: {}", bit_pos.leading_zeros());
                let new_node = match next_stride.unwrap() {
                    3_u8 => SizedStrideNode::Stride3(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: <Stride3 as Stride>::PtrSize::zero(),
                        pfxbitarr: Stride3::zero(),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    4_u8 => SizedStrideNode::Stride4(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: <Stride4 as Stride>::PtrSize::zero(),
                        pfxbitarr: Stride4::zero(),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    5_u8 => SizedStrideNode::Stride5(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: <Stride5 as Stride>::PtrSize::zero(),
                        pfxbitarr: Stride5::zero(),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    6_u8 => SizedStrideNode::Stride6(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: <Stride6 as Stride>::PtrSize::zero(),
                        pfxbitarr: Stride6::zero(),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    7_u8 => SizedStrideNode::Stride7(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: 0_u128,
                        pfxbitarr: U256(0_u128, 0_u128),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    8_u8 => SizedStrideNode::Stride8(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u16,
                        ptrbitarr: U256(0_u128, 0_u128),
                        pfxbitarr: U512(0_u128, 0_u128, 0_u128, 0_u128),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    }),
                    _ => {
                        panic!("can't happen");
                    }
                };

                self.ptr_vec.push(new_node);
                self.ptr_vec.sort();
            }
        } else {
            // only at the last stride do we create the bit in the prefix bitmap,
            // and only if it doesn't exist already
            if self.pfxbitarr & bit_pos == <S as std::ops::BitAnd>::Output::zero() {
                self.pfxbitarr = bit_pos | self.pfxbitarr;
                // println!(
                //     "pfx[{:?}]n[{}]: {:032b}",
                //     next_stride, nibble_len, self.pfxbitarr
                // );

                self.pfx_vec.push(pfx);
                self.pfx_vec.sort();
                // println!("{:?}", self.pfx_vec);
            }
            return None;
        }

        // println!("__bit_pos__: {:?}", bit_pos);
        // println!(
        //     "ptr[{:?}]: xxxxxxxxxxxxxxx{:0128b}x",
        //     next_stride, self.ptrbitarr
        // );
        // println!("{:?}", self.ptr_vec);
        // println!("{}", (Stride4::BITS >> 1) - nibble as u8 - 1);
        // println!(
        //     "{}",
        //     (self.ptrbitarr >> ((Stride4::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
        // );
        // // println!("{}", next_index);
        // println!("{:?}", self.ptr_vec);
        // println!("?? {:?}", S::get_ptr_index(self.ptrbitarr, nibble));
        Some(&mut self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)])
    }

    fn search<'b>(
        self: &Self,
        search_pfx: &Prefix<AF, NoMeta>,
        mut nibble: u32,
        nibble_len: u8,
        start_bit: u8,
        found_pfx: &'b mut Vec<&'a Prefix<AF, T>>,
    ) -> Option<&SizedStrideNode<'a, AF, T>> {
        let mut bit_pos = S::get_bit_pos(nibble, nibble_len);

        for n_l in 1..(nibble_len + 1) {
            // Move the bit in the right position.

            // nibble = (search_pfx.net << (stride_end - stride) as usize)
            //     >> (((Self::BITS - n_l) % AF::BITS) as usize);
            nibble = AddressFamily::get_nibble(search_pfx.net, start_bit, n_l);

            // offset = (1_u32 << n_l) - 1; // 15 for a full stride of 4
            // bit_pos = 0x1 << (Self::BITS - offset as u8 - nibble as u8 - 1);
            bit_pos = S::get_bit_pos(nibble, n_l);

            // Check if the prefix has been set, if so select the prefix. This is not
            // necessarily the final prefix that will be returned.
            // println!(
            //     "idxpfx:      {:32b} {:32b}",
            //     current_node.pfxbitarr >> (Self::BITS - offset as u8 - nibble as u8 - 1),
            //     nibble
            // );
            // println!("bit_pos:     {:032b}", bit_pos);
            // println!("___pfx:      {:032b}", self.pfxbitarr);
            // println!(
            //     "___ptr:      xxxxxxxxxxxxxxx{:016b}x",
            //     current_node.ptrbitarr
            // );
            // println!("{:?}", current_node.pfx_vec);

            // Check it there's an prefix matching in this bitmap for this nibble
            if self.pfxbitarr & bit_pos > S::zero() {
                found_pfx.push(self.pfx_vec[S::get_pfx_index(self.pfxbitarr, nibble, n_l)]);
                // println!("vec: {:?}", self.pfx_vec);
                println!("found: {:?}", found_pfx);
            }
        }

        // If we are at the end of the prefix length or if there are no more
        // children we're returning what we found so far.
        // println!("{:#?}", current_node.ptr_vec);

        // println!(
        //     "idxptr:      {:32b}",
        //     // current_node.ptrbitarr >> ((Self::BITS - offset as u8 - nibble as u8 - 1) as usize)
        //     current_node.ptrbitarr
        //         >> (AF::BITS - nibble as u8 - 1) as usize
        // );

        // Check if this the last stride, or if they're no more children to go to,
        // if so return what we found up until now.
        // let SizedStrideNode::Stride4(current_node) = node;
        if search_pfx.len < start_bit
            || (S::into_stride_size(self.ptrbitarr) & bit_pos)
                == <S as std::ops::BitAnd>::Output::zero()
        {
            return None;
        }

        Some(&self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)])
    }
}

pub struct TreeBitMap<'a, AF, T>(SizedStrideNode<'a, AF, T>)
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt;

impl<'a, AF, T> TreeBitMap<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug + PrimInt,
{
    const STRIDES: [u8; 4] = [8, 8, 8, 8];
    pub fn new() -> TreeBitMap<'a, AF, T> {
        // Check if the strides division makes sense
        assert!(Self::STRIDES.iter().fold(0, |acc, s| { acc + s }) == AF::BITS);
        let node = match Self::STRIDES[0] {
            3 => SizedStrideNode::Stride3(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: 0,
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            4 => SizedStrideNode::Stride4(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: 0,
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            5 => SizedStrideNode::Stride5(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: 0,
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            6 => SizedStrideNode::Stride6(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: 0,
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            7 => SizedStrideNode::Stride7(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: U256(0, 0),
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            8 => SizedStrideNode::Stride8(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: U256(0, 0),
                pfxbitarr: U512(0, 0, 0, 0),
                ptr_vec: vec![],
                pfx_vec: vec![],
            }),
            _ => {
                panic!("unknown stride size encountered in STRIDES array");
            }
        };

        TreeBitMap(node)
    }

    // Partition for stride 4
    //
    // ptr bits never happen in the first half of the bitmap for the stride-size. Consequently the ptrbitarr can be an integer type
    // half the size of the pfxbitarr.
    //
    // ptr bit arr (u16)                                                        0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15    x
    // pfx bit arr (u32)   0 1 2  3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24   25   26   27   28   29   30   31
    // nibble              * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111    x
    // nibble len offset   0 1    2            3                                4
    //
    // stride 3: 1 + 2 + 4 + 8                              =  15 bits. 2^4 - 1 (1 << 4) - 1. ptrbitarr starts at pos  7 (1 << 3) - 1
    // stride 4: 1 + 2 + 4 + 8 + 16                         =  31 bits. 2^5 - 1 (1 << 5) - 1. ptrbitarr starts at pos 15 (1 << 4) - 1
    // stride 5: 1 + 2 + 4 + 8 + 16 + 32 + 64               =  63 bits. 2^6 - 1
    // stride 6: 1 + 2 + 4 + 8 + 16 + 32 + 64               = 127 bits. 2^7 - 1
    // stride 7: 1 + 2 + 4 + 8 + 16 + 32 + 64 = 128         = 256 bits. 2^8 - 1
    // stride 8: 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256   = 511 bits. 2^9 - 1
    //
    // Ex.:
    // pfx            65.0.0.252/30                                             0100_0001_0000_0000_0000_0000_1111_1100
    //
    // nibble 1       (pfx << 0) >> 28                                          0000_0000_0000_0000_0000_0000_0000_0100
    // bit_pos        (1 << nibble length) - 1 + nibble                         0000_0000_0000_0000_0000_1000_0000_0000
    //
    // nibble 2       (pfx << 4) >> 24                                          0000_0000_0000_0000_0000_0000_0000_0001
    // bit_pos        (1 << nibble length) - 1 + nibble                         0000_0000_0000_0000_1000_0000_0000_0000
    // ...
    // nibble 8       (pfx << 28) >> 0                                          0000_0000_0000_0000_0000_0000_0000_1100
    // bit_pos        (1 << nibble length) - 1 + nibble = (1 << 2) - 1 + 2 = 5  0000_0010_0000_0000_0000_0000_0000_0000
    // 5 - 5 - 5 - 4 - 4 - [4] - 5
    // startpos (2 ^ nibble length) - 1 + nibble as usize

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>) {
        // println!("");
        // println!("{:?}", pfx);
        // println!("             0   4   8   12  16  20  24  28  32  36  40  44  48  52  56  60  64  68  72  76  80  84  88  92  96 100 104 108 112 116 120 124 128");
        // println!("             |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
        let mut stride_end: u8 = 0;
        let mut node = &mut self.0;
        let mut strides = Self::STRIDES.iter().peekable();
        while let Some(stride) = strides.next() {
            stride_end += stride;

            let nibble_len = if pfx.len < stride_end {
                stride + pfx.len - stride_end
            } else {
                *stride
            };

            let nibble = AF::get_nibble(pfx.net, stride_end - stride, nibble_len);

            // Some(&mut self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)])

            node = match node {
                // pfx.len
                // stride_end
                // nibble,nibble_len
                SizedStrideNode::Stride3(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
                SizedStrideNode::Stride4(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
                SizedStrideNode::Stride5(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
                SizedStrideNode::Stride6(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
                SizedStrideNode::Stride7(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
                SizedStrideNode::Stride8(current_node) => match current_node.traverse(
                    nibble,
                    nibble_len,
                    pfx,
                    strides.peek(),
                    pfx.len <= stride_end,
                ) {
                    Some(n) => n,
                    None => {
                        return;
                    }
                },
            }
        }
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<AF, NoMeta>,
        // mut found_pfx: Vec<&'a Prefix<AF, T>>
    ) -> Vec<&'a Prefix<AF, T>> {
        let mut stride_end = 0;
        let mut found_pfx: Vec<&'a Prefix<AF, T>> = vec![];
        let mut node = &self.0;
        // println!("");
        // println!("{:?}", search_pfx);
        // println!("             0   4   8   12  16  20  24  28  32");
        // println!("             |---|---|---|---|---|---|---|---|");

        for stride in Self::STRIDES.iter() {
            stride_end += stride;

            let nibble_len = if search_pfx.len < stride_end {
                stride + search_pfx.len - stride_end
            } else {
                *stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
            // let mut nibble = (search_pfx.net << (stride_end - stride) as usize)
            //     >> (((Self::BITS - nibble_len) % Self::BITS) as usize);
            let nibble = AddressFamily::get_nibble(search_pfx.net, stride_end - stride, nibble_len);

            // let mut bit_pos = S::get_bit_pos(nibble, nibble_len);
            // let mut offset: u32 = (1_u32 << nibble_len) - 1;
            // let mut bit_pos: u32 = 0x1 << (Self::BITS - offset as u8 - nibble as u8 - 1);

            // In a LMP search we have to go over all the nibble lengths in the stride up
            // until the value of the actual nibble length were looking for (until we reach
            // stride length for all strides that aren't the last) and see if the
            // prefix bit in that posision is set.
            // Note that this does not search for prefixes with length 0 (which would always
            // match).
            // So for matching a nibble 1010, we have to search for 1, 10, 101 and 1010 on
            // resp. position 1, 5, 12 and 25:
            //                       ↓          ↓                         ↓                                                              ↓
            // pfx bit arr (u32)   0 1 2  3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24   25   26   27   28   29   30   31
            // nibble              * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111    x
            // nibble len offset   0 1    2            3                                4
            match node {
                // nibble, nibble_len, pfx,
                SizedStrideNode::Stride3(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx,
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
                SizedStrideNode::Stride4(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx,
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
                SizedStrideNode::Stride5(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx,
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
                SizedStrideNode::Stride6(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx,
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
                SizedStrideNode::Stride7(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
                SizedStrideNode::Stride8(current_node) => {
                    match current_node.search(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx,
                    ) {
                        Some(n) => {
                            node = n;
                        }
                        None => {
                            return found_pfx;
                        }
                    }
                }
            };
        }

        println!("=");
        found_pfx
    }
}
