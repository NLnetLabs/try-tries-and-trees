
use std::fmt;

#[derive(Debug)]
pub struct PrefixAs(pub u32);

pub struct NoMeta;
impl fmt::Debug for NoMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("blaffer")
    }
}

pub trait Meta
where
    Self: fmt::Debug + Sized,
{
    fn with_meta(net: u32, len: u8, meta: Option<Self>) -> Prefix<Self> {
        Prefix {
            net: net,
            len: len,
            meta: meta,
        }
    }
}

pub struct Prefix<T>
where
    T: Meta,
{
    pub net: u32,
    pub len: u8,
    meta: Option<T>,
}

impl<T> Prefix<T>
where
    T: Meta,
{
    pub fn new(net: u32, len: u8) -> Prefix<T> {
        T::with_meta(net, len, None)
    }
    pub fn new_with_meta(net: u32, len: u8, meta: T) -> Prefix<T> {
        T::with_meta(net, len, Some(meta))
    }
}

impl<T> Meta for T
where
    T: fmt::Debug,
{
    fn with_meta(net: u32, len: u8, meta: Option<T>) -> Prefix<T> {
        Prefix::<T> { net, len, meta }
    }
}

impl<T> fmt::Debug for Prefix<T>
where
    T: fmt::Debug + Meta,
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

#[derive(Debug)]
pub struct BinaryNode<T>
where
    T: fmt::Debug + Meta,
{
    pub prefix: Option<Prefix<T>>,
    pub left: Option<Box<BinaryNode<T>>>,
    pub right: Option<Box<BinaryNode<T>>>,
}

impl<'a, T> BinaryNode<T>
where
    T: fmt::Debug + Meta,
{
    pub fn new(pfx: Option<(u32, u8)>) -> BinaryNode<T> {
        BinaryNode {
            prefix: if let Some((net, len)) = pfx {
                Some(Prefix::<T>::new(net, len))
            } else {
                None
            },
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct TrieNode {
    pub prefix: bool,
    pub left: Option<Box<TrieNode>>,
    pub right: Option<Box<TrieNode>>,
}

impl TrieNode {
    pub fn new(pfx: bool) -> TrieNode {
        TrieNode {
            prefix: pfx,
            left: None,
            right: None,
        }
    }
}

