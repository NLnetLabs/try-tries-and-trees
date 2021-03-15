use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;
use trie::common::{NoMeta, Prefix, PrefixAs};
use trie::treebitmap::TreeBitMap;

use shrust::{Shell, ShellIO};
use std::io::prelude::*;

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
    let mut tree_bitmap: TreeBitMap<u32, PrefixAs> = TreeBitMap::new();

    if let Err(err) = load_prefixes(&mut pfxs) {
        println!("error running example: {}", err);
        process::exit(1);
    }
    println!("finished loading {} prefixes...", pfxs.len());
    for pfx in pfxs.iter() {
        tree_bitmap.insert(pfx);
    }

    println!("finished building tree...");
    // println!(
    //     "{:?} nodes created",
    //     tree_bitmap.stats.iter().fold(0, |mut acc, c| {
    //         acc += c.created_nodes.iter().fold(0, |mut sum, l| {
    //             sum += l.count;
    //             sum
    //         });
    //         acc
    //     })
    // );
    println!("stride division  {:?}", TreeBitMap::<u32, PrefixAs>::STRIDES);
    for s in &tree_bitmap.stats {
        println!("{:?}", s);
    }

    let spfx = Prefix::new(std::net::Ipv4Addr::new(193, 0, 10, 0).into(), 23);
    let fpfx = tree_bitmap.match_longest_prefix(&spfx);
    println!("search for: {:?}, found {:?}", spfx, fpfx);

    let mut shell = Shell::new(tree_bitmap);
    shell.new_command("s", "search the RIB", 1, |io, tree_bitmap, s| {
        let s_pref: Vec<&str> = s[0].split("/").collect();
        let len = s_pref[1].parse::<u8>().unwrap();
        let s_net: Vec<u8> = s_pref[0]
            .split(".")
            .map(|o| -> u8 { o.parse::<u8>().unwrap() })
            .collect();
        let pfx = Prefix::<u32, NoMeta>::new(
            std::net::Ipv4Addr::new(s_net[0], s_net[1], s_net[2], s_net[3]).into(),
            len,
        );
        let s_pfx = tree_bitmap.match_longest_prefix(&pfx);
        writeln!(io, "{:?}", s_pfx)?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(7, 0, 0, 0).into(), 22);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(103, 108, 187, 247).into(), 24);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(103, 108, 187, 247).into(), 23);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(214,10,23,0).into(), 24);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(214,10,23,0).into(), 0);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);

    // let spfx = Prefix::<u32, NoMeta>::new(std::net::Ipv4Addr::new(0,0,0,0).into(), 0);
    // let fpfx = trie.match_longest_prefix(&spfx);
    // println!("search for: {:?}, found {:?}", spfx, fpfx);
}
