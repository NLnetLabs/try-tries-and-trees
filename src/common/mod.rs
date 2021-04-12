use num::PrimInt;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::ops::BitOr;

pub struct TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    pub prefix: Option<&'a Prefix<AF, T>>,
    pub left: Option<Box<TrieNode<'a, AF, T>>>,
    pub right: Option<Box<TrieNode<'a, AF, T>>>,
}

impl<'a, AF, T> TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    pub fn new(pfx: Option<&'a Prefix<AF, T>>) -> TrieNode<'a, AF, T> {
        TrieNode::<'a, AF, T> {
            prefix: pfx,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct PrefixAs(pub u32);

pub struct NoMeta;

impl fmt::Debug for NoMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("")
    }
}

pub trait Meta<AF>
where
    Self: fmt::Debug + Sized,
    AF: AddressFamily + PrimInt + Debug,
{
    fn with_meta(net: AF, len: u8, meta: Option<Self>) -> Prefix<AF, Self> {
        Prefix {
            net: net,
            len: len,
            meta: meta,
        }
    }
}

pub trait AddressFamily: PrimInt + Debug {
    const BITMASK: Self;
    const BITS: u8;
    fn fmt_net(net: Self) -> String;
    // returns the specified nibble from `start_bit` to (and
    // including) `start_bit + len` and shifted to the right.
    fn get_nibble(net: Self, start_bit: u8, len: u8) -> u32;
}

impl AddressFamily for u32 {
    const BITMASK: u32 = 0x1u32.rotate_right(1);
    const BITS: u8 = 32;

    fn fmt_net(net: Self) -> String {
        std::net::Ipv4Addr::from(net).to_string()
    }

    fn get_nibble(net: Self, start_bit: u8, len: u8) -> u32 {
        (net << start_bit) >> ((32 - len) % 32)
    }
}

impl AddressFamily for u128 {
    const BITMASK: u128 = 0x1u128.rotate_right(1);
    const BITS: u8 = 128;
    fn fmt_net(net: Self) -> String {
        std::net::Ipv6Addr::from(net).to_string()
    }

    fn get_nibble(net: Self, start_bit: u8, len: u8) -> u32 {
        ((net << start_bit) >> ((128 - len) % 128)) as u32
    }
}

pub struct IPv4(u32);

impl BitOr for IPv4 {
    // rhs is the "right-hand side" of the expression `a | b`
    type Output = Self;
    fn bitor(self, rhs: Self) -> IPv4 {
        Self(self.0 | rhs.0)
    }
}

// #[derive(Debug)]
pub struct Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily + PrimInt + Debug,
{
    pub net: AF,
    pub len: u8,
    pub meta: Option<T>,
}

impl<T, AF> Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily + PrimInt + Debug,
{
    pub fn new(net: AF, len: u8) -> Prefix<AF, T> {
        T::with_meta(net, len, None)
    }
    pub fn new_with_meta(net: AF, len: u8, meta: T) -> Prefix<AF, T> {
        T::with_meta(net, len, Some(meta))
    }
    pub fn strip_meta(self: &Self) -> Prefix::<AF, NoMeta> {
        Prefix::<AF, NoMeta> { net: self.net, len: self.len, meta: None }
    }
}

impl<T, AF> Meta<AF> for T
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    fn with_meta(net: AF, len: u8, meta: Option<T>) -> Prefix<AF, T> {
        Prefix::<AF, T> { net, len, meta }
    }
}

impl<AF, T> Ord for Prefix<AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (self.net >> (AF::BITS - self.len) as usize)
            .cmp(&(other.net >> ((AF::BITS - other.len) % 32) as usize))
    }
}

impl<AF, T> PartialEq for Prefix<AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.net >> (AF::BITS - self.len) as usize
            == other.net >> ((AF::BITS - other.len) % 32) as usize
    }
}

impl<AF, T> PartialOrd for Prefix<AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            (self.net >> (AF::BITS - self.len) as usize)
                .cmp(&(other.net >> ((AF::BITS - other.len) % 32) as usize)),
        )
    }
}

impl<AF, T> Eq for Prefix<AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug,
{
}

