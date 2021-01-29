use trie::nodes::*;

// fn main() {
//     let mut trie = BinaryNode {
//         prefix: "root".to_string(),
//         left: None,
//         right: None,
//     };

//     let bn = BinaryNode {
//         prefix: "01".to_string(),
//         left: None,
//         right: None,
//     };
//     trie.right = Some(Box::new(bn));
//     let mut cursor = trie.right.as_deref_mut().unwrap();

//     let bn2 = BinaryNode {
//         prefix: "011".to_string(),
//         left: None,
//         right: None,
//     };
//     cursor.left = Some(Box::new(bn2));
//     cursor = cursor.left.as_deref_mut().unwrap();

//     let bn3 = BinaryNode {
//         prefix: "0111".to_string(),
//         left: None,
//         right: None,
//     };

//     cursor.left = Some(Box::new(bn3));
//     cursor = cursor.left.as_deref_mut().unwrap();

//     println!("cursor: {:?}", cursor);
//     println!("trie: {:?}", trie);
// }

fn use_str() {
    let mut trie = BinaryNode::new("root".to_string());
    let pref_str = "0110";

    let mut cursor = &mut trie;

    for bn in pref_str.chars() {
        match bn.to_digit(2).unwrap() {
            0 => {
                cursor.left = Some(Box::new(BinaryNode::new("0".to_string())));
                cursor = cursor.left.as_deref_mut().unwrap();
            }
            1 => {
                cursor.right = Some(Box::new(BinaryNode::new("1".to_string())));
                cursor = cursor.right.as_deref_mut().unwrap();
            }
            _ => {
                panic!("illegal prefix encountered. Giving up.")
            }
        }
    }
    println!("trie: {:?}", trie);
}

fn main() {
    let mut trie = BinaryNode2::new(None);
    let mut cursor: &mut BinaryNode2;

    let pfxs = [
        Prefix::new(0b0110_0000_0110_0000_0111_0000_1101_0000_u32, 3),
        Prefix::new(0b0100_0000_0110_0000_0111_0000_1101_0000_u32, 3),
        Prefix::new(0b1100_0000_0000_0011_0000_0000_0010_0000_u32, 3),
    ];
    // let net_addr = std::net::Ipv4Addr::from(pfx.net);
    // println!("{}", net_addr);
    // println!("{}", pfx.net);

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
        cursor.prefix = Some(Prefix {
            net: built_prefix,
            len: len,
        });
        println!(
            "prefix: {:?}/{}",
            std::net::Ipv4Addr::from(built_prefix),
            len
        );
    }
    println!("trie: {:?}", trie);
}
