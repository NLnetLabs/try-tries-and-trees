use crate::trie::PrefixAs;
pub struct BitMap8stride (u8);

pub struct TreeBitMapNode {
    ptrbitarr: BitMap8stride,
    pfxbitarr: BitMap8stride,
    ptrblk: (Option<usize>, Option<usize>)
}

// pub struct TreeBitMap {
//     fn new() {}
// }