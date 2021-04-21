use std::error::Error;
use std::ffi::OsString;
use std::fs::File;

use std::env;
use std::process;
use trie::common::{Prefix, PrefixAs, Trie};

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn load_prefixes(pfxs: &mut Vec<Prefix<u32, PrefixAs>>) -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
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
    for pfx in pfxs.iter() {
        trie.insert(&pfx);
    }

    let (total_nodes, total_prefixes) = trie.1.iter().fold((0, 0), |total_n: (u64, u64), n| {
        (
            total_n.0 + n.nodes_num as u64,
            total_n.1 + n.prefixes_num as u64,
        )
    });

    println!("{{");
    println!("\"total_nodes\": {},", total_nodes);
    println!(
        "\"node_size_b\": {},",
        std::mem::size_of::<trie::common::TrieNode<u32, PrefixAs>>()
    );
    println!(
        "\"nodes_mem_kb\": {},",
        total_nodes * std::mem::size_of::<trie::common::TrieNode<u32, PrefixAs>>() as u64 / 1024
    );
    println!("\"total_prefixes\": {:?},", total_prefixes);
    println!(
        "\"prefixes_mem_kb\": {:?},",
        total_prefixes as usize * std::mem::size_of::<Prefix<u32, PrefixAs>>() / 1024
    );
    println!(
        "\"prefixes_per_node\": {},",
        total_prefixes as f64 / total_nodes as f64
    );
    println!("\"levels\":{:#?}", trie.1);
    println!("}}");

    println!("counters: {:?}", trie.traverse_count());
}
