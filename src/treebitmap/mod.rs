use crate::common::{AddressFamily, Prefix};
use num::PrimInt;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::{
    borrow::Borrow,
    fmt::{Binary, Debug},
};

// pub struct BitMap8stride(u8, 8);

type BitMap4stride = u32;
pub struct TreeBitMapNode
{
    ptrbitarr: BitMap4stride,
    pfxbitarr: BitMap4stride,
}

pub struct TreeBitMap<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    pfx_vec: Vec<&'a Prefix<AF, T>>,
    ptr_vec: Vec<Box<TreeBitMapNode>>,
}

impl<'a, T> TreeBitMap<'a, u32, T>
where
    T: Debug,
{
    const BITS: u8 = 32;
    pub fn new() -> TreeBitMap<'a, u32, T> {
        let ptr_vec = vec![Box::new(TreeBitMapNode {
            ptrbitarr: 0,
            pfxbitarr: 0,
        })];

        TreeBitMap {
            ptr_vec,
            pfx_vec: vec![],
        }
    }

    // 0 1 2 3  4  5  6   7   8   9  10  11  12  13  14   15   16   17   18   19   20   21   22   23   24    25  26    27  28   29   30
    // * 0 1 00 01 10 11 000 001 010 011 100 101 110 111 0000 0001 0010 0011 0100 0101 0110 0111 1000 1001 1010 1011 1100 1101 1110 1111
    // stride 4: 1 + 2 + 4 + 8 + 16 = 30 bits. 2^5 - 1
    // stride 5: 1 + 2 + 4 + 8 + 16 + 32 + 64 = 63 bits. 2^5 - 1
    // stride 6: 1 + 2 + 4 + 8 + 16 + 32 + 64 = 127 bits.  2^7 - 1
    // stride 8: 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256 = 511 bits. 2^9 - 1
    // 5 - 5 - 5 - 4 - 4 - [4] - 5
    // startpos (2 ^ stride_depth) - 1 + nibble as usize
    pub fn insert(&mut self, pfx: &'a Prefix<u32, T>) {
        let mut found_child_node = false;
        let mut stride_end = 4;
        let mut current_node = self.ptr_vec.first_mut().unwrap();
        for stride in vec![4; 8] {
            // let stride = 4;
            let mut last_stride = false;
            let bit_depth;
            if stride_end <= pfx.len {
                bit_depth = stride;
                if stride_end == pfx.len {
                    last_stride = true;
                }
            } else {
                bit_depth = stride_end - pfx.len;
                last_stride = true;
            };

            let built_prefix =
                (pfx.net << (stride_end - bit_depth)) >> ((Self::BITS - bit_depth) as usize);

            // Move the bit in the right position.
            // println!("{}", bit_depth);
            let offset: u8 = (2 << (bit_depth - 1)) - 1;

            println!("---");
            println!(
                "stride: {} offset: {:?}, pos: {:?}",
                stride_end, offset, built_prefix
            );

            let bp2: u32 = 0x1 << (Self::BITS - offset as u8 - built_prefix as u8 - 1);

            if !last_stride {
                // Check it the ptr bit is already set
                if current_node.ptrbitarr & bp2 > 0 {
                    println!("child node found");
                    found_child_node = true;
                } else {
                    // Nope, set it.
                    current_node.ptrbitarr = bp2 | current_node.ptrbitarr;
                }
            } else {
                println!("reached {:?}", pfx);
                current_node.pfxbitarr = bp2 | current_node.pfxbitarr;
                // current_node.prefix = Some(&pfx);
                self.pfx_vec.push(pfx);

                println!("bits  : {:32b}", &built_prefix);
                println!("_bp2__: {:032b}", &bp2);
                println!("ptrarr: {:032b}", &current_node.ptrbitarr);
                println!("pfxarr: {:032b}", &current_node.pfxbitarr);
                println!("end");
                return;
            }

            let next_index = current_node.ptrbitarr.count_ones() as usize;

            println!("bits  : {:32b}", &built_prefix);
            println!("_bp2__: {:032b}", &bp2);
            println!("ptrarr: {:032b}", &current_node.ptrbitarr);
            println!("pfxarr: {:032b}", &current_node.pfxbitarr);

            if !found_child_node {
                println!("create child node");
                let new_node = Box::new(TreeBitMapNode {
                    ptrbitarr: 0,
                    pfxbitarr: 0,
                });
                self.ptr_vec.push(new_node);
                // self.ptr_vec.sort();
            }

            current_node = &mut self.ptr_vec[next_index];
            stride_end = stride_end + stride;
        }
    }

    pub fn get_pfx(net: u32) -> Prefix<u32, T> {
        Prefix::<u32, T>::new(net, 0)
    }
}

pub mod allocator;
