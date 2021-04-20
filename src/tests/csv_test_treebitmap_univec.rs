#![allow(unused_imports)]
mod test {
    use crate::common::TrieLevelStats;
    use crate::common::{NoMeta, Prefix, PrefixAs};
    use crate::treebitmap_univec::TreeBitMap;
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

        println!("[");
        let strides_vec = [
            vec![8],
            vec![4],
            vec![6, 6, 6, 6, 4, 4],
            vec![3, 4, 4, 6, 7, 8],
        ];
        for strides in strides_vec.iter().enumerate() {
            println!("[");
            for n in 1..6 {
                let mut pfxs: Vec<Prefix<u32, PrefixAs>> = vec![];
                let mut tree_bitmap: TreeBitMap<u32, PrefixAs> =
                    TreeBitMap::new(strides.1.to_owned());

                if let Err(err) = load_prefixes(&mut pfxs) {
                    println!("error running example: {}", err);
                    process::exit(1);
                }
                // println!("finished loading {} prefixes...", pfxs.len());
                let start = std::time::Instant::now();

                let inserts_num = pfxs.len();
                for pfx in pfxs.into_iter() {
                    tree_bitmap.insert(pfx);
                }
                let ready = std::time::Instant::now();
                let dur_insert_nanos = ready.checked_duration_since(start).unwrap().as_nanos();

                // println!(
                //     "finished building tree in {} msecs...",
                //     ready.checked_duration_since(start).unwrap().as_millis()
                // );
                // println!(
                //     "{} inserts/sec",
                //     inserts_num as f32 / ready.checked_duration_since(start).unwrap().as_secs_f32()
                // );

                // println!("prefix vec size {}", tree_bitmap.prefixes.len());

                // println!("finished building tree...");

                // println!(
                //     "stride division  {:?}",
                //     TreeBitMap::<u32, PrefixAs>::STRIDES
                // );
                // for s in &tree_bitmap.stats {
                //     println!("{:?}", s);
                // }

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
                let dur_search_nanos = ready.checked_duration_since(start).unwrap().as_nanos();
                let searches_num = inet_max as u128 * inet_max as u128 * len_max as u128;

                println!("{{");
                println!("\"type\": \"treebitmap_univec\",");
                println!("\"strides\": {:?},", tree_bitmap.strides);
                println!("\"run_no\": {},", n);
                println!("\"inserts_num\": {},", inserts_num);
                println!("\"insert_duration_nanos\": {},", dur_insert_nanos);
                println!(
                    "\"global_prefix_vec_size\": {},",
                    tree_bitmap.prefixes.len()
                );
                println!("\"global_node_vec_size\": {},", tree_bitmap.nodes.len());
                println!(
                    "\"insert_time_nanos\": {},",
                    dur_insert_nanos as f32 / inserts_num as f32
                );
                println!("\"searches_num\": {},", searches_num);
                println!("\"search_duration_nanos\": {},", dur_search_nanos);
                println!(
                    "\"search_time_nanos\": {}",
                    dur_search_nanos as f32 / searches_num as f32
                );
                println!("}}{}", if n != 5 { "," } else { "" });
            }

            println!(
                "]{}",
                if strides.0 != strides_vec.len() - 1 {
                    ","
                } else {
                    ""
                }
            );
        }
        println!("]");
    }
}