impl<T, AF> Debug for Prefix<AF, T>
where
    AF: AddressFamily + PrimInt + Debug,
    T: Debug + Meta<AF>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}/{} with {:?}",
            AddressFamily::fmt_net(self.net),
            self.len,
            self.meta
        ))
    }
}

#[derive(Debug)]
pub struct TrieLevelStats {
    pub level: u8,
    pub nodes_num: u32,
    pub prefixes_num: u32,
}

pub struct Trie<'a, AF, T>(TrieNode<'a, AF, T>, pub Vec<TrieLevelStats>)
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug;

impl<'a, AF, T> Trie<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + Debug + fmt::Binary,
{
    pub fn new() -> Trie<'a, AF, T> {
        Trie(
            TrieNode {
                prefix: None,
                left: None,
                right: None,
            },
            (0..33)
                .collect::<Vec<u8>>()
                .into_iter()
                .map(|level| TrieLevelStats {
                    level,
                    nodes_num: 0,
                    prefixes_num: 0,
                })
                .collect(),
        )
    }

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>) {
        let mut cursor = &mut self.0;

        let mut first_bit = pfx.net;
        let mut built_prefix: AF = num::zero();
        let zero = num::zero();
        let mut level: usize = 0;

        for _ in 0..pfx.len {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            match first_bit & AF::BITMASK {
                b if b == zero => {
                    if !cursor.left.is_some() {
                        // new node on the left
                        self.1[level + 1].nodes_num += 1;

                        cursor.left = Some(Box::new(TrieNode::new(None)))
                    };
                    built_prefix = built_prefix << 1;
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                _ => {
                    if !cursor.right.is_some() {
                        // new node on the right
                        self.1[level + 1].nodes_num += 1;

                        cursor.right = Some(Box::new(TrieNode::new(None)));
                    }
                    built_prefix = built_prefix << 1 | num::one();
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            }
            first_bit = first_bit << num::one();
            level += 1;
        }
        // println!("bp: {:b}", built_prefix);

        // let len = pfx.len;
        // let shift: usize = (AF::BITS - pfx.len) as usize;
        // let net = built_prefix << if shift < AF::BITS as usize { shift } else { 0 };

        if cursor.prefix.is_none() {
            self.1[level].prefixes_num += 1;
        }
        // println!("{:b}", net);
        cursor.prefix = Some(&pfx);
        // println!("inserted prefix: {:?}/{}", AF::fmt_net(net), len);
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<AF, NoMeta>,
    ) -> Option<&'a Prefix<AF, T>> {
        let mut cursor = &self.0;
        let mut cursor_pfx: AF = num::zero();
        let mut match_pfx: Option<&'a Prefix<AF, T>> = None;
        // let mut build_pfx = num::zero();
        // let mut match_len = 0;
        let zero: AF = num::zero();
        let mut first_bit = search_pfx.net;

        for _ in 0..(search_pfx.len + 1) {
            if let Some(found_pfx) = cursor.prefix {
                match_pfx = Some(found_pfx);
                // build_pfx = cursor_pfx;
                // match_len = i;
                // let shift = if i > 0 { (AF::BITS - i) as usize } else { 0 };
                // println!(
                //     "less-specific: {}/{} with {:?}",
                //     AF::fmt_net(cursor_pfx << shift).as_str(),
                //     match_len,
                //     found_pfx.meta
                // );
            }

            match first_bit & AF::BITMASK {
                b if b == zero => {
                    if cursor.left.is_some() {
                        cursor = cursor.left.as_deref().unwrap();
                        cursor_pfx = cursor_pfx << num::one();
                    } else {
                        break;
                    }
                }
                _ => {
                    if cursor.right.is_some() {
                        cursor = cursor.right.as_deref().unwrap();
                        cursor_pfx = cursor_pfx << num::one() | num::one();
                    } else {
                        break;
                    }
                }
            }
            first_bit = first_bit << 1;
        }

        // if match_len > 0 {
            // let build_pfx_net = AF::fmt_net(build_pfx << (AF::BITS - match_len) as usize);
        //     println!("built prefix: {}/{}", build_pfx_net.as_str(), match_len);
        // }
        // println!("{:?}", match_pfx);
        match_pfx
    }
}
