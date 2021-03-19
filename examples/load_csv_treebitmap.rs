use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use trie::common::{NoMeta, Prefix, PrefixAs};
use trie::treebitmap::TreeBitMap;

use async_channel;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

async fn load_prefixes(
    tx: async_channel::Sender<Prefix<u32, PrefixAs>>,
) -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.records() {
        let tx = tx.clone();
        tokio::spawn(async move {

            let record = result.unwrap();
            let ip: Vec<_> = record[0]
                .split(".")
                .map(|o| -> u8 { o.parse().unwrap() })
                .collect();
            let net = std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
            let len: u8 = record[1].parse().unwrap();
            let asn: u32 = record[2].parse().unwrap();
            let pfx = Prefix::<u32, PrefixAs>::new_with_meta(net.into(), len, PrefixAs(asn));

            match tx.send(pfx).await {
                Ok(_) => {
                    // print!(".");
                }
                Err(err) => {
                    println!("err {:?} {}", err.into_inner(), tx.is_closed());
                }
            };
        });
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut tree_bitmap: TreeBitMap<u32, PrefixAs> = TreeBitMap::new();
    println!("start loading prefixes...");
    let start = std::time::Instant::now();

    let (tx, rx) = async_channel::bounded(10000);

    let store_tree = tokio::spawn(async move {
        let mut duplicate_cnt: usize = 0;
        loop {
            let rx = rx.clone();
            match rx.recv().await {
                Ok(pfx) => {
                    duplicate_cnt += !tree_bitmap.insert(pfx) as usize;
                }
                Err(err) => {
                    if rx.is_closed() {
                        println!("channel closed...");
                        println!("found {} duplicate prefixes", duplicate_cnt);
                        return tree_bitmap;
                    }
                    else {
                        panic!("channel crashed: {:?}", err);
                    }
                }
            }
        }
    });

    load_prefixes(tx).await.unwrap();
    let tree_bitmap = store_tree.await.unwrap();

    let ready = std::time::Instant::now();
    println!("prefix vec size {}", tree_bitmap.prefixes.len());
    println!(
        "finished building tree in {} msecs...",
        ready.checked_duration_since(start).unwrap().as_millis()
    );
    println!(
        "{:?} nodes created in total",
        tree_bitmap.stats.iter().fold(0, |mut acc, c| {
            acc += c.created_nodes.iter().fold(0, |mut sum, l| {
                sum += l.count;
                sum
            });
            acc
        })
    );
    println!(
        "stride division  {:?}",
        TreeBitMap::<u32, PrefixAs>::STRIDES
    );
    for s in &tree_bitmap.stats {
        println!("{:?}", s);
    }

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
}
