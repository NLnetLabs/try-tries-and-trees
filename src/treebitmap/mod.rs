use crate::common::{AddressFamily, NoMeta, Prefix};
use num::{PrimInt, Zero};
use std::cmp::Ordering;
use std::fmt::{Binary, Debug};

// pub struct BitMap8stride(u8, 8);

// type Stride3 = u16;
pub type Stride4 = u32;
// type Stride5 = u64;
// type Stride6 = u128;

pub trait Stride {
    type PtrSize;
    type PfxSize;
    const BITS: u8;

    fn get_bit_pos(nibble: u32, len: u8) -> Self;
    fn get_pfx_index(nibble: u32, len: u8) -> usize;
    fn get_ptr_index(nibble: u32) -> usize;
    fn into_stride_size(bitmap: Self::PtrSize) -> Self;
    fn into_ptrbitarr_size(bitmap: Self) -> Self::PtrSize;
}

impl Stride for Stride4 {
    type PtrSize = u16;
    type PfxSize = u32;
    const BITS: u8 = 32;

    fn get_bit_pos(nibble: u32, len: u8) -> u32 {
        1 << (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1)
    }

    fn get_pfx_index(nibble: u32, len: u8) -> usize {
        // (1_u32 << len) - 1 is the offset for this nibble length (len).
        (<Self as Stride>::BITS - ((1 << len) - 1) as u8 - nibble as u8 - 1) as usize
    }

    fn get_ptr_index(nibble: u32) -> usize {
        ((<Self as Stride>::BITS >> 1) - nibble as u8 - 1) as usize
    }

    fn into_stride_size(bitmap: u16) -> u32 {
        bitmap as u32
    }

    fn into_ptrbitarr_size(bitmap: u32) -> u16 {
        bitmap as u16
    }
}
pub struct TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
    bit_id: u8,
    ptrbitarr: <S as Stride>::PtrSize,
    pfxbitarr: S,
    pfx_vec: Vec<&'a Prefix<AF, T>>,
    ptr_vec: Vec<Box<TreeBitMapNode<'a, AF, T, S>>>,
}

impl<'a, AF, T, S> Eq for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
}

impl<'a, AF, T, S> Ord for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.ptrbitarr.cmp(&other.ptrbitarr)
    }
}

impl<'a, AF, T, S> PartialOrd for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.bit_id.cmp(&other.bit_id))
    }
}

impl<'a, AF, T, S> PartialEq for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
    fn eq(&self, other: &Self) -> bool {
        self.bit_id == other.bit_id
    }
}

impl<'a, AF, T, S> Debug for TreeBitMapNode<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Stride + PrimInt + Debug + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
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
pub struct TreeBitMap<'a, AF, T, S>(TreeBitMapNode<'a, AF, T, S>)
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + PrimInt + Binary,
    S::PtrSize: PrimInt + Debug + Binary;

