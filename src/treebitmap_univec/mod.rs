use crate::common::{AddressFamily, NoMeta, Prefix};
use std::cmp::Ordering;
use std::fmt::{Binary, Debug};

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

pub trait Stride: Sized + Debug + Binary + Eq + PartialOrd + PartialEq + Copy {
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
                U256(0, 1 << n)
            }
            n => {
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
            n => (bitmap.0 >> (n - 128)).count_ones() as usize - 1,
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
                U512(0, 0, 0, 1 << n)
            }
            n if n < 256 => U512(0, 0, 1 << (n as u16 - 128), 0),
            n if n < 384 => {
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
                    + (bitmap.2 >> (n - 128)).count_ones() as usize
                    - 1
            }

            n if n < 384 => {
                bitmap.0.count_ones() as usize + (bitmap.1 >> (n - 256)).count_ones() as usize - 1
            }

            // if we move more than 384 bits to the right,
            // all of bitmap.[1,2,3] will be shifted out of sight,
            // so we only have to count bitmap.0 zeroes then (after shifting of course).
            n => {
                (bitmap.0 >> (n - 384)).count_ones() as usize - 1
            }
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
            n => (bitmap.0 >> (n - 128)).count_ones() as usize - 1,
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
pub enum SizedStrideNode<AF: AddressFamily> {
    Stride3(TreeBitMapNode<AF, Stride3>),
    Stride4(TreeBitMapNode<AF, Stride4>),
    Stride5(TreeBitMapNode<AF, Stride5>),
    Stride6(TreeBitMapNode<AF, Stride6>),
    Stride7(TreeBitMapNode<AF, Stride7>),
    Stride8(TreeBitMapNode<AF, Stride8>),
}

pub struct TreeBitMapNode<AF, S>
where
    S: Stride,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
    AF: AddressFamily,
{
    ptrbitarr: <S as Stride>::PtrSize,
    pfxbitarr: S,
    // The vec of prefixes hosted by this node,
    // referenced by (bit_id, global prefix index)
    // We need the AF typed value to sort the vec
    // that is stored in the node.
    pfx_vec: Vec<(AF, u32)>,
    // The vec of child nodes hosted by this
    // node, referenced by (ptrbitarr_index, global vec index)
    // We need the u16 (ptrbitarr_index) to sort the
    // vec that's stored in the node.
    ptr_vec: Vec<(u16, u32)>,
}

impl<AF> Default for SizedStrideNode<AF>
where
    AF: AddressFamily,
{
    fn default() -> Self {
        SizedStrideNode::Stride3(TreeBitMapNode {
            ptrbitarr: 0,
            pfxbitarr: 0,
            pfx_vec: vec![],
            ptr_vec: vec![],
        })
    }
}

impl<AF, S> Debug for TreeBitMapNode<AF, S>
where
    AF: AddressFamily,
    S: Stride,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeBitMapNode")
            .field("ptrbitarr", &self.ptrbitarr)
            .field("pfxbitarr", &self.pfxbitarr)
            .field("ptr_vec", &self.ptr_vec)
            .finish()
    }
}

enum NewNodeOrIndex<AF: AddressFamily> {
    NewNode(SizedStrideNode<AF>, u16), // New Node and bit_id of the new node
    ExistingNode(u32),
    NewPrefix,
    ExistingPrefix,
}

