#![allow(unused_imports)]
mod test {
    use crate::common::{NoMeta, Prefix, PrefixAs};
    use crate::treebitmap_univec::TreeBitMap;
    use crate::common::TrieLevelStats;
    use ansi_term::Colour;
    use std::env;
    use std::error::Error;
    use std::ffi::OsString;
    use std::fs::File;
    use std::process;

    // use shrust::{Shell, ShellIO};
    // use std::io::prelude::*;

    #[test]
    fn test_csv() {
        const CSV_FILE_PATH: &str = "./data/uniq_pfx_asn_dfz_rnd.csv";

        fn load_prefixes(pfxs: &mut Vec<Prefix<u32, PrefixAs>>) -> Result<(), Box<dyn Error>> {
            let file = File::open(CSV_FILE_PATH)?;
            let mut rdr = csv::Reader::from_reader(file);
            for result in rdr.records() {
                let record = result?;
                let ip: Vec<_> = record[0]
                    .split(".")
                    .map(|o| -> u8 { o.parse().unwrap() })
                    .collect();
                let net = std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
                let len: u8 = record[1].parse().unwrap();
                let asn: u32 = record[2].parse().unwrap();
                let pfx = Prefix::<u32, PrefixAs>::new_with_meta(net.into(), len, PrefixAs(asn));
                pfxs.push(pfx);
            }
            Ok(())
        }

        let mut pfxs: Vec<Prefix<u32, PrefixAs>> = vec![];
        let mut tree_bitmap: TreeBitMap<u32, PrefixAs> = TreeBitMap::new();

        if let Err(err) = load_prefixes(&mut pfxs) {
            println!("error running example: {}", err);
            process::exit(1);
        }
        println!("finished loading {} prefixes...", pfxs.len());
        let start = std::time::Instant::now();

        let pfxs_len = pfxs.len();
        for pfx in pfxs.into_iter() {
            tree_bitmap.insert(pfx);
        }
        let ready = std::time::Instant::now();

        println!(
            "finished building tree in {} msecs...",
            ready.checked_duration_since(start).unwrap().as_millis()
        );
        println!(
            "{} inserts/sec",
            pfxs_len as f32 / ready.checked_duration_since(start).unwrap().as_secs_f32()
        );

        println!("prefix vec size {}", tree_bitmap.prefixes.len());

        println!("finished building tree...");

        println!(
            "stride division  {:?}",
            TreeBitMap::<u32, PrefixAs>::STRIDES
        );
        for s in &tree_bitmap.stats {
            println!("{:?}", s);
        }

        let inet_max = 255;
        let len_max = 32;

        let start = std::time::Instant::now();
        for i_net in 0..inet_max {
            for s_len in 0..len_max {
                for ii_net in 0..inet_max {
                    let pfx = Prefix::<u32, NoMeta>::new(
                        std::net::Ipv4Addr::new(i_net, ii_net, 0, 0).into(),
                        s_len,
                    );
                    tree_bitmap.match_longest_prefix_only(&pfx);
                }
            }
        }
        let ready = std::time::Instant::now();

        println!(
            "finished searching {} prefixes in {} seconds...",
            (inet_max as u16 * inet_max as u16 * len_max as u16),
            ready.checked_duration_since(start).unwrap().as_secs_f32()
        );
        println!(
            "1 lmp lookup takes {} nsec on average",
            ready.checked_duration_since(start).unwrap().as_nanos()
                / (inet_max as u128 * inet_max as u128 * len_max as u128)
        );
        println!(
            "{} lmp lookups/sec",
            (inet_max as u16 * inet_max as u16 * len_max as u16) as f32
                / ready.checked_duration_since(start).unwrap().as_secs_f32()
        );
    }
}
