use trie::common::{NoMeta, Prefix, PrefixAs};
use trie::treebitmap::TreeBitMap;
type Prefix4<'a> = Prefix<u32, NoMeta>;

fn main() {
    let mut tree_bitmap = TreeBitMap::new();
    let pfxs = vec![
        Prefix::<u32, PrefixAs>::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::<u32, PrefixAs>::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(193,0,0,0).into(), 24),
        Prefix::<u32, PrefixAs>::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4),
        Prefix::<u32, PrefixAs>::new(0b1000_0011_1000_1111_0000_0000_0000_0000_u32, 11),
        Prefix::<u32, PrefixAs>::new(0b1000_0010_0101_0111_1111_1000_0000_0000_u32, 13),
        Prefix::<u32, PrefixAs>::new(0b1111_1111_0000_0001_0000_0000_0000_0000_u32, 12),
        Prefix::<u32, PrefixAs>::new(0b1111_1111_0011_0111_0000_0000_0000_0000_u32, 17),
        Prefix::<u32, PrefixAs>::new(std::net::Ipv4Addr::new(100, 0, 0, 0).into(), 16),
    ];

    for pfx in pfxs.iter() {
        tree_bitmap.insert(pfx);
    }
    println!("------ end of inserts\n");

    // println!("pfxbitarr: {:032b}", tree_bitmap.0.pfxbitarr);

    for spfx in &[
        Prefix4::new(std::net::Ipv4Addr::new(255, 1, 0, 0).into(), 24),
        Prefix4::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 4),
        Prefix4::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 8),
        Prefix4::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32),
        Prefix4::new(std::net::Ipv4Addr::new(1, 0, 100, 10).into(), 16),
        Prefix4::new(std::net::Ipv4Addr::new(131, 143, 0, 0).into(), 11),
        Prefix4::new(std::net::Ipv4Addr::new(12, 0, 0, 34).into(), 32),
        Prefix::new(std::net::Ipv4Addr::new(130,55,240,0).into(),24),
        Prefix::new(std::net::Ipv4Addr::new(193,0,0,0).into(),24),
        Prefix::new(std::net::Ipv4Addr::new(100, 0,12,0).into(),24)

    ] {
        println!("search for: {:?}", spfx);
        let s_spfx = tree_bitmap.match_longest_prefix(&spfx);
        println!("lmp: {:?}", s_spfx);
        println!("-----------");
    }
}
