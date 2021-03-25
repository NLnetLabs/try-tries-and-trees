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

        let built_prefix: AF = num::zero();
        let zero = num::zero();

        loop {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());

            print!("*bp{}", cursor.bit_pos);
            println!("cursor#1 {:#?}", cursor);

            // we might already be at the place we need to be,
            // at either a leaf or an internal node, just
            // set our insert prefix here and be done with it.
            if pfx.len == cursor.bit_pos {
                println!("cursor {:?}/{:?}", cursor.bit_id, cursor.bit_pos);
                cursor.prefix = Some(&pfx);
                break;
            }

            // if cur_bit <= pfx.len {
            println!("prefix {:?}", pfx);
            println!(
                "test bitpos {} {:032b}",
                cursor.bit_pos,
                pfx.net << (cursor.bit_pos as usize - 1)
            );

            match (pfx.net << cursor.bit_pos as usize) & AF::BITMASK {
                // Inspected bit is ZERO, this node should go to the LEFT
                b if b == zero => {
                    match &cursor.left {
                        // No node on the left, so we can create a new leaf node
                        // with our prefix length and the prefix.
                        None => {
                            print!("+l{}", pfx.len);
                            let mut new_leaf = RadixTrieNode::new(Some(&pfx));
                            new_leaf.bit_pos = pfx.len;
                            new_leaf.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                            cursor.left = Some(Box::new(new_leaf));
                            println!("bit_id {:032b}", pfx.net >> (AF::BITS - pfx.len) as usize);
                            break;
                        }
                        // There is a node on the left side, we need to see if that node's
                        // bit_id matches our bit_id.
                        Some(next_node) => {
                            println!("=l");
                            println!("{:#?}", cursor);
                            // The prefix is aligned with the next_node, move on
                            if pfx.net >> (AF::BITS - next_node.bit_pos) as usize
                                == next_node.bit_id
                            {
                                println!("<-<-<-turn Left");
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
                                    .leading_zeros()
                                    as u8;
                                let in_bit_id = pfx.net >> (AF::BITS - in_bit_pos) as usize;

                                // Only create a new intermediary node if it does not overshoot
                                // the existing downstream node (its intended child).

                                let mut intermediary_node = RadixTrieNode::new(None);
                                intermediary_node.bit_pos = in_bit_pos;
                                intermediary_node.bit_id = in_bit_id;

                                println!(
                                    "pfx.len {:2} pfx.net          {:032b}",
                                    &pfx.len, &pfx.net
                                );
                                println!(
                                    "next_node bit_pos {:2} bit_id {:032b}",
                                    &next_node.bit_pos, &next_node.bit_id
                                );
                                println!(
                                    "shift next_node             {:032b}",
                                    next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize
                                );
                                println!(
                                    "XOR                         {:032b}",
                                    pfx.net
                                        ^ (next_node.bit_id
                                            << (AF::BITS - next_node.bit_pos) as usize)
                                );
                                println!(
                                    "interm bit_pos {:2} bit_id  {:032b}",
                                    intermediary_node.bit_pos, intermediary_node.bit_id
                                );
                                println!(
                                    "leading zeros {}",
                                    (pfx.net
                                        ^ (next_node.bit_id
                                            << (AF::BITS - next_node.bit_pos) as usize))
                                        .leading_zeros()
                                );

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
                                // let ii = pfx.net
                                //     ^ (next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize);
                                let ident = next_node.bit_id
                                    << ((AF::BITS - next_node.bit_pos) + intermediary_node.bit_pos)
                                        as usize;
                                let l_r_bit_next_node = ident.leading_zeros() == 0;
                                println!("next_node bit_id {:032b}", next_node.bit_id);
                                println!(
                                    "shift            {:032b}",
                                    next_node.bit_id
                                        << ((AF::BITS - next_node.bit_pos)
                                            + intermediary_node.bit_pos)
                                            as usize
                                );
                                println!("dir -><- {:?}", l_r_bit_next_node);
                                print!("ident {:?}", ident);

                                if in_bit_pos < pfx.len {
                                    // if ii != AF::zero() {
                                    match l_r_bit_next_node {
                                        // Insert the intermediary node in between the cursor
                                        // and its left-hand child.
                                        // Our prefix to be inserted goes on the right, so move
                                        // the existing one to the left.
                                        true => {
                                            // cursor is cut off at this point after this assignment!
                                            intermediary_node.right =
                                                std::mem::take(&mut cursor.left);
                                            if !done {
                                                let mut nn = RadixTrieNode::new(Some(&pfx));
                                                nn.bit_pos = pfx.len;
                                                nn.bit_id =
                                                    pfx.net >> (AF::BITS - pfx.len) as usize;
                                                intermediary_node.left = Some(Box::new(nn));
                                                done = true;
                                            }
                                            // weld the cursor to the newly created intermediary node.
                                            cursor.left = Some(Box::new(intermediary_node));
                                            // cursor = cursor.left.as_deref_mut().unwrap();

                                            // break;
                                        }
                                        // Insert the intermediary node in between the cursor and its
                                        // right hand child.
                                        false => {
                                            intermediary_node.left =
                                                std::mem::take(&mut cursor.left);
                                            if !done {
                                                let mut nn = RadixTrieNode::new(Some(&pfx));
                                                nn.bit_pos = pfx.len;
                                                nn.bit_id =
                                                    pfx.net >> (AF::BITS - pfx.len) as usize;
                                                intermediary_node.right = Some(Box::new(nn));
                                                done = true;
                                            }
                                            // weld the cursor to the newly created intermediary node.
                                            cursor.left = Some(Box::new(intermediary_node));
                                            // cursor = cursor.left.as_deref_mut().unwrap();

                                            // break;
                                        }
                                    }
                                    // }

                                    if done {
                                        print!("D");
                                        break;
                                    }

                                // println!("#2 cursor {:#?}", cursor);
                                // }
                                }
                                // we are overshooting our intended child, we have to go back to inserting
                                // this intermediary node on cursor, not next_node.
                                else {
                                    print!("else");
                                    // cursor.prefix = Some(&pfx);
                                    // cursor.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                    // cursor.bit_pos = pfx.len;
                                    match l_r_bit_next_node {
                                        true => {
                                            // cursor.left = Some(Box::new(intermediary_node));
                                            let mut nn = RadixTrieNode::new(Some(&pfx));
                                            nn.left = std::mem::take(&mut cursor.left);
                                            nn.bit_pos = pfx.len;
                                            nn.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                            cursor.left = Some(Box::new(nn));
                                            break;
                                            // cursor = cursor.left.as_deref_mut().unwrap();
                                        }
                                        false => {
                                            // cursor.right = Some(Box::new(intermediary_node));
                                            let mut nn = RadixTrieNode::new(Some(&pfx));
                                            nn.left = std::mem::take(&mut cursor.right);
                                            nn.bit_pos = pfx.len;
                                            nn.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                            cursor.left = Some(Box::new(nn));
                                            break;
                                            // cursor = cursor.right.as_deref_mut().unwrap();
                                        }
                                    }
                                }
                            };
                        }
                    };
                    cursor = cursor.left.as_deref_mut().unwrap();
                    // println!("#3 cursor {:#?}", cursor);
                    // break;
                }
                _ => {
                    match &cursor.right {
                        None => {
                            print!("+r{}", pfx.len);
                            let mut nn = RadixTrieNode::new(Some(&pfx));
                            nn.bit_pos = pfx.len;
                            nn.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                            cursor.right = Some(Box::new(nn));
                            break;
                        }
                        Some(next_node) => {
                            println!("=r");

                            if pfx.net >> (AF::BITS - next_node.bit_pos) as usize
                                == next_node.bit_id
                            {
                                println!("->->->Right");
                                println!("{:#?}", cursor);
                            } else {
                                let in_bit_pos = (pfx.net
                                    ^ (next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize))
                                    .leading_zeros()
                                    as u8;
                                let in_bit_id = pfx.net >> (AF::BITS - in_bit_pos) as usize;

                                // Only create a new intermediary node if it does not overshoot,
                                // the existing downstream node (its intended child).

                                let mut intermediary_node = RadixTrieNode::new(None);
                                intermediary_node.bit_pos = in_bit_pos;
                                intermediary_node.bit_id = in_bit_id;

                                println!(
                                    "pfx.len {:2} pfx.net          {:032b}",
                                    &pfx.len, &pfx.net
                                );
                                println!(
                                    "next_node bit_pos {:2} bit_id {:032b}",
                                    &next_node.bit_pos, &next_node.bit_id
                                );
                                println!(
                                    "shift next_node             {:032b}",
                                    next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize
                                );
                                println!(
                                    "XOR                         {:032b}",
                                    pfx.net
                                        ^ (next_node.bit_id
                                            << (AF::BITS - next_node.bit_pos) as usize)
                                );
                                println!(
                                    "nn bit_pos {:2} bit_id        {:032b}",
                                    intermediary_node.bit_pos, intermediary_node.bit_id
                                );
                                println!(
                                    "leading zeros {}",
                                    (pfx.net
                                        ^ (next_node.bit_id
                                            << (AF::BITS - next_node.bit_pos) as usize))
                                        .leading_zeros()
                                );
                                // let ii = pfx.net
                                //     ^ (next_node.bit_id << (AF::BITS - next_node.bit_pos) as usize);
                                let ident = next_node.bit_id
                                    << ((AF::BITS - next_node.bit_pos) + intermediary_node.bit_pos)
                                        as usize;
                                let l_r_bit_next_node = ident.leading_zeros() == 0;
                                println!("next_node bit_id {:032b}", next_node.bit_id);
                                println!(
                                    "shift            {:032b}",
                                    next_node.bit_id
                                        << ((AF::BITS - next_node.bit_pos)
                                            + intermediary_node.bit_pos)
                                            as usize
                                );
                                println!(
                                    "shift pfx        {:032b}",
                                    pfx.net << (intermediary_node.bit_pos + 1) as usize
                                );
                                println!("dir -><- {:?}", l_r_bit_next_node);
                                print!("ident {:?}", ident);

                                // If we've reached the pfx.len in the intermediary_node,
                                // that means that it will have to host a prefix, (so
                                // it'll become a regular internal node).
                                // Also we should break out of the loop after inserting
                                // it into the trie.
                                let mut done = false;
                                if intermediary_node.bit_pos == pfx.len {
                                    intermediary_node.prefix = Some(&pfx);
                                    done = true;
                                    // l_r_bit_next_node = true;
                                }

                                if in_bit_pos < pfx.len {
                                    // if ii != AF::zero() {
                                    match l_r_bit_next_node {
                                        // Our prefix should go the right, so move the
                                        // existing one to the left.
                                        false => {
                                            print!("->");
                                            intermediary_node.left =
                                                std::mem::take(&mut cursor.right);
                                            if !done {
                                                let mut nn = RadixTrieNode::new(Some(&pfx));
                                                nn.bit_pos = pfx.len;
                                                nn.bit_id =
                                                    pfx.net >> (AF::BITS - pfx.len) as usize;
                                                intermediary_node.right = Some(Box::new(nn));
                                                done = true;
                                            }
                                            cursor.right = Some(Box::new(intermediary_node));
                                            // cursor = cursor.right.as_deref_mut().unwrap();

                                            // break;
                                        }
                                        true => {
                                            print!("<-");
                                            intermediary_node.right =
                                                std::mem::take(&mut cursor.right);
                                            if !done {
                                                let mut nn = RadixTrieNode::new(Some(&pfx));

                                                nn.bit_pos = pfx.len;
                                                nn.bit_id =
                                                    pfx.net >> (AF::BITS - pfx.len) as usize;
                                                intermediary_node.left = Some(Box::new(nn));
                                                done = true;
                                            }
                                            cursor.right = Some(Box::new(intermediary_node));
                                            // cursor = cursor.right.as_deref_mut().unwrap();

                                            // break;
                                        }
                                    }
                                } else {
                                    print!("else");
                                    // cursor.prefix = Some(&pfx);
                                    // cursor.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                    // cursor.bit_pos = pfx.len;
                                    match l_r_bit_next_node {
                                        true => {
                                            print!(";;<-;;");
                                            // cursor.left = Some(Box::new(intermediary_node));
                                            let mut nn = RadixTrieNode::new(Some(&pfx));
                                            nn.bit_pos = pfx.len;
                                            nn.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                            nn.left = std::mem::take(&mut cursor.right);
                                            cursor.right = Some(Box::new(nn));
                                            break;
                                            // cursor = cursor.left.as_deref_mut().unwrap();
                                        }
                                        false => {
                                            print!(";;->;;");
                                            // cursor.right = Some(Box::new(intermediary_node));
                                            let mut nn = RadixTrieNode::new(Some(&pfx));
                                            nn.bit_pos = pfx.len;
                                            nn.bit_id = pfx.net >> (AF::BITS - pfx.len) as usize;
                                            nn.right = std::mem::take(&mut cursor.right);
                                            cursor.right = Some(Box::new(nn));
                                            break;
                                            // cursor = cursor.right.as_deref_mut().unwrap();
                                        }
                                    }
                                }
                                if done {
                                    break;
                                }
                            };
                        }
                    };
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            }
            // } else {
            //     print!("d{}", pfx.len);
            //     break;
            // }
        }
        // println!("bp: {:b}", built_prefix);

        let len = pfx.len;
        let shift: usize = (AF::BITS - pfx.len) as usize;
        let net = built_prefix << if shift < AF::BITS as usize { shift } else { 0 };

        // println!("{:b}", net);
        // cursor.prefix = Some(&pfx);
        println!(
            "S inserted prefix: {:?} -> {:?}/{}",
            pfx,
            AF::fmt_net(net),
            len
        );

        println!("cursor {:#?}", cursor);
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Option<&'a Prefix<AF, T>> {
        let mut cursor = &self.0;
        let mut match_pfx: Option<&'a Prefix<AF, T>> = None;
        let zero: AF = num::zero();

        // let mut i = cursor.bit_pos;
        let mut first_bit = search_pfx.net << 0;
        // for i in 0..(search_pfx.len + 1) {
        while cursor.bit_pos <= search_pfx.len {
            print!("*bp{}", cursor.bit_pos);
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
                    print!("l");
                    if cursor.left.is_some() {
                        cursor = cursor.left.as_deref().unwrap();
                        // cursor_pfx = cursor_pfx << num::one();
                        if cursor.prefix.is_some() {
                            println!("L {} less specific : {:?}", cursor.bit_pos, cursor.prefix);
                            match_pfx = cursor.prefix;
                        }
                    } else {
                        print!("- ");
                        break;
                    }
                }
                _ => {
                    print!("r");
                    if cursor.right.is_some() {
                        cursor = cursor.right.as_deref().unwrap();
                        // cursor_pfx = cursor_pfx << num::one() | num::one();
                        if cursor.prefix.is_some() {
                            println!("R {} less specific : {:?}", cursor.bit_pos, cursor.prefix);
                            match_pfx = cursor.prefix;
                        }
                    } else {
                        print!("- ");
                        break;
                    }
                }
            }
            first_bit = search_pfx.net << (cursor.bit_pos as usize - 1);
        }

        // if match_len > 0 {
        //     let build_pfx_net = AF::fmt_net(build_pfx << (AF::BITS - match_len) as usize);
        //     println!("built prefix: {}/{}", build_pfx_net.as_str(), match_len);
        // }

        match_pfx
    }
}
