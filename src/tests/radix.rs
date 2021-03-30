#[cfg(test)]
mod test {
    use crate::common::{Prefix, NoMeta, PrefixAs};
    use crate::radixtrie::RadixTrie;

    #[test]
    fn test_insert_extremes_ipv4() {
        let trie = &mut RadixTrie::<u32, NoMeta>::new();
       
        let min_pfx = Prefix::new(std::net::Ipv4Addr::new(0, 0, 0, 0).into(), 1);
        trie.insert(&min_pfx);
        let res = trie.match_longest_prefix(&min_pfx).unwrap();
        assert_eq!(res, &min_pfx);

        let max_pfx = Prefix::new(std::net::Ipv4Addr::new(255, 255, 255, 255).into(), 32);
        trie.insert(&max_pfx);
        let res = trie.match_longest_prefix(&max_pfx).unwrap();
        assert_eq!(res, &max_pfx);
    }

    #[test]
    fn test_tree_ipv4() {
        let mut trie: RadixTrie<u32, PrefixAs> = RadixTrie::new();
        let pfxs = vec![
            // Prefix::<u32, PrefixAs>::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 0),
            // Prefix::<u32, PrefixAs>::new(0b1111_1111_1111_1111_1111_1111_1111_1111_u32, 32),
            // Prefix::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b0111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 9),
            // Prefix::new(0b1111_0000_1000_0000_0000_0000_0000_0000_u32, 9),
            // Prefix::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 12),
            // Prefix::<u32, PrefixAs>::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 9),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 9),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 10),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 11),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 12),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 12),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 13),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 13),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 14),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_0100_0000_0000_0000_0000_0000_u32, 14),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 14),
            // Prefix::<u32, PrefixAs>::new(0b0111_0111_1100_0000_0000_0000_0000_0000_u32, 14),
            // Prefix::<u32, PrefixAs>::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 23),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 4),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 16),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 10, 0).into(), 23),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 9, 0).into(), 24),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 0, 0).into(), 23),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 23),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(209, 0, 0, 0).into(), 16),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 9, 0).into(), 24),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 24),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 14, 0).into(), 23),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 14, 0).into(), 24),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 15, 0).into(), 24),
            Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 10, 10).into(), 32),
            // Prefix::<u32, PrefixAs>::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
            // Prefix::<u32, PrefixAs>::new(0b1000_0011_1000_1111_0000_0000_0000_0000_u32, 11),
            // Prefix::<u32, PrefixAs>::new(0b1000_0010_0101_0111_1111_1000_0000_0000_u32, 13),
            // Prefix::new(std::net::Ipv4Addr::new(130, 55, 240, 0).into(), 24),
            // Prefix::<u32, PrefixAs>::new(0b1111_1111_0000_0001_0000_0000_0000_0000_u32, 12),
            // Prefix::<u32, PrefixAs>::new(0b1111_1111_0011_0111_0000_0000_0000_0000_u32, 17),
            // Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(100, 0, 12, 0).into(), 24),
            // Prefix::new(0b0000_0001_0000_0000_0000_0000_0000_0000_u32, 24),
            // Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(1, 0, 128, 0).into(), 24),
        ];

        for pfx in pfxs.iter() {
            trie.insert(pfx);
        }

        println!("{:#?}", trie);
        
        for pfx in pfxs.iter() {
            let pfx_nm = pfx.strip_meta();
            let res = trie.match_longest_prefix(&pfx_nm).unwrap();
            assert_eq!(res, pfx);
        }

        let res = trie.match_longest_prefix(&Prefix::<u32, NoMeta>::new(
            std::net::Ipv4Addr::new(192, 0, 1, 0).into(),
            24,
        )).unwrap();

        assert_eq!(
            res,
            &Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 16)
        );
    }

    #[test]
    fn test_ranges_ipv4() {
        for i_net in [0, 4, 8, 12, 14, 15, 16, 32, 65, 127, 213, 254 as u8].iter() {
            let mut tree_bitmap: RadixTrie<u32, NoMeta> = RadixTrie::new();

            let pfx_vec: Vec<Prefix<u32, NoMeta>> = (1..32)
                .collect::<Vec<u8>>()
                .into_iter()
                .map(|i_len| {
                    Prefix::<u32, NoMeta>::new(
                        std::net::Ipv4Addr::new(*i_net, 0, 0, 0).into(),
                        i_len,
                    )
                })
                .collect();

            let mut i_len_s = 0;
            for pfx in &pfx_vec {
                i_len_s += 1;
                tree_bitmap.insert(&pfx);

                let res_pfx = Prefix::<u32, NoMeta>::new(
                    std::net::Ipv4Addr::new(*i_net, 0, 0, 0).into(),
                    i_len_s,
                );

                for s_len in i_len_s..32 {
                    let pfx = Prefix::<u32, NoMeta>::new(
                        std::net::Ipv4Addr::new(*i_net, 0, 0, 0).into(),
                        s_len,
                    );
                    let res = tree_bitmap.match_longest_prefix(&pfx);
                    println!("{:?}", pfx);

                    assert_eq!(res.unwrap(), &res_pfx);
                }
            }
        }
    }
}
