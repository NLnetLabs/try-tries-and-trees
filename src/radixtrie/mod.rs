use crate::common::{AddressFamily, NoMeta, Prefix};
use num::PrimInt;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub struct RadixTrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    pub prefix: Option<&'a Prefix<AF, T>>,
    pub bit_pos: u8,
    pub left: Option<Box<RadixTrieNode<'a, AF, T>>>,
    pub right: Option<Box<RadixTrieNode<'a, AF, T>>>,
}

impl<'a, AF, T> RadixTrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    pub fn new(pfx: Option<&'a Prefix<AF, T>>) -> RadixTrieNode<'a, AF, T> {
        RadixTrieNode::<'a, AF, T> {
            prefix: pfx,
            bit_pos: 0,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct RadixTrie<'a, AF, T>(RadixTrieNode<'a, AF, T>)
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug;

impl<'a, AF, T> RadixTrie<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug + fmt::Binary,
{
    pub fn new() -> RadixTrie<'a, AF, T> {
        RadixTrie(RadixTrieNode {
            prefix: None,
            bit_pos: 0,
            left: None,
            right: None,
        })
    }

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>) {
        let mut cursor = &mut self.0;

        let mut cur_bit = cursor.bit_pos;
        let mut first_bit = pfx.net << cursor.bit_pos as usize;
        let mut built_prefix: AF = num::zero();
        let zero = num::zero();

        loop {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            println!("cur {}", cur_bit);

            if cur_bit == pfx.len {
                print!("==");
                cursor.prefix = Some(&pfx);
                cursor.bit_pos = pfx.len;
                break;
            }

            let mut next_cursor_bit_pos = 0;

            if cur_bit < pfx.len {
                match first_bit & AF::BITMASK {
                    b if b == zero => {
                        if !cursor.left.is_some() {
                            // new node on the left
                            // cursor.next_bit = pfx.len - cur_bit;
                            print!("+l");
                            let mut nn = RadixTrieNode::new(None);
                            nn.bit_pos = pfx.len - cur_bit;
                            // nn.prefix = Some(&pfx);
                            cursor.left = Some(Box::new(nn));
                        };
                        built_prefix = built_prefix << cursor.bit_pos as usize;
                        // next_cursor = cursor.left.as_deref_mut().unwrap();
                        next_cursor_bit_pos = cursor.left.as_deref().unwrap().bit_pos;
                    }
                    _ => {
                        if !cursor.right.is_some() {
                            // new node on the right
                            // cursor.next_bit = pfx.len - cur_bit;
                            print!("+r");
                            let mut nn = RadixTrieNode::new(None);
                            nn.bit_pos = pfx.len - cur_bit;
                            // nn.prefix = Some(&pfx);
                            cursor.right = Some(Box::new(nn));
                        }
                        built_prefix = built_prefix << cursor.bit_pos as usize | num::one();
                        // next_cursor = cursor.right.as_deref_mut().unwrap();
                        next_cursor_bit_pos = cursor.right.as_deref().unwrap().bit_pos;
                    }
                }
            }

            let next_first_bit = first_bit << next_cursor_bit_pos as usize;
            let next_cur_bit = cur_bit + next_cursor_bit_pos;

            if next_cur_bit > pfx.len {
                // TODO: Backtrack here to parent and
                // insert new node as its child with bit_pos: cur_bit - pfx.len

                print!(">bt{}-{}<", cur_bit - pfx.len, cursor.bit_pos);

                // cur_bit += cursor.bit_pos;
                let mut nn = RadixTrieNode::new(Some(&pfx));
                nn.bit_pos = cursor.bit_pos - (cur_bit - pfx.len);
                match first_bit & AF::BITMASK {
                    b if b == zero => {
                        if !cursor.left.is_some() {
                            print!("+ll");
                            cursor.left.replace(Box::new(nn));
                        } else {
                            print!("|l");
                            nn.left = std::mem::take(&mut cursor.left);
                            // nn.right = std::mem::take(&mut cursor.right);
                            cursor.left = Some(Box::new(nn));
                        }
                    }
                    _ => {
                        print!("+r");
                        if !cursor.right.is_some() {
                            print!("+lr");
                            cursor.right = Some(Box::new(nn));
                        } else {
                            print!("|r");
                            nn.right = std::mem::take(&mut cursor.right);
                            // nn.left = std::mem::take(&mut cursor.left);
                            cursor.right = Some(Box::new(nn));
                        }
                    }
                }
                break;
            }

            match first_bit & AF::BITMASK {
                b if b == zero => {
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                _ => {
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            };

            first_bit = next_first_bit;
            cur_bit = next_cur_bit;
        }
        // println!("bp: {:b}", built_prefix);

        let len = pfx.len;
        let shift: usize = (AF::BITS - pfx.len) as usize;
        let net = built_prefix << if shift < AF::BITS as usize { shift } else { 0 };

        // println!("{:b}", net);
        // cursor.prefix = Some(&pfx);
        println!(
            "inserted prefix: {:?} -> {:?}/{}",
            pfx,
            AF::fmt_net(net),
            len
        );
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Option<&'a Prefix<AF, T>> {
        let mut cursor = &self.0;
        let mut match_pfx: Option<&'a Prefix<AF, T>> = None;
        let zero: AF = num::zero();

        print!("cb {}", cursor.bit_pos);

        let mut i = cursor.bit_pos;
        let mut first_bit = search_pfx.net << cursor.bit_pos as usize;
        // for i in 0..(search_pfx.len + 1) {
        while i <= search_pfx.len {
            // if let Some(found_pfx) = cursor.prefix {
            //     match_pfx = Some(found_pfx);
            //     // build_pfx = cursor_pfx;
            //     // match_len = i;
            //     // let shift = if i > 0 { (AF::BITS - i) as usize } else { 0 };
            //     println!(
            //         "less-specific: {}/{} with {:?}",
            //         // AF::fmt_net(cursor_pfx << shift).as_str(),
            //         // match_len,
            //         found_pfx.meta
            //     );
            // }

            match first_bit & AF::BITMASK {
                b if b == zero => {
                    if cursor.left.is_some() {
                        cursor = cursor.left.as_deref().unwrap();
                        // cursor_pfx = cursor_pfx << num::one();
                        if cursor.prefix.is_some() {
                            println!("less specific : {:?}", cursor.prefix);
                            match_pfx = cursor.prefix;
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    if cursor.right.is_some() {
                        cursor = cursor.right.as_deref().unwrap();
                        // cursor_pfx = cursor_pfx << num::one() | num::one();
                        if cursor.prefix.is_some() {
                            println!("less specific : {:?}", cursor.prefix);
                            match_pfx = cursor.prefix;
                        }
                    } else {
                        break;
                    }
                }
            }
            first_bit = first_bit << cursor.bit_pos as usize;
            i += cursor.bit_pos;
        }

        // if match_len > 0 {
        //     let build_pfx_net = AF::fmt_net(build_pfx << (AF::BITS - match_len) as usize);
        //     println!("built prefix: {}/{}", build_pfx_net.as_str(), match_len);
        // }

        match_pfx
    }
}
