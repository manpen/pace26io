# PACE 2026 I/O Crate

This crate implements parsers and writers for the PACE 2026 file format. It was originally 
developped for the official PACE tools (e.g., verifier and stride). As such, it offers a
great deal of flexibility including quite pedantic parsing modes. Most users should stay
away from this mess and rather use the simplified reader interface:

## Simplified reader interface 

We offer a simplified interface in [`pace::simplified::Instance`] intend to be used by solver
implementers. To read an instance, you may use:

```rust
use std::{fs::File, io::BufReader};
use pace26io::{binary_tree::*, newick::NewickWriter, pace::simplified::*};

type Builder = IndexedBinTreeBuilder; // If you do not care about inner node indices, use BinTreeBuilder
type Node = <Builder as TreeBuilder>::Node;

// A solver would typically use `std::io::stdin().lock()` instead of reading a file
let mut input = BufReader::new(File::open("examples/tiny01.nw").unwrap());

// Parse instance
let mut tree_builder = Builder::default();
let instance = Instance::try_read(&mut input, &mut tree_builder)
    .expect("Valid PACE26 Instance");

println!("# Found {} trees", instance.trees.len());
```

This interface will ignore most parser warnings and only report raise errors if parsing cannot continue. We recommend the `stride` tool to debug broken instances.

## Tree representation

We offer only rudamentary tree representations, more specifically 
[`binary_tree::BinTree`] and [`binary_tree::IndexedBinTree`]. The latter also stores 
node ids of internal nodes, which are for instance used by graph parameters. 

We expect that solvers will typically need more control over their data structures. 
For this reason, the crate is designed to make implementation of own tree structures straight forward.
You need to provide 
 - A node type which respresents both inner nodes and leafes. It needs to implement [`binary_tree::TopDownCursor`] and --if applicable-- [`binary_tree::TreeWithNodeIdx`].
 - A struct implementing [`binary_tree::TreeBuilder`].

## Writing Newick strings

A Newick String writer is provided for each data structure implementing [`binary_tree::TopDownCursor`].
For further details see [`newick::NewickWriter`].
