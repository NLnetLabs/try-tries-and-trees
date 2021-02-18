use num::PrimInt;
use std::fmt;
use std::fmt::Debug;
use std::ops::BitOr;
use std::cmp::Ordering;

pub struct TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt,
{
    pub prefix: Option<&'a Prefix<AF, T>>,
    pub left: Option<Box<TrieNode<'a, AF, T>>>,
    pub right: Option<Box<TrieNode<'a, AF, T>>>,
}

impl<'a, AF, T> TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily + PrimInt,
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
    AF: AddressFamily + PrimInt,
{
    fn with_meta(net: AF, len: u8, meta: Option<Self>) -> Prefix<AF, Self> {
        Prefix {
            net: net,
            len: len,
            meta: meta,
        }
    }
}

pub trait AddressFamily {
    const BITMASK: Self;
    const BITS: u8;
    fn fmt_net(net: Self) -> String;
}

impl AddressFamily for u32 {
    const BITMASK: u32 = 0x1u32.rotate_right(1);
    const BITS: u8 = 32;
    fn fmt_net(net: Self) -> String {
        std::net::Ipv4Addr::from(net).to_string()
    }
}

impl AddressFamily for u128 {
    const BITMASK: u128 = 0x1u128.rotate_right(1);
    const BITS: u8 = 128;
    fn fmt_net(net: Self) -> String {
        std::net::Ipv6Addr::from(net).to_string()
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
pub struct Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily + PrimInt,
{
    pub net: AF,
    pub len: u8,
    meta: Option<T>,
}

impl<T, AF> Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily + PrimInt,
{
    pub fn new(net: AF, len: u8) -> Prefix<AF, T> {
        T::with_meta(net, len, None)
    }
    pub fn new_with_meta(net: AF, len: u8, meta: T) -> Prefix<AF, T> {
        T::with_meta(net, len, Some(meta))
    }
}

impl<T, AF> Meta<AF> for T
where
    T: Debug,
    AF: AddressFamily + PrimInt,
{
    fn with_meta(net: AF, len: u8, meta: Option<T>) -> Prefix<AF, T> {
        Prefix::<AF, T> { net, len, meta }
    }
}

impl<AF, T> Ord for Prefix<AF, T> where T: Debug, AF: AddressFamily + PrimInt {
    fn cmp(&self, other: &Self) -> Ordering {
        self.net.cmp(&other.net)
    }
}

impl<AF, T> PartialEq for Prefix<AF, T> where T: Debug, AF: AddressFamily + PrimInt {
    fn eq(&self, other: &Self) -> bool {
        self.net == other.net
    }
}

impl<AF, T> PartialOrd for Prefix<AF, T> where T: Debug, AF: AddressFamily + PrimInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.net.cmp(&other.net))
    }
}

impl<AF, T> Eq for Prefix<AF, T> where T: Debug, AF: AddressFamily + PrimInt {}

impl<T> Debug for Prefix<u32, T>
where
    T: Debug + Meta<u32>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}/{} with {:?}",
            &std::net::Ipv4Addr::from(self.net),
            self.len.to_string(),
            self.meta
        ))
    }
}

impl<T> Debug for Prefix<u128, T>
where
    T: Debug + Meta<u128>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}/{} with {:?}",
            &std::net::Ipv6Addr::from(self.net),
            self.len.to_string(),
            self.meta
        ))
    }
}

pub struct Trie<'a, AF, T>(TrieNode<'a, AF, T>)
where
    T: Debug,
    AF: AddressFamily + PrimInt;

impl<'a, AF, T> Trie<'a, AF, T>
where
    T: Debug,
    AF: AddressFamily + PrimInt + fmt::Binary,
{
    pub fn new() -> Trie<'a, AF, T> {
        Trie(TrieNode {
            prefix: None,
            left: None,
            right: None,
        })
    }

    pub fn insert(&mut self, pfx: &'a Prefix<AF, T>) {
        let mut cursor = &mut self.0;

        let mut first_bit = pfx.net;
        let mut built_prefix: AF = num::zero();
        let zero = num::zero();

        for _ in 0..pfx.len {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            match first_bit & AF::BITMASK {
                b if b == zero => {
                    if !cursor.left.is_some() {
                        // new node on the left
                        cursor.left = Some(Box::new(TrieNode::new(None)))
                    };
                    built_prefix = built_prefix << 1;
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                _ => {
                    if !cursor.right.is_some() {
                        // new node on the right
                        cursor.right = Some(Box::new(TrieNode::new(None)));
                    }
                    built_prefix = built_prefix << 1 | num::one();
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            }
            first_bit = first_bit << num::one();
        }
        // println!("bp: {:b}", built_prefix);

        // let len = pfx.len;
        // let shift: usize = (AF::BITS - pfx.len) as usize;
        // let net = built_prefix << if shift < AF::BITS as usize { shift } else { 0 };

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
        let mut build_pfx = num::zero();
        let mut match_len = 0;
        let zero: AF = num::zero();
        let mut first_bit = search_pfx.net;

        for i in 0..(search_pfx.len + 1) {
            if let Some(found_pfx) = cursor.prefix {
                match_pfx = Some(found_pfx);
                build_pfx = cursor_pfx;
                match_len = i;
                let shift = if i > 0 { (AF::BITS - i) as usize } else { 0 };
                println!(
                    "less-specific: {}/{} with {:?}",
                    AF::fmt_net(cursor_pfx << shift).as_str(),
                    match_len,
                    found_pfx.meta
                );
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

        if match_len > 0 {
            let build_pfx_net = AF::fmt_net(build_pfx << (AF::BITS - match_len) as usize);
            println!("built prefix: {}/{}", build_pfx_net.as_str(), match_len);
        }

        match_pfx
    }
}
