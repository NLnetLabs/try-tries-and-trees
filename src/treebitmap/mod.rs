use crate::common::{AddressFamily, Prefix};
use num::PrimInt;
use std::cmp::Ordering;
use std::fmt::Debug;

// pub struct BitMap8stride(u8, 8);

type BitMap4stride = u32;

pub struct TreeBitMapNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    bit_id: u8,
    ptrbitarr: BitMap4stride,
    pfxbitarr: BitMap4stride,
    pfx_vec: Vec<&'a Prefix<AF, T>>,
    ptr_vec: Vec<Box<TreeBitMapNode<'a, AF, T>>>,
}

impl<'a, AF, T> Eq for TreeBitMapNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
}

impl<'a, AF, T> Ord for TreeBitMapNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.ptrbitarr.cmp(&other.ptrbitarr)
    }
}

impl<'a, AF, T> PartialOrd for TreeBitMapNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.bit_id.cmp(&other.bit_id))
    }
}

impl<'a, AF, T> PartialEq for TreeBitMapNode<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    fn eq(&self, other: &Self) -> bool {
        self.bit_id == other.bit_id
    }
}
pub struct TreeBitMap<'a, AF, T>(TreeBitMapNode<'a, AF, T>)
where
    T: Debug,
    AF: AddressFamily + PrimInt;

impl<'a, T> TreeBitMap<'a, u32, T>
where
    T: Debug,
{
    const BITS: u8 = 32;
    pub fn new() -> TreeBitMap<'a, u32, T> {
        TreeBitMap(TreeBitMapNode {
            bit_id: 0,
            ptrbitarr: 0,
            pfxbitarr: 0,
            ptr_vec: vec![Box::new(TreeBitMapNode {
                bit_id: 0,
                ptrbitarr: 0,
                pfxbitarr: 0,
                ptr_vec: vec![],
                pfx_vec: vec![],
            })],
            pfx_vec: vec![],
        })
    }

    // 0 1 2 3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24    25  26    27  28   29   30
    // * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111
    // stride 3: 1 + 2 + 4 + 8                              =  15 bits. 2^4 - 1 (1 << 4) - 1. ptrbitarr starts at pos  7 (1 << 3) - 1
    // stride 4: 1 + 2 + 4 + 8 + 16                         =  31 bits. 2^5 - 1 (1 << 5) - 1. ptrbitarr starts at pos 15 (1 << 4) - 1
    // stride 5: 1 + 2 + 4 + 8 + 16 + 32 + 64               =  63 bits. 2^6 - 1
    // stride 6: 1 + 2 + 4 + 8 + 16 + 32 + 64               = 127 bits. 2^7 - 1
    // stride 7: 1 + 2 + 4 + 8 + 16 + 32 + 64 = 128         = 256 bits. 2^8 - 1
    // stride 8: 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256   = 511 bits. 2^9 - 1
    // 5 - 5 - 5 - 4 - 4 - [4] - 5
    // startpos (2 ^ stride_depth) - 1 + nibble as usize
    pub fn insert(&mut self, pfx: &'a Prefix<u32, T>) {
        let mut stride_end = 0;
        let mut current_node = self.0.ptr_vec.first_mut().unwrap();
        for stride in vec![4; 8] {
            stride_end += stride;

            let nibble_len = if pfx.len < stride_end {
                stride_end - pfx.len
            } else {
                stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
            let nibble =
                (pfx.net << (stride_end - nibble_len)) >> ((Self::BITS - nibble_len) as usize);

            // Move the bit in the right position.
            let offset: u8 = (0x1 << nibble_len) - 1; // 15 for a full stride of 4
            let bit_pos: u32 = 0x1 << (Self::BITS - offset as u8 - nibble as u8 - 1);

            print!("[{}]", stride,);

            // We're not at the end of the Prefix, so continue.
            if pfx.len > stride_end {
                // Check it the ptr bit is already set
                if current_node.ptrbitarr & bit_pos > 0 {
                    print!(".");
                } else {
                    // Nope, set it.
                    current_node.ptrbitarr = bit_pos | current_node.ptrbitarr;
                    print!("+");
                    let new_node = Box::new(TreeBitMapNode {
                        bit_id: bit_pos.leading_zeros() as u8,
                        ptrbitarr: 0,
                        pfxbitarr: 0,
                        pfx_vec: vec![],
                        ptr_vec: vec![],
                    });
                    current_node.ptr_vec.push(new_node);
                    current_node.ptr_vec.sort();
                }
            } else {
                current_node.pfxbitarr = bit_pos | current_node.pfxbitarr;
                current_node.pfx_vec.push(pfx);
                current_node.pfx_vec.sort();
                println!("={:?}", pfx);
                return;
            }

            let next_index = current_node.ptrbitarr.count_ones() as usize;
            current_node = &mut current_node.ptr_vec[next_index - 1];
        }
    }

    pub fn match_longest_prefix(&self, search_pfx: &Prefix<u32, T>) -> Option<&'a Prefix<u32, T>> {
        let mut found_pfx = None;
        let mut stride_end = 0;
        let mut current_node = self.0.ptr_vec.first().unwrap();
        for stride in vec![4; 8] {
            print!(".");
            stride_end += stride;

            let nibble_len = if search_pfx.len < stride_end {
                stride_end - search_pfx.len
            } else {
                stride
            };

            // Shift left and right to set the bits to zero that are not
            // in the nibble we're handling here.
            let nibble = (search_pfx.net << (stride_end - nibble_len))
                >> ((Self::BITS - nibble_len) as usize);

            // Move the bit that we're concerned about now in the right position in the mask,
            // without any other bits from the nibble.
            let offset: u8 = (2 << (nibble_len - 1)) - 1;
            let bit_pos: u32 = 0x1 << (Self::BITS - offset as u8 - nibble as u8 - 1);

            // Check if the prefix has been set, if so select the prefix. This is not
            // necessarily the final prefix that will be returned.
            if current_node.pfxbitarr & bit_pos > 0 {
                print!("f");
                found_pfx =
                    Some(current_node.pfx_vec[current_node.pfxbitarr.count_ones() as usize - 1]);
            }

            // If we are at the end of the prefix length or if there are no more
            // children we're returning what we found so far.
            if search_pfx.len <= stride_end || current_node.ptrbitarr == 0 {
                print!("|");
                return found_pfx;
            }

            let next_index = current_node.ptrbitarr.count_ones() as usize;

            current_node = &current_node.ptr_vec[next_index - 1];
        }

        println!("=");
        found_pfx
    }
}

pub mod allocator;