impl<'a, AF, T, S> TreeBitMap<'a, AF, T, S>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
    S: Debug + Stride + PrimInt + Binary,
    <S as Stride>::PtrSize: PrimInt + Debug + Binary,
{
    const STRIDES: [u8; 8] = [4; 8];
    pub fn new() -> TreeBitMap<'a, AF, T, S> {
        // Check if the strides division makes sense
        assert!(Self::STRIDES.iter().fold(0, |acc, s| { acc + s }) == S::BITS);
        TreeBitMap(TreeBitMapNode {
            bit_id: 0,
            ptrbitarr: S::PtrSize::zero(),
            pfxbitarr: S::zero(),
            ptr_vec: vec![Box::new(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: S::PtrSize::zero(),
                pfxbitarr: S::zero(),
                ptr_vec: vec![],
                pfx_vec: vec![],
            })],
            pfx_vec: vec![],
        })
    }

    // Partition for stride 4
    //
    // ptr bits never happen in the first half of the bitmap for the stride-size. Consequently the ptrbitarr can be an integer type
    // half the size of the pfxbitarr.
    //
    // ptr bit pos (u16)                                                        0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15    x
    // pfx bit pos (u32)   0 1 2  3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24   25   26   27   28   29   30   31
    // nibble              * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111    x
    // nibble length       0 1    2            3                                4
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

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>)
    where
        S: Stride + PrimInt,
        <S as Stride>::PtrSize: PrimInt,
    {
        // println!("");
        // println!("{:?}", pfx);
        // println!("             0   4   8   12  16  20  24  28  32");
        // println!("             |---|---|---|---|---|---|---|---|");
        let mut stride_end: u8 = 0;
        let mut current_node = self.0.ptr_vec.first_mut().unwrap();
        for stride in Self::STRIDES.iter() {
            stride_end += stride;

            let nibble_len = if pfx.len < stride_end {
                stride + pfx.len - stride_end
            } else {
                *stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
            // let nibble = pfx.net << (stride_end - stride) as usize
            //     >> (((AF::BITS - nibble_len) % AF::BITS) as usize);

            let nibble = AddressFamily::get_nibble(pfx.net, stride_end - stride, nibble_len);

            // Move the bit in the right position.
            let bit_pos = S::get_bit_pos(nibble, nibble_len);

            // println!("n {:b} nl {}", nibble, nibble_len);

            // Check if we're at the last stride (pfx.len > stride_end)
            if pfx.len > stride_end {
                // We are not at the last stride
                // Check it the ptr bit is already set in this position
                if (S::from(current_node.ptrbitarr).unwrap() << 1) & bit_pos == S::zero() {
                    // Nope, set it and create a child node
                    // Note that bitwise operators align bits of unsigend types with different
                    // sizes to the right, so we don't have to do anything to pad the smaller sized
                    // type.
                    current_node.ptrbitarr = S::into_ptrbitarr_size(
                        (bit_pos | ((S::from(current_node.ptrbitarr).unwrap()) << 1)) >> 1,
                    );

                    // println!(
                    //     "__ptr[{:02}]__: xxxxxxxxxxxxxxx{:016b}x",
                    //     stride_end, current_node.ptrbitarr
                    // );
                    // println!("bit_id: {}", bit_pos.leading_zeros());
                    let new_node = Box::new(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u8,
                        ptrbitarr: S::PtrSize::zero(),
                        pfxbitarr: S::zero(),
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    });
                    current_node.ptr_vec.push(new_node);
                    current_node.ptr_vec.sort();
                }
            } else {
                // only at the last stride do we create the bit in the prefix bitmap,
                // and only if it doesn't exist already
                if current_node.pfxbitarr & bit_pos == S::zero() {
                    current_node.pfxbitarr = bit_pos | current_node.pfxbitarr;
                    // println!(
                    //     "pfx[{:02}]n[{}]: {:032b}",
                    //     stride_end, nibble_len, current_node.pfxbitarr
                    // );

                    current_node.pfx_vec.push(pfx);
                    current_node.pfx_vec.sort();
                    // println!("{:?}", current_node.pfx_vec);
                }
                return;
            }

            // println!("__bit_pos__: {:032b}", bit_pos);
            // println!(
            //     "__ptr[{:02}]__: xxxxxxxxxxxxxxx{:016b}x",
            //     stride_end, current_node.ptrbitarr
            // );
            // // println!("{:?}", current_node.ptr_vec);
            // println!("{}", (S::BITS >> 1) - nibble as u8 - 1);
            // println!(
            //     "{}",
            //     (current_node.ptrbitarr >> ((S::BITS >> 1) - nibble as u8 - 1) as usize).count_ones()
            // );
            let next_index = (current_node.ptrbitarr
                >> ((S::BITS >> 1) - nibble as u8 - 1) as usize)
                .count_ones() as u32;

            // println!("{}", next_index);
            // println!("{:?}", current_node.ptr_vec);
            current_node = &mut current_node.ptr_vec[next_index as usize - 1];
        }
    }

    pub fn match_longest_prefix(&self, search_pfx: &Prefix<AF, NoMeta>) -> Option<&'a Prefix<AF, T>>
    where
        <S as Stride>::PtrSize: PrimInt,
    {
        let mut found_pfx = None;
        let mut stride_end = 0;
        let mut current_node = self.0.ptr_vec.first().unwrap();
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
            let mut nibble =
                AddressFamily::get_nibble(search_pfx.net, stride_end - stride, nibble_len);

            let mut bit_pos = S::get_bit_pos(nibble, nibble_len);
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
            // pfx bit pos (u32)   0 1 2  3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24   25   26   27   28   29   30   31
            // nibble              * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111    x
            // nibble length       0 1    2            3                                4

            for n_l in 1..(nibble_len + 1) {
                // Move the bit in the right position.

                // Shift left and right to set the bits to zero that are not
                // in the nibble we're handling here.
                // nibble = (search_pfx.net << (stride_end - stride) as usize)
                //     >> (((Self::BITS - n_l) % AF::BITS) as usize);
                nibble = AddressFamily::get_nibble(search_pfx.net, stride_end - stride, n_l);

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
                // println!("___pfx:      {:032b}", current_node.pfxbitarr);
                // println!(
                //     "___ptr:      xxxxxxxxxxxxxxx{:016b}x",
                //     current_node.ptrbitarr
                // );
                // println!("{:?}", current_node.pfx_vec);

                // Check it there's an prefix matching in this bitmap for this nibble
                if current_node.pfxbitarr & bit_pos > S::zero() {
                    found_pfx = Some(
                        current_node.pfx_vec[(current_node.pfxbitarr >> S::get_pfx_index(nibble, n_l))
                            // >> (Self::BITS - offset as u8 - nibble as u8 - 1))
                            .count_ones() as usize
                            - 1],
                    );
                    println!("found: {:?}", found_pfx.unwrap());
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
            }

            // Check if this the last stride, if so return what we found up until now
            if search_pfx.len < (stride_end - stride)
                || (((S::into_stride_size(current_node.ptrbitarr)) << 1) & bit_pos) == S::zero()
            {
                return found_pfx;
            }

            // shift all the bits away that do not concern this ptr, so we can count
            // the ones left to the bit cursor.
            let next_index = (current_node.ptrbitarr
                >> ((S::BITS >> 1) - nibble as u8 - 1) as usize)
                .count_ones();

            current_node = &current_node.ptr_vec[next_index as usize - 1];
        }

        println!("=");
        found_pfx
    }
}

pub mod allocator;
