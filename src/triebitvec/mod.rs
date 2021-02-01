use crate::nodes::{Prefix, BinaryNode2, NoMeta, PrefixAs};
use bitvec::prelude::*;

type EmptyPrefix<'a> = Prefix<'a, NoMeta>;
type PrefixWithAs<'a> = Prefix<'a, PrefixAs>;

type BinaryNode<'a> = BinaryNode2<'a, NoMeta>;

pub fn longest_matching_prefix<'a>(search_pfx: &EmptyPrefix, trie: &'a BinaryNode) -> EmptyPrefix<'a> {
    let mut cursor: &'a BinaryNode = trie;
    let mut match_pfx = bitvec![Msb0, u32;0; 32];
    let mut match_len = 0;
    let mut bits = bitvec![Msb0, u32;0; search_pfx.len as usize];
    bits.store(search_pfx.net);
    for (i, b) in bits.iter().enumerate() {
        match *b {
            false => {
                if cursor.left.is_some() {
                    cursor = cursor.left.as_deref().unwrap();
                    match_pfx.set(i, false);
                    match_len += 1;
                } else {
                    break;
                }
            }
            true => {
                if cursor.right.is_some() {
                    cursor = cursor.right.as_deref().unwrap();
                    match_pfx.set(i, true);
                    match_len += 1;
                } else {
                    break;
                }
            }
        }
    }
    EmptyPrefix::new(match_pfx.load(), match_len)
}

pub fn create_trie() {
    let pfxs = [
        PrefixWithAs::new_with_meta(0b1111_0000_0000_0000_0000_0000_1111_1111_u32, 32, &PrefixAs(1)),
        PrefixWithAs::new_with_meta(0b0100_0000_0110_0000_0111_0000_1101_0011_u32, 4, &PrefixAs(2)),
        PrefixWithAs::new_with_meta(0b1100_0000_0000_0011_0000_0000_0010_0000_u32, 8, &PrefixAs(2)),
    ];

    let mut trie = BinaryNode::new(None);
    let mut cursor: &mut BinaryNode;

    for pfx in pfxs.iter() {
        // let mut net: u32 = pfx.net;
        let len = pfx.len as usize;
        // let bits = net.view_bits_mut::<Msb0>();
        let mut bits = bitvec![Msb0, u32; 0; 32];
        bits.store(pfx.net);
        bits.resize(pfx.len as usize, false);
        println!("{}", bits);

        cursor = &mut trie;

        // for b in bits[0..len].iter() {
        for b in bits.iter() {
            match *b {
                false => {
                    if !cursor.left.is_some() {
                        cursor.left = Some(Box::new(BinaryNode::new(None)));
                    };
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                true => {
                    if !cursor.right.is_some() {
                        cursor.right = Some(Box::new(BinaryNode::new(None)));
                    }
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            }
        }
        // let len = pfx.len as usize;
        // if len < 32 {
        //     bits[len..32].store(0_u32);
        // };
        bits.resize(32, false);
        println!("{}", bits);
        let net: u32 = bits.load();

        cursor.prefix = Some(EmptyPrefix::new(net, len as u8));
        println!("prefix: {:?}/{}", std::net::Ipv4Addr::from(net), pfx.len);
    }
    let spfx = EmptyPrefix::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32);
    let spfx2 = EmptyPrefix::new(std::net::Ipv4Addr::new(64, 10, 10, 10).into(), 32);
    println!("trie: {:?}", trie);

    let s_spfx = longest_matching_prefix(&spfx, &trie);
    println!("search: {:?}", s_spfx);

    let s_spfx = longest_matching_prefix(&spfx2, &trie);
    println!("search: {:?}", s_spfx);
}
