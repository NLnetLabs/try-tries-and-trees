# try-tries-and-trees

This is the experimental Tries and Trees IP Lookup repository.

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

You can change the strides in by changing this line in `src/treebitmap/mod.rs`:
`const STRIDES: [u8; 6] = [6, 6, 6, 6, 5, 3];`

Possible stride sizes are 3,4,5,6.

Enjoy.
