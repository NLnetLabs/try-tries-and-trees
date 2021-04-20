use trie::common::{Prefix, PrefixAs, NoMeta};
use trie::treebitmap_univec::TreeBitMap;
type Prefix4<'a> = Prefix<u32, NoMeta>;

fn main() {
    let mut tree_bitmap: TreeBitMap<u32, PrefixAs> = TreeBitMap::new(vec![4]);
    let pfxs = vec![
        Prefix::<u32, PrefixAs>::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 0),
        Prefix::<u32, PrefixAs>::new(0b1111_1111_1111_1111_1111_1111_1111_1111_u32, 32),
        Prefix::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 9),
        Prefix::new(0b1111_0000_1000_0000_0000_0000_0000_0000_u32, 9),
        Prefix::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 12),
        Prefix::<u32, PrefixAs>::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 9),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 9),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 10),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 11),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 12),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 12),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 13),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 13),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_0000_0000_0000_0000_0000_0000_u32, 14),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_0100_0000_0000_0000_0000_0000_u32, 14),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 14),
        Prefix::<u32, PrefixAs>::new(0b0111_0111_1100_0000_0000_0000_0000_0000_u32, 14),
        Prefix::<u32, PrefixAs>::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 23),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 0, 0).into(), 16),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 10, 0).into(), 23),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(192, 0, 9, 0).into(), 24),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 0, 0).into(), 23),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 23),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(209, 0, 0, 0).into(), 16),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 9, 0).into(), 24),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 24),
        Prefix::<u32, PrefixAs>::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::<u32, PrefixAs>::new(0b1000_0011_1000_1111_0000_0000_0000_0000_u32, 11),
        Prefix::<u32, PrefixAs>::new(0b1000_0010_0101_0111_1111_1000_0000_0000_u32, 13),
        Prefix::new(std::net::Ipv4Addr::new(130, 55, 240, 0).into(), 24),
        Prefix::<u32, PrefixAs>::new(0b1111_1111_0000_0001_0000_0000_0000_0000_u32, 12),
        Prefix::<u32, PrefixAs>::new(0b1111_1111_0011_0111_0000_0000_0000_0000_u32, 17),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(100, 0, 12, 0).into(), 24),
        Prefix::new(0b0000_0001_0000_0000_0000_0000_0000_0000_u32, 24),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(1, 0, 128, 0).into(), 24),
    ];

    for pfx in pfxs.into_iter() {
        // println!("insert {:?}", pfx);
        tree_bitmap.insert(pfx);
    }
    println!("------ end of inserts\n");
    println!("{:#?}", tree_bitmap.prefixes);

    // println!("pfxbitarr: {:032b}", tree_bitmap.0.pfxbitarr);

    for spfx in &[
        Prefix::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 0),
        Prefix::new(0b0000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1000_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1001_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1010_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1100_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1101_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b0111_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32, 9),
        Prefix4::new(std::net::Ipv4Addr::new(255, 1, 0, 0).into(), 24),
        Prefix4::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 4),
        Prefix4::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 8),
        Prefix4::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32),
        Prefix4::new(std::net::Ipv4Addr::new(1, 0, 100, 10).into(), 16),
        Prefix4::new(std::net::Ipv4Addr::new(131, 143, 0, 0).into(), 11),
        Prefix4::new(std::net::Ipv4Addr::new(12, 0, 0, 34).into(), 32),
        Prefix::new(std::net::Ipv4Addr::new(130, 55, 240, 0).into(), 24),
        Prefix::new(std::net::Ipv4Addr::new(193, 0, 3, 0).into(), 23),
        Prefix::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 23),
        Prefix::new(0b0111_0111_1000_0000_0000_0000_0000_0000_u32, 14),
        Prefix::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 24),
        Prefix::new(std::net::Ipv4Addr::new(100, 0, 12, 0).into(), 24),
        Prefix::new(std::net::Ipv4Addr::new(255, 255, 255, 255).into(), 32),
        Prefix::new(std::net::Ipv4Addr::new(1, 0, 0, 0).into(), 24),
        Prefix::new(std::net::Ipv4Addr::new(1, 0, 128, 0).into(), 24),

    ] {
        println!("search for: {:?}", spfx);
        let s_spfx = tree_bitmap.match_longest_prefix(&spfx);
        println!("lmp: {:?}", s_spfx);
        println!("-----------");
    }
}
