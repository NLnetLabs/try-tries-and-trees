use trie::nodes::*;

fn main() {
    type BinaryNode<'a> = BinaryNode2<'a, NoMeta>;
    let mut trie = BinaryNode::new(None);
    let mut cursor: &mut BinaryNode;

    let pfxs = [
        Prefix::<NoMeta>::new(0b0110_0000_0110_0000_0111_0000_1101_0000_u32, 3),
        Prefix::new(0b0100_0000_0110_0000_0111_0000_1101_0000_u32, 3),
        Prefix::new(0b1100_0000_0000_0011_0000_0000_0010_0000_u32, 3),
    ];

    for pfx in pfxs.iter() {
        cursor = &mut trie;

        let mut first_bit = pfx.net;
        let mut built_prefix: u32 = 0;

        for n in 0..pfx.len {
            println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            match first_bit.leading_ones() {
                0 => {
                    if !cursor.left.is_some() {
                        cursor.left = Some(Box::new(BinaryNode2::new(None)))
                    };
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                1..=32 => {
                    if !cursor.right.is_some() {
                        cursor.right = Some(Box::new(BinaryNode2::new(None)));
                    }
                    cursor = cursor.right.as_deref_mut().unwrap();
                    built_prefix = built_prefix + (0x01 << (31 - n));
                }
                _ => {
                    panic!("illegal prefix encountered. Giving up.");
                }
            }
            first_bit = first_bit << 1;
        }
        let len = pfx.len;
        cursor.prefix = Some(Prefix::new(built_prefix, len));
        println!(
            "prefix: {:?}/{}",
            std::net::Ipv4Addr::from(built_prefix),
            len
        );
    }
    println!("trie: {:?}", trie);
}
