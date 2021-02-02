use std::error::Error;
use std::io;
use std::process;
use trie::trie::{NoMeta, Prefix, PrefixAs, Trie};

fn load_prefixes(pfxs: &mut Vec<Prefix<u32, PrefixAs>>) -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        // let pfx = Prefix::<u32, PrefixAs>::new_with_meta(net, len, asn);
        let ip: Vec<_> = record[0]
            .split(".")
            .map(|o| -> u8 { o.parse().unwrap() })
            .collect();
        let net = std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
        let len: u8 = record[1].parse().unwrap();
        let asn: u32 = record[2].parse().unwrap();
        let pfx = Prefix::<u32, PrefixAs>::new_with_meta(net.into(), len, PrefixAs(asn));
        pfxs.push(pfx);
        // trie.insert(&pfx);
        // println!("{:?}", pfx);
    }
    Ok(())
}

fn main() {
    let mut pfxs: Vec<Prefix<u32, PrefixAs>> = vec![];
    let mut trie = Trie::<u32, PrefixAs>::new();
    if let Err(err) = load_prefixes(&mut pfxs) {
        println!("error running example: {}", err);
        process::exit(1);
    }
    println!("finished loading {} prefixes...", pfxs.len());
    for pfx in pfxs.iter() {
        trie.insert(&pfx);
    }
    println!("finished building tree...");

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(103, 108, 187, 247).into(), 32);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(103, 108, 187, 247).into(), 24);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(103, 108, 187, 247).into(), 23);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(214,10,23,0).into(), 24);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(214,10,23,0).into(), 0);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(0,0,0,0).into(), 0);
    let fpfx = trie.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);
}