impl<AF, S> TreeBitMapNode<AF, S>
where
    AF: AddressFamily,
    S: Stride + std::ops::BitAnd<Output = S> + std::ops::BitOr<Output = S>,
    <S as Stride>::PtrSize: Debug + Binary + Copy,
{
    // Inspects the stride (nibble, nibble_len) to see it there's
    // already a child node (if not at the last stride) or a prefix
    //  (if it's the last stride).
    //
    // Returns one of:
    // - A newly created child node.
    // - The index of the existing child node in the global `nodes` vec
    // - A newly created Prefix
    // - The index of the existing prefix in the global `prefixes` vec
    fn eval_node_or_prefix_at(
        self: &mut Self,
        nibble: u32,
        nibble_len: u8,
        next_stride: Option<&u8>,
        is_last_stride: bool,
    ) -> NewNodeOrIndex<AF> {
        let bit_pos = S::get_bit_pos(nibble, nibble_len);
        let new_node: SizedStrideNode<AF>;

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

                match next_stride.unwrap() {
                    3_u8 => {
                        new_node = SizedStrideNode::Stride3(TreeBitMapNode {
                            ptrbitarr: <Stride3 as Stride>::PtrSize::zero(),
                            pfxbitarr: Stride3::zero(),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    4_u8 => {
                        new_node = SizedStrideNode::Stride4(TreeBitMapNode {
                            ptrbitarr: <Stride4 as Stride>::PtrSize::zero(),
                            pfxbitarr: Stride4::zero(),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    5_u8 => {
                        new_node = SizedStrideNode::Stride5(TreeBitMapNode {
                            ptrbitarr: <Stride5 as Stride>::PtrSize::zero(),
                            pfxbitarr: Stride5::zero(),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    6_u8 => {
                        new_node = SizedStrideNode::Stride6(TreeBitMapNode {
                            ptrbitarr: <Stride6 as Stride>::PtrSize::zero(),
                            pfxbitarr: Stride6::zero(),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    7_u8 => {
                        new_node = SizedStrideNode::Stride7(TreeBitMapNode {
                            ptrbitarr: 0_u128,
                            pfxbitarr: U256(0_u128, 0_u128),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    8_u8 => {
                        new_node = SizedStrideNode::Stride8(TreeBitMapNode {
                            ptrbitarr: U256(0_u128, 0_u128),
                            pfxbitarr: U512(0_u128, 0_u128, 0_u128, 0_u128),
                            pfx_vec: vec![],
                            ptr_vec: vec![],
                        });
                    }
                    _ => {
                        panic!("can't happen");
                    }
                };

                // we can return bit_pos.leading_zeros() since bit_pos is the bitmap that
                // points to the current bit in ptrbitarr (it's **not** the prefix of the node!),
                // so the number of zeros in front of it should always be unique and describes
                // the index of this node in the ptrbitarr.
                // ex.:
                // In a stride3 (ptrbitarr lenght is 8):
                // bit_pos 0001 0000
                // so this is the fourth bit, so points to index = 3
                return NewNodeOrIndex::NewNode(new_node, bit_pos.leading_zeros() as u16);
            }
        } else {
            // only at the last stride do we create the bit in the prefix bitmap,
            // and only if it doesn't exist already
            if self.pfxbitarr & bit_pos == <S as std::ops::BitAnd>::Output::zero() {
                self.pfxbitarr = bit_pos | self.pfxbitarr;

                return NewNodeOrIndex::NewPrefix;
            }
            return NewNodeOrIndex::ExistingPrefix;
        }

        NewNodeOrIndex::ExistingNode(self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)].1)
    }

    fn search_stride_at<'b>(
        self: &Self,
        search_pfx: &Prefix<AF, NoMeta>,
        mut nibble: u32,
        nibble_len: u8,
        start_bit: u8,
        // found_pfx: &'b mut Vec<&'a Prefix<AF, T>>,
        found_pfx: &'b mut Vec<(AF, u32)>,
    ) -> Option<u32> {
        let mut bit_pos = S::get_bit_pos(nibble, nibble_len);

        for n_l in 1..(nibble_len + 1) {
            // Move the bit in the right position.
            nibble = AddressFamily::get_nibble(search_pfx.net, start_bit, n_l);
            bit_pos = S::get_bit_pos(nibble, n_l);

            // Check it there's an prefix matching in this bitmap for this nibble
            if self.pfxbitarr & bit_pos > S::zero() {
                found_pfx.push(self.pfx_vec[S::get_pfx_index(self.pfxbitarr, nibble, n_l)]);
            }
        }

        // Check if this the last stride, or if they're no more children to go to,
        // if so return what we found up until now.
        // let SizedStrideNode::Stride4(current_node) = node;
        if search_pfx.len < start_bit
            || (S::into_stride_size(self.ptrbitarr) & bit_pos)
                == <S as std::ops::BitAnd>::Output::zero()
        {
            return None;
        }

        Some(self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)].1)
    }

    fn search_stride_at_lmp_only<'b>(
        self: &Self,
        search_pfx: &Prefix<AF, NoMeta>,
        mut nibble: u32,
        nibble_len: u8,
        start_bit: u8,
    ) -> (Option<u32>, Option<(AF, u32)>) {
        let mut bit_pos = S::get_bit_pos(nibble, nibble_len);
        let mut found_pfx = None;

        for n_l in 1..(nibble_len + 1) {
            // Move the bit in the right position.
            nibble = AddressFamily::get_nibble(search_pfx.net, start_bit, n_l);
            bit_pos = S::get_bit_pos(nibble, n_l);

            // Check if the prefix has been set, if so select the prefix. This is not
            // necessarily the final prefix that will be returned.

            // Check it there's an prefix matching in this bitmap for this nibble
            if self.pfxbitarr & bit_pos > S::zero() {
                found_pfx = Some(self.pfx_vec[S::get_pfx_index(self.pfxbitarr, nibble, n_l)]);
            }
        }

        // Check if this the last stride, or if they're no more children to go to,
        // if so return what we found up until now.
        if search_pfx.len < start_bit
            || (S::into_stride_size(self.ptrbitarr) & bit_pos)
                == <S as std::ops::BitAnd>::Output::zero()
        {
            return (None, found_pfx);
        }

        (
            Some(self.ptr_vec[S::get_ptr_index(self.ptrbitarr, nibble)].1),
            found_pfx,
        )
    }
}
pub struct TreeBitMap<AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug,
{
    // root: SizedStrideNode<AF, T>,
    pub strides: Vec<u8>,
    pub stats: Vec<StrideStats>,
    pub nodes: Vec<SizedStrideNode<AF>>,
    pub prefixes: Vec<Prefix<AF, T>>,
}

impl<'a, AF, T> TreeBitMap<AF, T>
where
    T: Debug,
    AF: AddressFamily + Debug,
{
    pub fn new(_strides_vec: Vec<u8>) -> TreeBitMap<AF, T> {
        // Check if the strides division makes sense
        let mut strides = vec![];
        let mut strides_sum = 0;
        for s in _strides_vec.iter().cycle() {
            strides.push(*s);
            strides_sum += s;
            if strides_sum >= AF::BITS - 1 {
                break;
            }
        }
        assert_eq!(strides.iter().fold(0, |acc, s| { acc + s }), AF::BITS);

        let mut stride_stats: Vec<StrideStats> = vec![
            StrideStats::new(SizedStride::Stride3, strides.len() as u8), // 0
            StrideStats::new(SizedStride::Stride4, strides.len() as u8), // 1
            StrideStats::new(SizedStride::Stride5, strides.len() as u8), // 2
            StrideStats::new(SizedStride::Stride6, strides.len() as u8), // 3
            StrideStats::new(SizedStride::Stride7, strides.len() as u8), // 4
            StrideStats::new(SizedStride::Stride8, strides.len() as u8), // 5
        ];

        let node: SizedStrideNode<AF>;

        match strides[0] {
            3 => {
                node = SizedStrideNode::Stride3(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: 0,
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[0].inc(0);
            }
            4 => {
                node = SizedStrideNode::Stride4(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: 0,
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[1].inc(0);
            }
            5 => {
                node = SizedStrideNode::Stride5(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: 0,
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[2].inc(0);
            }
            6 => {
                node = SizedStrideNode::Stride6(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: 0,
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[3].inc(0);
            }
            7 => {
                node = SizedStrideNode::Stride7(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: U256(0, 0),
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[4].inc(0);
            }
            8 => {
                node = SizedStrideNode::Stride8(TreeBitMapNode {
                    ptrbitarr: U256(0, 0),
                    pfxbitarr: U512(0, 0, 0, 0),
                    ptr_vec: vec![],
                    pfx_vec: vec![],
                });
                stride_stats[5].inc(0);
            }
            _ => {
                panic!("unknown stride size encountered in STRIDES array");
            }
        };

        TreeBitMap {
            strides,
            stats: stride_stats,
            nodes: vec![node],
            prefixes: vec![],
        }
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
    // stride 7: 1 + 2 + 4 + 8 + 16 + 32 + 64 = 128         = 256 bits. 2^8 - 1126
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

    pub fn insert(&mut self, pfx: Prefix<AF, T>) {
        let mut stride_end: u8 = 0;
        let mut cur_i = 0;
        let mut node = std::mem::take(self.retrieve_node_mut(cur_i).unwrap());

        let mut level: u8 = 0;
        let pfx_len = pfx.len.clone();
        let pfx_net = pfx.net.clone();

        loop {
            let stride = self.strides[level as usize];
            let next_stride = self.strides.get((level + 1) as usize);

            stride_end += stride;

            let nibble_len = if pfx_len < stride_end {
                stride + pfx_len - stride_end
            } else {
                stride
            };

            let nibble = AF::get_nibble(pfx_net, stride_end - stride, nibble_len);

            let is_last_stride = pfx_len <= stride_end;

            let (next_node_idx, cur_node) = match node {
                SizedStrideNode::Stride3(mut current_node) => match current_node
                    .eval_node_or_prefix_at(nibble, nibble_len, next_stride, is_last_stride)
                {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[0].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();
                        (Some(i), SizedStrideNode::Stride3(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride3(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[0].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride3(current_node),
                        );
                        return;
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride3(current_node))
                    }
                },
                SizedStrideNode::Stride4(mut current_node) => match current_node
                    .eval_node_or_prefix_at(
                        nibble,
                        nibble_len,
                        // No, next_stride.is_none does *not* mean that it's the last stride
                        // There may very well be a Some(next_stride), next_stride goes all the
                        // way to the end of the length of the network address space (like 32 bits for IPv4 etc),
                        // whereas the last stride stops at the end of the prefix length.
                        // `is_last_stride` is an indicator for the upsert function to write the prefix in the
                        // node's vec.
                        next_stride,
                        pfx_len <= stride_end,
                    ) {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[1].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();
                        (Some(i), SizedStrideNode::Stride4(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride4(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[1].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride4(current_node),
                        );
                        return;
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride4(current_node))
                    }
                },
                SizedStrideNode::Stride5(mut current_node) => match current_node
                    .eval_node_or_prefix_at(
                        nibble,
                        nibble_len,
                        next_stride,
                        pfx_len <= stride_end,
                    ) {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[2].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();
                        (Some(i), SizedStrideNode::Stride5(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride5(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[2].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride5(current_node),
                        );
                        return;
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride5(current_node))
                    }
                },
                SizedStrideNode::Stride6(mut current_node) => match current_node
                    .eval_node_or_prefix_at(
                        nibble,
                        nibble_len,
                        next_stride,
                        pfx_len <= stride_end,
                    ) {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[3].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();
                        (Some(i), SizedStrideNode::Stride6(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride6(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[3].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort_unstable();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride6(current_node),
                        );
                        return;
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride6(current_node))
                    }
                },
                SizedStrideNode::Stride7(mut current_node) => match current_node
                    .eval_node_or_prefix_at(
                        nibble,
                        nibble_len,
                        next_stride,
                        pfx_len <= stride_end,
                    ) {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[4].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();
                        (Some(i), SizedStrideNode::Stride7(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride7(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[4].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride7(current_node),
                        );
                        return;
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride7(current_node))
                    }
                },
                SizedStrideNode::Stride8(mut current_node) => match current_node
                    .eval_node_or_prefix_at(
                        nibble,
                        nibble_len,
                        next_stride,
                        pfx_len <= stride_end,
                    ) {
                    NewNodeOrIndex::NewNode(n, bit_id) => {
                        self.stats[5].inc(level);
                        let i = self.store_node(n);
                        current_node.ptr_vec.push((bit_id, i));
                        current_node.ptr_vec.sort();

                        (Some(i), SizedStrideNode::Stride8(current_node))
                    }
                    NewNodeOrIndex::ExistingNode(i) => {
                        (Some(i), SizedStrideNode::Stride8(current_node))
                    }
                    NewNodeOrIndex::ExistingPrefix => {
                        (None, SizedStrideNode::Stride8(current_node))
                    }
                    NewNodeOrIndex::NewPrefix => {
                        let i = self.store_prefix(pfx);
                        self.stats[5].inc_prefix_count(level);
                        current_node
                            .pfx_vec
                            .push(((pfx_net >> (AF::BITS - pfx_len) as usize), i));
                        current_node.pfx_vec.sort();
                        let _default_val = std::mem::replace(
                            self.retrieve_node_mut(cur_i).unwrap(),
                            SizedStrideNode::Stride8(current_node),
                        );
                        return;
                    }
                },
            };

            let _default_val = std::mem::replace(self.retrieve_node_mut(cur_i).unwrap(), cur_node);

            if let Some(i) = next_node_idx {
                node = std::mem::take(self.retrieve_node_mut(i).unwrap());
                cur_i = i;
                level += 1;
            } else {
                return;
            }
        }
    }

    pub fn store_node(&mut self, next_node: SizedStrideNode<AF>) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(next_node);
        id
    }

    #[inline]
    pub fn retrieve_node(&self, index: u32) -> Option<&SizedStrideNode<AF>> {
        self.nodes.get(index as usize)
    }

    #[inline]
    pub fn retrieve_node_mut(&mut self, index: u32) -> Option<&mut SizedStrideNode<AF>> {
        self.nodes.get_mut(index as usize)
    }

    pub fn store_prefix(&mut self, next_node: Prefix<AF, T>) -> u32 {
        let id = self.prefixes.len() as u32;
        self.prefixes.push(next_node);
        id
    }

    #[inline]
    pub fn retrieve_prefix(&'a self, index: u32) -> Option<&'a Prefix<AF, T>> {
        self.prefixes.get(index as usize)
    }

    #[inline]
    pub fn retrieve_prefix_mut(&mut self, index: u32) -> Option<&mut Prefix<AF, T>> {
        self.prefixes.get_mut(index as usize)
    }

    pub fn match_longest_prefix(
        &'a self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Vec<&'a Prefix<AF, T>> {
        let mut stride_end = 0;
        let mut found_pfx_idxs: Vec<(AF, u32)> = vec![];
        let mut node = self.retrieve_node(0).unwrap();

        for stride in self.strides.iter() {
            stride_end += stride;

            let nibble_len = if search_pfx.len < stride_end {
                stride + search_pfx.len - stride_end
            } else {
                *stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
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
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .into_iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
                SizedStrideNode::Stride4(current_node) => {
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
                SizedStrideNode::Stride5(current_node) => {
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
                SizedStrideNode::Stride6(current_node) => {
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
                SizedStrideNode::Stride7(current_node) => {
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
                SizedStrideNode::Stride8(current_node) => {
                    match current_node.search_stride_at(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                        &mut found_pfx_idxs,
                    ) {
                        Some(n) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        None => {
                            return found_pfx_idxs
                                .iter()
                                .map(|i| self.retrieve_prefix(i.1).unwrap())
                                .collect();
                        }
                    }
                }
            };
        }

        found_pfx_idxs
            .iter()
            .map(|i| self.retrieve_prefix(i.1).unwrap())
            .collect()
    }

    pub fn match_longest_prefix_only(
        &'a self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Option<&'a Prefix<AF, T>> {
        let mut stride_end = 0;
        let mut found_pfx_idx: Option<u32> = None;
        let mut node = self.retrieve_node(0).unwrap();
  
        for stride in self.strides.iter() {
            stride_end += stride;

            let nibble_len = if search_pfx.len < stride_end {
                stride + search_pfx.len - stride_end
            } else {
                *stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
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
                SizedStrideNode::Stride3(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
                SizedStrideNode::Stride4(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
                SizedStrideNode::Stride5(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
                SizedStrideNode::Stride6(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
                SizedStrideNode::Stride7(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
                SizedStrideNode::Stride8(current_node) => {
                    match current_node.search_stride_at_lmp_only(
                        search_pfx,
                        nibble,
                        nibble_len,
                        stride_end - stride,
                    ) {
                        (Some(n), Some(pfx_idx)) => {
                            found_pfx_idx = Some(pfx_idx.1);
                            node = self.retrieve_node(n).unwrap();
                        }
                        (Some(n), None) => {
                            node = self.retrieve_node(n).unwrap();
                        }
                        (None, Some(pfx_idx)) => {
                            return Some(self.retrieve_prefix(pfx_idx.1).unwrap())
                        }
                        (None, None) => {
                            break;
                        }
                    }
                }
            };
        }

        if let Some(pfx_idx) = found_pfx_idx {
            Some(self.retrieve_prefix(pfx_idx).unwrap())
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SizedStride {
    Stride3,
    Stride4,
    Stride5,
    Stride6,
    Stride7,
    Stride8,
}
pub struct StrideStats {
    pub stride_type: SizedStride,
    pub stride_size: usize,
    pub stride_len: u8,
    pub node_size: usize,
    pub created_nodes: Vec<CreatedNodes>,
    pub prefixes_num: Vec<CreatedNodes>,
}

impl StrideStats {
    pub fn new(stride_type: SizedStride, num_depth_levels: u8) -> Self {
        match stride_type {
            SizedStride::Stride3 => Self {
                stride_type: SizedStride::Stride3,
                stride_size: 16,
                stride_len: 3,
                node_size: std::mem::size_of::<Stride3>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
            SizedStride::Stride4 => Self {
                stride_type: SizedStride::Stride4,
                stride_size: 32,
                stride_len: 4,
                node_size: std::mem::size_of::<Stride4>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
            SizedStride::Stride5 => Self {
                stride_type: SizedStride::Stride5,
                stride_size: 64,
                stride_len: 5,
                node_size: std::mem::size_of::<Stride5>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
            SizedStride::Stride6 => Self {
                stride_type: SizedStride::Stride6,
                stride_size: 128,
                stride_len: 6,
                node_size: std::mem::size_of::<Stride6>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
            SizedStride::Stride7 => Self {
                stride_type: SizedStride::Stride7,
                stride_size: 256,
                stride_len: 7,
                node_size: std::mem::size_of::<Stride7>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
            SizedStride::Stride8 => Self {
                stride_type: SizedStride::Stride8,
                stride_size: 512,
                stride_len: 8,
                node_size: std::mem::size_of::<Stride8>(),
                created_nodes: Self::nodes_vec(num_depth_levels),
                prefixes_num: Self::nodes_vec(num_depth_levels),
            },
        }
    }

    pub fn mem_usage(&self) -> usize {
        self.stride_size
            * self.created_nodes.iter().fold(0, |mut acc, c| {
                acc += c.count;
                acc
            })
    }

    fn nodes_vec(num_depth_levels: u8) -> Vec<CreatedNodes> {
        let mut vec: Vec<CreatedNodes> = vec![];
        for n in 0..num_depth_levels {
            vec.push(CreatedNodes {
                depth_level: n,
                count: 0,
            })
        }
        vec
    }

    fn inc(&mut self, depth_level: u8) {
        self.created_nodes[depth_level as usize].count += 1;
    }

    fn inc_prefix_count(&mut self, depth_level: u8) {
        self.prefixes_num[depth_level as usize].count += 1;
    }
}

impl Debug for StrideStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}:{:>8?} {:?} ({}k)",
            &self.stride_type,
            &self.created_nodes.iter().fold(0, |mut a, n| {
                a += n.count;
                a
            }),
            &self.created_nodes,
            &self.mem_usage() / 1024
        )
    }
}

#[derive(Copy, Clone)]
pub struct CreatedNodes {
    pub depth_level: u8,
    pub count: usize,
}

impl CreatedNodes {
    pub fn add(mut self, num: usize) {
        self.count += num;
    }
}

impl Debug for CreatedNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.count))
    }
}
