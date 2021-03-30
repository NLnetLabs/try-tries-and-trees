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
    pub bit_id: AF,
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
            bit_id: AF::zero(),
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
            bit_id: AF::zero(),
            left: None,
            right: None,
        })
    }

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>) {
        let mut cursor = &mut self.0;

        let zero = num::zero();

        loop {
            // we might already be at the place we need to be,
            // at either a leaf or an internal node, just
            // set our insert prefix here and be done with it.
            if pfx.len == cursor.bit_pos {
                cursor.prefix = Some(&pfx);
                break;
            }

            let mut next_cursor = match (pfx.net << cursor.bit_pos as usize) & AF::BITMASK {
                b if b == zero => &mut cursor.left,
                _ => &mut cursor.right,
            };

            match &next_cursor {
                // No node in the direction we're heading, so we can create a new leaf node
                // with our prefix length and the prefix.
                None => {
                    let mut new_leaf = RadixTrieNode::new(Some(&pfx));
                    new_leaf.bit_pos = pfx.len;
                    new_leaf.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                    *next_cursor = Some(Box::new(new_leaf));
                    break;
                }
                // There is a node on the left side, we need to see if that node's
                // bit_id matches our bit_id.
                Some(next_node) => {
                    // The prefix is aligned with the next_node, move on
                    if pfx.net >> (AF::BITS - next_node.bit_pos) as usize == next_node.bit_id {
                    } else {
                        // figure out where a possible new intermediary node should be placed
                        // , by comparing the new intermediary node's prefix with the next
                        // node's bit_id.
                        // ex.:
                        // new intem. node pfx  : 1010 0000 0000 0000 0000 / 24 (len)
                        // next node bit_id     : 0000 0000 0000 0000 0100 / 3 (bit_pos)
                        // so, we left shift the next node's bit_id to match the length of the
                        // next_node's bit_pos and then XOR:
                        // 1010 0000 0000 0000 0000 ^
                        // 1000 0000 0000 0000 0000
                        // ------------------------ =
                        // 0010 0000 0000 0000 0000
                        //
                        // then the number of leading zeros (2) marks the number of bits that
                        // are the same in both bitmaps for the given bit_pos size.
                        // This will be the bit_pos size of the intermediary node (nn)
                        let in_bit_pos = (pfx.net
                            ^ (next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize))
                            .leading_zeros() as u8;
                        let in_bit_id = pfx.net >> (AF::BITS - in_bit_pos) as usize;

                        // Only create a new intermediary node if it does not overshoot
                        // the existing downstream node (its intended child).
                        let mut intermediary_node = RadixTrieNode::new(None);
                        intermediary_node.bit_pos = in_bit_pos;
                        intermediary_node.bit_id = in_bit_id;

                        // If we've reached the pfx.len in the intermediary_node,
                        // that means that it will have to host a prefix, (so
                        // it'll become a regular internal node).
                        // Also we should break out of the loop after inserting
                        // it into the trie.
                        let mut done = false;
                        if intermediary_node.bit_pos == pfx.len {
                            intermediary_node.prefix = Some(&pfx);
                            done = true;
                        }

                        // Now, check if we should attach the next_node to the
                        // left or the right of our new intermediary node.
                        // We'll check the bit at (intermediary_node.bit_pos + 1),
                        // since that's the bit where they start to diverge.
                        let l_r_bit_next_node = (next_node.bit_id
                            << ((AF::BITS - next_node.bit_pos) + intermediary_node.bit_pos)
                                as usize)
                            .leading_zeros()
                            == 0;

                        let mut insert_node = RadixTrieNode::new(Some(&pfx));

                        let (next_node, next_node_opp) = match l_r_bit_next_node {
                            false => (&mut intermediary_node.left, &mut intermediary_node.right),
                            true => (&mut intermediary_node.right, &mut intermediary_node.left),
                        };

                        // We are overshooting our intended child, we have to go back
                        // to inserting this final prefix node on cursor, not next_node.
                        if in_bit_pos >= pfx.len {
                            match l_r_bit_next_node {
                                true => {
                                    insert_node.right = std::mem::take(&mut next_cursor);
                                    insert_node.bit_pos = pfx.len;
                                    insert_node.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                    *next_cursor = Some(Box::new(insert_node));
                                    break;
                                }
                                false => {
                                    insert_node.left = std::mem::take(&mut next_cursor);
                                    insert_node.bit_pos = pfx.len;
                                    insert_node.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                    *next_cursor = Some(Box::new(insert_node));
                                    break;
                                }
                            }
                        } else {
                            // Insert the intermediary node in between the cursor
                            // and its [left|right] child.

                            // cursor is cut off at this point after this assignment!
                            *next_node = std::mem::take(&mut next_cursor);
                            if !done {
                                // let mut nn = RadixTrieNode::new(Some(&pfx));
                                insert_node.bit_pos = pfx.len;
                                insert_node.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                *next_node_opp = Some(Box::new(insert_node));
                                done = true;
                            }
                            // Weld the cursor to the newly created intermediary node.
                            *next_cursor = Some(Box::new(intermediary_node));

                            if done {
                                break;
                            }
                        }
                    };
                }
            };
            cursor = next_cursor.as_deref_mut().unwrap();
        }
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Option<&'a Prefix<AF, T>> {
        let mut cursor = Some(&self.0);
        let mut match_pfx: Option<&'a Prefix<AF, T>> = None;
        let zero: AF = num::zero();

        let mut next_pos = search_pfx.net << cursor.unwrap().bit_pos as usize;
        loop {
            let bit_id_match = (search_pfx.net
                ^ (cursor.unwrap().bit_id << (AF::BITS - cursor.unwrap().bit_pos) as usize))
                .leading_zeros()
                >= cursor.unwrap().bit_pos as u32;

            match next_pos & AF::BITMASK {
                b if b == zero && bit_id_match => {
                    cursor = cursor
                        .and_then(|c| {
                            match_pfx = c.prefix;
                            c.left.as_deref()
                        })
                        .and_then(|c| {
                            Some(c)
                        });
                    println!("{:?}", match_pfx);
                }
                _ if bit_id_match => {
                    cursor = cursor
                        .and_then(|c| {
                            match_pfx = c.prefix;
                            c.right.as_deref()
                        })
                        .and_then(|c| {
                            Some(c)
                        });
                    println!("{:?}", match_pfx);
                }
                _ => {
                    break;
                }
            }
            if let Some(c) = cursor {
                next_pos = search_pfx.net << c.bit_pos as usize;
            } else {
                break;
            }
        }

        match_pfx
    }
}
