use trie::common::{Prefix, PrefixAs};
use trie::treebitmap::TreeBitMap;

fn main() {
    let mut tree_bitmap = TreeBitMap::new();
    // let pfx1 = Prefix::<u32, PrefixAs>::new(0b0110_0000_0000_0000_0000_0000_0000_0000_u32,4);
    let pfx1 = Prefix::<u32, PrefixAs>::new(0b1110_0000_0000_0000_0000_0000_0000_0000_u32, 4);
    let pfx2 = Prefix::<u32, PrefixAs>::new(0b0011_0000_0000_0000_0000_0000_0000_0000_u32, 4);
    let pfx3 = Prefix::<u32, PrefixAs>::new(0b1000_0010_1000_0000_0000_0000_0000_0000_u32, 11);
    let pfx4 = Prefix::<u32, PrefixAs>::new(0b1000_0010_0011_0111_1111_0000_0000_0000_u32, 16);


    // let pfx5 = Prefix::<u32, PrefixAs>::new(0b1111_0000_0000_0000_0000_0000_0000_0000_u32,4);

    tree_bitmap.insert(&pfx1);
    tree_bitmap.insert(&pfx2);
    tree_bitmap.insert(&pfx3);
    tree_bitmap.insert(&pfx4);
    // tree_bitmap.insert(&pfx5);

    // println!("pfxbitarr: {:032b}", tree_bitmap.0.pfxbitarr);
}
