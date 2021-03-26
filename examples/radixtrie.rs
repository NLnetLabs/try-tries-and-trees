use trie::common::{NoMeta, Prefix, PrefixAs};
use trie::radixtrie::RadixTrie;

type Prefix4WithAs<'a> = Prefix<u32, PrefixAs>;
type Prefix4NoMeta<'a> = Prefix<u32, NoMeta>;

fn main() {
    let mut trie = RadixTrie::<u32, PrefixAs>::new();

    let pfxs = [
        Prefix4WithAs::new_with_meta(
            0b1111_0000_0000_0000_0000_0000_1111_1111_u32,
            32,
            PrefixAs(1),
        ),
        Prefix4WithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            32,
            PrefixAs(1),
        ),
        Prefix4WithAs::new_with_meta(
            0b0100_0000_0110_0000_0111_0000_1101_0011_u32,
            32,
            PrefixAs(2),
        ),
        Prefix4WithAs::new_with_meta(
            0b1100_0000_0000_0011_0000_0000_0010_0000_u32,
            32,
            PrefixAs(2),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
            3,
            PrefixAs(300),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(224, 0, 0, 0).into(),
            16,
            PrefixAs(3),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(193, 0, 14, 0).into(),
            24,
            PrefixAs(3),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(193, 0, 15, 0).into(),
            24,
            PrefixAs(4),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(193, 0, 16, 0).into(),
            24,
            PrefixAs(5),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(193, 0, 17, 0).into(),
            24,
            PrefixAs(6),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(193, 0 , 0, 0).into(),
            20,
            PrefixAs(3),
        ),
        Prefix4WithAs::new_with_meta(
            std::net::Ipv4Addr::new(12, 0, 0, 0).into(),
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
    println!("{:#?}", trie);
    println!("------ end of inserts\n");

    for spfx in &[
        Prefix4NoMeta::new(0b0100_0001_0000_0000_0000_0000_1111_1111_u32, 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(64, 10, 10, 10).into(), 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(13, 10, 10, 10).into(), 24),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(12, 0, 0, 34).into(), 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(240, 0, 0, 255).into(), 32),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(193, 0, 14, 0).into(), 24),
        Prefix4NoMeta::new(std::net::Ipv4Addr::new(193, 0, 15, 0).into(), 24),
    ] {
        println!("search for: {:?}", spfx);
        let s_spfx = trie.match_longest_prefix(&spfx);
        println!("lmp: {:?}", s_spfx);
        println!("-----------");
    }
}
