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
    pub struct Prefix {
        pub net: u32,
        pub len: u8,
    }

    impl Prefix {
        pub fn new(net: u32, len: u8) -> Prefix {
            Prefix { net: net, len: len }
        }
    }

    impl fmt::Debug for Prefix {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!(
                "{}/{}",
                &std::net::Ipv4Addr::from(self.net),
                self.len.to_string()
            ))
        }
    }

    #[derive(Debug)]
    pub struct BinaryNode2 {
        pub prefix: Option<Prefix>,
        pub left: Option<Box<BinaryNode2>>,
        pub right: Option<Box<BinaryNode2>>,
    }

    impl BinaryNode2 {
        pub fn new(pfx: Option<(u32, u8)>) -> BinaryNode2 {
            BinaryNode2 {
                prefix: if let Some((net, len)) = pfx {
                    Some(Prefix { net: net, len: len })
                } else {
                    None
                },
                left: None,
                right: None,
            }
        }
    }
}
