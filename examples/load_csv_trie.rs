use ansi_term::Colour;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;

use std::env;
use std::process;
use trie::common::{NoMeta, Prefix, PrefixAs, Trie};

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
    let mut trie = Trie::<u32, PrefixAs>::new();

    if let Err(err) = load_prefixes(&mut pfxs) {
        println!("error running example: {}", err);
        process::exit(1);
    }
    println!("finished loading {} prefixes...", pfxs.len());
    let start = std::time::Instant::now();
    for pfx in pfxs.iter() {
        trie.insert(&pfx);
    }
    let ready = std::time::Instant::now();
    println!(
        "finished building tree in {} msecs...",
        ready.checked_duration_since(start).unwrap().as_millis()
    );

    let (total_nodes, total_prefixes) = trie.1.iter().fold((0, 0), |total_n: (u64, u64), n| {
        (
            total_n.0 + n.nodes_num as u64,
            total_n.1 + n.prefixes_num as u64,
        )
    });
    println!("total intermediary nodes : {:?}", total_nodes);
    println!(
        "size of node: {} bytes",
        std::mem::size_of::<trie::common::TrieNode<u32, NoMeta>>()
    );
    println!(
        "memory used by nodes: {}kb",
        total_nodes * std::mem::size_of::<trie::common::TrieNode<u32, NoMeta>>() as u64 / 1024
    );
    println!("total prefix nodes counted: {:?}", total_prefixes);
    println!(
        "nodes per prefix: {}",
        total_nodes as f64 / total_prefixes as f64
    );

    println!("level\t[bars:prefix|nodes] nodes occupied/max nodes percentage_max_nodes_occupied prefixes");
    let bars = ["▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    const SCALE: u32 = 3500;

    for s in &trie.1 {
        print!("{}\t", s.level);
        let n = (s.nodes_num / SCALE) as usize;
        for x in 0..n {
            if x <= (s.prefixes_num / SCALE) as usize {
                print!("{}", Colour::Blue.paint("█"));
            } else {
                print!("{}", Colour::Green.paint("█"));
            }
        }
        if s.nodes_num / SCALE <= s.prefixes_num / SCALE {
            print!(
                "{}",
                Colour::Blue.paint(bars[((s.nodes_num % SCALE) / (SCALE / 7)) as usize]) //  = scale / 7
            );
        } else {
            print!(
                "{}",
                Colour::Green.paint(bars[((s.nodes_num % SCALE) / (SCALE / 7)) as usize]) //  = 1500 / 7
            );
        }

        println!(
            " {}/{} {:.2}% {}",
            s.nodes_num,
            u64::pow(2, s.level as u32),
            (s.nodes_num as f64 / u64::pow(2, s.level as u32) as f64) * 100.0,
            s.prefixes_num,
        );
    }

    let mut shell = Shell::new(trie);
    shell.new_command("s", "search the RIB", 1, |io, trie, s| {
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
        let s_pfx = trie.match_longest_prefix(&pfx);
        writeln!(io, "{:?}", s_pfx)?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}
