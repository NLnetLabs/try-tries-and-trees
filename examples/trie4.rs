use trie::common::{NoMeta, Prefix, PrefixAs, Trie};
type Prefix4WithAs<'a> = Prefix<u32, PrefixAs>;
type Prefix4NoMeta<'a> = Prefix<u32, NoMeta>;

fn main() {
    let mut trie = Trie::<u32, PrefixAs>::new();

    let pfxs = [
        Prefix4WithAs::new_with_meta(
            0b1111_0000_0000_0000_0000_0000_1111_1111_u32,
            32,
            PrefixAs(1),
        ),
        Prefix4WithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            4,
            PrefixAs(1),
        ),
        Prefix4WithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            8,
            PrefixAs(2),
        ),
        Prefix4WithAs::new_with_meta(
            0b1100_0000_0000_0011_0000_0000_0010_0000_u32,
            3,
            PrefixAs(2),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(12, 0, 0, 34).into(),
            8,
            PrefixAs(100),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(12, 0, 0, 34).into(),
            32,
            PrefixAs(100),
        ),
        Prefix4WithAs::new_with_meta(std::net::Ipv4Addr::new(0, 0, 0, 0).into(), 0, PrefixAs(21)),
        Prefix4WithAs::new_with_meta(std::net::Ipv4Addr::new(0, 0, 0, 0).into(), 1, PrefixAs(21)),
    ];

    for pfx in pfxs.iter() {
        trie.insert(pfx);
    }
    println!("------ end of inserts\n");

    for spfx in &[
        Prefix4NoMeta::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(64, 10, 10, 10).into(), 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(13, 10, 10, 10).into(), 24),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(12, 0, 0, 34).into(), 32),
    ] {
        println!("search for: {:?}", spfx);
        let s_spfx = trie.match_longest_prefix(&spfx);
        println!("lmp: {:?}", s_spfx);
        println!("-----------");
    }
}
