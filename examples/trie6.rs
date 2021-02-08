use trie::common::{Trie, Prefix, PrefixAs, NoMeta};
type Prefix6WithAs<'a> = Prefix<u128, PrefixAs>;
type Prefix6NoMeta<'a> = Prefix<u128, NoMeta>;

fn main() {
    let mut trie = Trie::<u128, PrefixAs>::new();

    let pfx32 = Prefix::<u32, NoMeta>::new(0b1000_0000_0000_0000_0000_0000_0000_0000_u32, 32);
    println!("{:?}", pfx32);

    let num128 = 0b0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_00000_0000_0000_0000_00000_0000_0000_0000_00000_0000_0000_u128;
    println!("{}", num128);
    let pfxs = [
        Prefix6WithAs::new_with_meta(
            num128,
            128,
            PrefixAs(1),
        ),
        Prefix6WithAs::new_with_meta(
            std::net::Ipv6Addr::new(0x2001, 0x98e, 0x248, 0x34, 0x12, 0x12, 0x12,0x12).into(),
            48,
            PrefixAs(100),
        ),
        Prefix6WithAs::new_with_meta(
            std::net::Ipv6Addr::new(0x12, 0x0, 0x0, 0x0, 0x0, 0 , 0,0).into(),
            48,
            PrefixAs(100),
        ),
        Prefix6WithAs::new_with_meta(
            std::net::Ipv6Addr::new(0x12, 0x0, 0x0, 0x34, 0x10, 0 , 0,0).into(),
            64,
            PrefixAs(100),
        ),
    ];

    for pfx in pfxs.iter() {
        trie.insert(pfx);
    }
 
    let spfx = Prefix6NoMeta::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u128, 32);
    println!("search for: {:?}", spfx);

    let s_spfx= trie.match_longest_prefix(&spfx);
    println!("lmp: {:?}", s_spfx);

    let spfx = Prefix6NoMeta::new(std::net::Ipv6Addr::new(64, 10, 10, 10, 0,0,0,0).into(), 32);
    println!("search for: {:?}", spfx);

    let s_spfx = trie.match_longest_prefix(&spfx);
    println!("lmp: {:?}", s_spfx);

    let spfx = Prefix6NoMeta::new(std::net::Ipv6Addr::new(0x12, 0x0, 0x00, 0x34, 0x0,0x0,0x0, 0x0 ).into(), 128);
    println!("search for: {:?}", spfx);

    let s_spfx = trie.match_longest_prefix(&spfx);
    println!("lmp: {:?}", s_spfx);
}