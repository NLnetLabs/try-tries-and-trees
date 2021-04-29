use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;
use trie::common::{Prefix, PrefixAs};
use trie::treebitmap_univec::TreeBitMap;

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
    let strides_vec = [
        vec![8],
        vec![4],
        vec![6, 6, 6, 6, 4, 4],
        vec![3, 4, 4, 6, 7, 8],
    ];

    println!("[");
    for strides in strides_vec.iter().enumerate() {
        let mut pfxs: Vec<Prefix<u32, PrefixAs>> = vec![];
        let mut tree_bitmap: TreeBitMap<u32, PrefixAs> = TreeBitMap::new(strides.1.to_owned());

        if let Err(err) = load_prefixes(&mut pfxs) {
            println!("error running example: {}", err);
            process::exit(1);
        }

        for pfx in pfxs.into_iter() {
            tree_bitmap.insert(pfx);
        }

        let total_nodes = tree_bitmap.stats.iter().fold(0, |mut acc, c| {
            acc += c.created_nodes.iter().fold(0, |mut sum, l| {
                sum += l.count;
                sum
            });
            acc
        });

        println!("{{");
        println!("\"total_nodes\": {},", total_nodes);
        println!(
            "\"node_size_b\": {},",
            std::mem::size_of::<trie::treebitmap_univec::SizedStrideNode<u32>>()
        );
        println!(
            "\"nodes_mem_kb\": {},",
            (total_nodes * std::mem::size_of::<trie::treebitmap_univec::SizedStrideNode<u32>>()
                + tree_bitmap.prefixes.len() * 5
                + total_nodes * 5)
                / 1024 // 5 is the size of a (u32, u8)
        );
        println!("\"total_prefixes\": {:?},", tree_bitmap.prefixes.len());
        println!(
            "\"prefixes_mem_kb\": {:?},",
            tree_bitmap.prefixes.len() * std::mem::size_of::<Prefix<u32, PrefixAs>>() / 1024
        );
        println!(
            "\"prefixes_per_node\": {},",
            tree_bitmap.prefixes.len() as f64 / total_nodes as f64
        );
        println!("\"strides\": {:?},", tree_bitmap.strides);

        println!("\"levels\": [");

        let mut stride_bits = [0, 0];

        for stride in tree_bitmap.strides.iter().enumerate() {
            // let level = stride.0;
            stride_bits = [stride_bits[1] + 1, stride_bits[1] + stride.1];
            let nodes_num = tree_bitmap
                .stats
                .iter()
                .find(|s| s.stride_len == *stride.1)
                .unwrap()
                .created_nodes[stride.0]
                .count as u32;
            let prefixes_num = tree_bitmap
                .stats
                .iter()
                .find(|s| s.stride_len == *stride.1)
                .unwrap()
                .prefixes_num[stride.0]
                .count as u32;

            let max_pfx: u64 = u64::pow(2, stride_bits[1] as u32);

            println!("{{");
            println!("\"level\":{},", stride.0);
            println!(
                "\"bit_start\": {}, \"bit_stop\": {},",
                stride_bits[0], stride_bits[1]
            );
            println!("\"nodes_num\":{},", nodes_num);
            println!("\"prefixes_num\":{},", prefixes_num);
            println!("\"max_prefixes_num\":{}", max_pfx);
            println!(
                "}}{}",
                if stride.0 != (tree_bitmap.strides.len() - 1) {
                    ","
                } else {
                    ""
                }
            );
        }
        println!(
            "]}}{}",
            if strides.0 != strides_vec.len() - 1 {
                ","
            } else {
                ""
            }
        );
    }
    println!("]");
}
