pub mod nodes {
    use std::fmt;

    #[derive(Debug)]
    pub struct BinaryNode {
        pub prefix: String,
        pub left: Option<Box<BinaryNode>>,
        pub right: Option<Box<BinaryNode>>,
    }

    impl BinaryNode {
        pub fn new(b: String) -> BinaryNode {
            BinaryNode {
                prefix: b,
                left: None,
                right: None,
            }
        }
    }

    #[derive(Debug)]
    pub struct PrefixAs(pub u32);

    #[derive(Debug)]
    pub struct NoMeta;

    pub trait Meta
    where
        Self: fmt::Debug + Sized,
    {
        fn with_meta(net: u32, len: u8, meta: Option<&Self>) -> Prefix<Self> {
            Prefix {
                net: net,
                len: len,
                meta: meta,
            }
        }
    }

    pub struct Prefix<'a, T>
    where
        T: 'a + Meta + fmt::Debug,
    {
        pub net: u32,
        pub len: u8,
        meta: Option<&'a T>,
    }

    impl<'a, T> Prefix<'a, T>
    where
        T: Meta,
    {
        pub fn new(net: u32, len: u8) -> Prefix<'a, T> {
            T::with_meta(net, len, None)
        }
        pub fn new_with_meta(net: u32, len: u8, meta: &'a T) -> Prefix<'a, T> {
            T::with_meta(net, len, Some(meta))
        }
    }

    impl<T> Meta for T
    where
        T: fmt::Debug,
    {
        fn with_meta(net: u32, len: u8, meta: Option<&T>) -> Prefix<T> {
            Prefix::<T> { net, len, meta }
        }
    }

    impl<'a, T> fmt::Debug for Prefix<'a, T>
    where
        T: fmt::Debug + Meta,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!(
                "{}/{} {:?}",
                &std::net::Ipv4Addr::from(self.net),
                self.len.to_string(),
                self.meta
            ))
        }
    }

    #[derive(Debug)]
    pub struct BinaryNode2<'a, T> where T: fmt::Debug + Meta {
        pub prefix: Option<Prefix<'a, T>>,
        pub left: Option<Box<BinaryNode2<'a, T>>>,
        pub right: Option<Box<BinaryNode2<'a, T>>>,
    }

    impl<'a, T> BinaryNode2<'a, T> where T: fmt::Debug + Meta {
        pub fn new(pfx: Option<(u32, u8)>) -> BinaryNode2<'a, T> {
            BinaryNode2 {
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

    #[derive(Debug)]
    pub struct TrieNodePointer<'a, T> where T: fmt::Debug {
        pub prefix: Option<&'a Prefix<'a, T>>,
        pub left: Option<Box<TrieNode>>,
        pub right: Option<Box<TrieNode>>,
    }
}

pub mod triebitvec;
