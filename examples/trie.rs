use trie::simpletrie::{NoMeta, Prefix, PrefixAs, TrieNode};
type EmptyPrefix<'a> = Prefix<NoMeta>;
type PrefixWithAs<'a> = Prefix<PrefixAs>;

pub fn longest_matching_prefix<'a>(
    search_pfx: &EmptyPrefix,
    trie: &'a TrieNode,
) -> EmptyPrefix<'a> {
    let mut cursor: &'a TrieNode = trie;
    let mut cursor_pfx: u32 = 0;
    let mut match_pfx: u32 = 0;
    let mut match_len = 0;
    let mut first_bit = search_pfx.net;

    for i in 1..search_pfx.len {
        match first_bit.leading_ones() {
            0 => {
                if cursor.left.is_some() {
                    cursor = cursor.left.as_deref().unwrap();
                    cursor_pfx = cursor_pfx << 1;
                    if cursor.prefix == true {
                        match_pfx = cursor_pfx;
                        match_len = i;
                        println!(
                            "lmp less-spefific: {:?}/{}",
                            std::net::Ipv4Addr::from(match_pfx << (32 - match_len)),
                            match_len
                        );
                    }
                } else {
                    break;
                }
            }
            b if b >= 1 => {
                if cursor.right.is_some() {
                    cursor = cursor.right.as_deref().unwrap();
                    cursor_pfx = cursor_pfx << 1 | 1;
                    if cursor.prefix == true {
                        match_pfx = cursor_pfx;
                        match_len = i;
                        println!(
                            "lmp less-spefific: {:?}/{}",
                            std::net::Ipv4Addr::from(match_pfx << (32 - match_len)),
                            match_len
                        );
                    }
                } else {
                    break;
                }
            }
            _ => {
                panic!("illegal prefix encountered. Giving up.");
            }
        }
        first_bit = first_bit << 1;
    }
    Prefix::new(match_pfx << (32 - match_len), match_len)
}

fn main() {
    let mut trie = TrieNode::new(false);
    let mut cursor: &mut TrieNode;

    let pfxs = [
        PrefixWithAs::new_with_meta(
            0b1111_0000_0000_0000_0000_0000_1111_1111_u32,
            32,
            PrefixAs(1),
        ),
        PrefixWithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            4,
            PrefixAs(1),
        ),
        PrefixWithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            8,
            PrefixAs(2),
        ),
        PrefixWithAs::new_with_meta(
            0b1100_0000_0000_0011_0000_0000_0010_0000_u32,
            3,
            PrefixAs(2),
        ),
        PrefixWithAs::new_with_meta(
            std::net::Ipv4Addr::new(12, 0, 0, 34).into(),
            8,
            PrefixAs(100),
        ),
        PrefixWithAs::new_with_meta(
            std::net::Ipv4Addr::new(12, 0, 0, 34).into(),
            32,
            PrefixAs(100),
        ),
    ];

    for pfx in pfxs.iter() {
        cursor = &mut trie;

        let mut first_bit = pfx.net;
        let mut built_prefix: u32 = 0;

        for _ in 0..pfx.len {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            match first_bit.leading_ones() {
                0 => {
                    if !cursor.left.is_some() {
                        cursor.left = Some(Box::new(TrieNode::new(false)))
                    };
                    built_prefix = built_prefix << 1;
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                1..=32 => {
                    if !cursor.right.is_some() {
                        cursor.right = Some(Box::new(TrieNode::new(false)));
                    }
                    cursor = cursor.right.as_deref_mut().unwrap();
                    built_prefix = built_prefix << 1 | 1;
                }
                _ => {
                    panic!("illegal prefix encountered. Giving up.");
                }
            }
            first_bit = first_bit << 1;
        }
        println!("bp: {:b}", built_prefix);

        let len = pfx.len;
        let net = built_prefix << (32 - pfx.len);

        println!("{:b}", net);
        cursor.prefix = true;
        println!("prefix: {:?}/{}", std::net::Ipv4Addr::from(net), len);
    }
    println!("trie: {:?}", trie);

    let spfx = Prefix::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32);
    let s_spfx = longest_matching_prefix(&spfx, &trie);
    println!("search: {:?}", spfx);
    println!("lmp: {:?}", s_spfx);

    let spfx = Prefix::new(std::net::Ipv4Addr::new(64, 10, 10, 10).into(), 32);
    let s_spfx = longest_matching_prefix(&spfx, &trie);
    println!("search: {:?}", spfx);
    println!("lmp: {:?}", s_spfx);
}
