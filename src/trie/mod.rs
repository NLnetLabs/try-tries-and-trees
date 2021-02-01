use num::traits::int::PrimInt;
use std::fmt;
use std::fmt::Debug;
pub struct TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily,
{
    pub prefix: Option<&'a Prefix<AF, T>>,
    pub left: Option<Box<TrieNode<'a, AF, T>>>,
    pub right: Option<Box<TrieNode<'a, AF, T>>>,
}

impl<'a, AF, T> TrieNode<'a, AF, T>
where
    T: fmt::Debug,
    AF: AddressFamily,
{
    pub fn new(pfx: Option<&'a Prefix<AF, T>>) -> TrieNode<'a, AF, T> {
        TrieNode::<'a, AF, T> {
            prefix: pfx,
            left: None,
            right: None,
        }
    }
}

impl<'a, AF> Debug for TrieNode<'a, AF, NoMeta>
where
    AF: AddressFamily,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug)]
pub struct PrefixAs(pub u32);
pub struct NoMeta;
impl fmt::Debug for NoMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("blaffer")
    }
}

pub trait Meta<AF>
where
    Self: fmt::Debug + Sized,
    AF: AddressFamily,
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
    fn get_family() -> u8;
}

impl AddressFamily for u32 {
    fn get_family() -> u8 {
        4
    }
}

impl AddressFamily for u128 {
    fn get_family() -> u8 {
        6
    }
}

pub struct Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily,
{
    pub net: AF,
    pub len: u8,
    meta: Option<T>,
}

impl<T, AF> Prefix<AF, T>
where
    T: Meta<AF>,
    AF: AddressFamily,
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
    AF: AddressFamily,
{
    fn with_meta(net: AF, len: u8, meta: Option<T>) -> Prefix<AF, T> {
        Prefix::<AF, T> { net, len, meta }
    }
}

impl<T> Debug for Prefix<u32, T>
where
    T: Debug + Meta<u32>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}/{} -> {:?}",
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
            "{}/{} -> {:?}",
            &std::net::Ipv6Addr::from(self.net),
            self.len.to_string(),
            self.meta
        ))
    }
}
pub struct Trie<'a, AF, T>(TrieNode<'a, AF, T>)
where
    T: Debug,
    AF: AddressFamily;

impl<'a, T> Trie<'a, u32, T>
where
    T: Debug,
{
    pub fn new() -> Trie<'a, u32, T> {
        Trie(TrieNode {
            prefix: None,
            left: None,
            right: None,
        })
    }

    pub fn insert(&mut self, pfx: &'a Prefix<u32, T>) {
        let mut cursor = &mut self.0;

        let mut first_bit: u32 = pfx.net;
        let mut built_prefix: u32 = 0;
        let bitmask: u32 = 0x1.rotate_right(1);

        for _ in 0..pfx.len {
            // println!("{:#b} : {}", first_bit, first_bit.leading_ones());
            match first_bit & bitmask {
                0 => {
                    if !cursor.left.is_some() {
                        // new node on the left
                        cursor.left = Some(Box::new(TrieNode::new(None)))
                    };
                    built_prefix = built_prefix << 1;
                    cursor = cursor.left.as_deref_mut().unwrap();
                }
                1..=u32::MAX => {
                    if !cursor.right.is_some() {
                        // new node on the right
                        cursor.right = Some(Box::new(TrieNode::new(None)));
                    }
                    built_prefix = built_prefix << 1 | 1;
                    cursor = cursor.right.as_deref_mut().unwrap();
                }
            }
            first_bit = first_bit << 1;
        }
        println!("bp: {:b}", built_prefix);

        let len = pfx.len;
        let net = built_prefix << (32 - pfx.len);

        println!("{:b}", net);
        cursor.prefix = Some(&pfx);
        println!("prefix: {:?}/{}", std::net::Ipv4Addr::from(net), len);
    }

    pub fn match_longest_prefix(
        &self,
        search_pfx: &Prefix<u32, NoMeta>,
    ) -> Option<&'a Prefix<u32, T>> {
        let mut cursor = &self.0;
        let mut cursor_pfx: u32 = 0;
        let mut match_pfx: Option<&'a Prefix<u32, T>> = None;
        let mut build_pfx = 0;
        let mut match_len = 0;
        let mut first_bit = search_pfx.net;
        let bitmask = 0x1_u32.rotate_right(1);

        for i in 1..search_pfx.len {
            match first_bit & bitmask {
                0 => {
                    if cursor.left.is_some() {
                        cursor = cursor.left.as_deref().unwrap();
                        cursor_pfx = cursor_pfx << 1;
                    } else {
                        break;
                    }
                }
                1..=u32::MAX => {
                    if cursor.right.is_some() {
                        cursor = cursor.right.as_deref().unwrap();
                        cursor_pfx = cursor_pfx << 1 | 1;
                    } else {
                        break;
                    }
                }
            }

            if let Some(found_pfx) = cursor.prefix {
                match_pfx = Some(found_pfx);
                build_pfx = cursor_pfx;
                match_len = i;
                println!(
                    "less-specific: {:?}/{}",
                    std::net::Ipv4Addr::from(cursor_pfx << (32 - match_len)),
                    match_len
                );
            }

            first_bit = first_bit << 1;
        }
        if match_len > 0 {
            println!(
                "built prefix: {:?}",
                Prefix::<u32, NoMeta>::new(build_pfx << (32 - match_len), match_len)
            );
        }
        match_pfx
    }
}
