# try-tries-and-trees

This is the ~~experimental~~ research Tries and Trees IP Lookup repository.

Currently we have for you:

- A simple binary Trie. It can be run by invoking `cargo run --release --example simpletrie`, 
`cargo run --release --example trie4` for a IPv4 trie and `cargo run --release --example trie6`.
- A Treebitmap, can be run likewise: `cargo run --release --example bittreemap`.

There are also two very crude REPL examples, that take csv files as input:
- `cargo run --release --example load_csv_treebitmap -- ./data/uniq_pfx_asn.csv` for the Treebitmap and
- `cargo run --release --example load_csv -- ./data/uniq_pfx_asn.csv` for the simple trie.

The `./data/uniq_pfx_asn.csv` is derived from a RisWHOIS file and thus approximates a full table.

On the REPL the only thing you can do is:
`s <PREFIX/LEN>`

## Treebitmap

The new() constructor of both treebitmaps takes an vec with strides. You can specify a full stride vec, like so:
`vec![6, 6, 6, 4, 4]` or by specifying a vec like so `vec![8]`. It will fill up with 8 untill it reaches the requested number of bits (32 for IPv3, 128 for IPv6).

Possible stride sizes are 3,4,5,6,7,8

There are two treebitmaps, one that stores prefixes internally in the tree (a vec per node) and a treebitmap that stores it in a global Vec<Prefix>.

## Benchmarks

ex.:
```cargo test --release tests::csv_test_treebitmap::test -- --show-output --test-threads=1```

Do not forget ``---release``, otherwise it will panic, because the timer will overflow!

Enjoy.
