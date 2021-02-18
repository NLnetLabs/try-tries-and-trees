use trie::common::{Prefix, PrefixAs};
use trie::treebitmap::TreeBitMap;
type Prefix4PrefixAs<'a> = Prefix<u32, PrefixAs>;

fn main() {
    let mut tree_bitmap = TreeBitMap::new();
    // let pfx1 = Prefix::<u32, PrefixAs>::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32,4);
    let pfx1 = Prefix::<u32, PrefixAs>::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4);
    let pfx2 = Prefix::<u32, PrefixAs>::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4);
    let pfx3 = Prefix::<u32, PrefixAs>::new(0b1000_0011_1000_1111_0000_0000_0000_0000_u32, 11);
    let pfx4 = Prefix::<u32, PrefixAs>::new(0b1000_0010_0011_0111_1111_0000_0000_0000_u32, 16);
    let pfx5 = Prefix::<u32, PrefixAs>::new(0b1000_0010_0101_0111_1111_1000_0000_0000_u32, 18);

    // let pfx5 = Prefix::<u32, PrefixAs>::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32,4);

    tree_bitmap.insert(&pfx1);
    tree_bitmap.insert(&pfx2);
    tree_bitmap.insert(&pfx3);
    tree_bitmap.insert(&pfx4);
    tree_bitmap.insert(&pfx5);

    // println!("pfxbitarr: {:032b}", tree_bitmap.0.pfxbitarr);

    for spfx in &[
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(225, 0, 0, 0).into(), 4),
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 4),
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(224, 0, 0, 0).into(), 8),
        Prefix4PrefixAs::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32),
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(64, 10, 10, 10).into(), 32),
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(131,143,0,0).into(), 11),
        Prefix4PrefixAs::new(std::net::Ipv4Addr::new(12, 0, 0, 34).into(), 32),
    ] {
        println!("search for: {:?}", spfx);
        let s_spfx = tree_bitmap.match_longest_prefix(&spfx);
        println!("lmp: {:?}", s_spfx);
        println!("-----------");
    }
}
