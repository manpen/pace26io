/// This example reads in an instance, orders the trees, the children of each
/// inner node, such that the left child always contains the smallest leaf label.
///
/// To execute it, run `cat examples/tiny01.nw | cargo --example normalize`
use pace26io::{binary_tree::*, newick::NewickWriter, pace::simplified::*};

type Builder = IndexedBinTreeBuilder; // If you do not care about inner node indices, use BinTreeBuilder
type Node = <Builder as TreeBuilder>::Node;

fn main() {
    let mut tree_builder = Builder::default();
    let instance = Instance::try_read(&mut std::io::stdin().lock(), &mut tree_builder)
        .expect("Valid PACE26 Instance");

    println!("# Found {} trees", instance.trees.len());
    if let Some(td) = instance.tree_decomposition.as_ref() {
        println!(
            "# Found tree decomposition with treewidth {}, {} bags, and {} edges",
            td.treewidth,
            td.bags.len(),
            td.edges.len()
        );
    }

    for (tree_id, tree) in instance.trees.iter().enumerate() {
        let root_id = (tree_id + 1) * (instance.num_leaves - 1) + 2;
        let normalized_tree =
            build_normalized_tree(&mut tree_builder, tree, NodeIdx(root_id as u32));

        println!("{}", normalized_tree.top_down().to_newick_string());
    }
}

fn build_normalized_tree(
    builder: &mut Builder,
    node: impl TopDownCursor,
    node_id: NodeIdx,
) -> Node {
    let root = build_normalized_tree_rec(builder, node, node_id).0;
    builder.make_root(root)
}

fn build_normalized_tree_rec(
    builder: &mut Builder,
    node: impl TopDownCursor,
    node_id: NodeIdx,
) -> (Node, Label, NodeIdx) {
    match node.visit() {
        // Base case: For a leaf with simply copy the label and build a new leaf node
        NodeType::Leaf(label) => (builder.new_leaf(label), label, node_id),

        // Recursion into subtrees:
        NodeType::Inner(left, right) => {
            // recursively decent into both subtrees
            let (child0, label0, next_node_id) =
                build_normalized_tree_rec(builder, left, node_id.incremented());
            let (child1, label1, next_node_id) =
                build_normalized_tree_rec(builder, right, next_node_id);

            // construct a new inner node with the smaller subtree to the left
            if label0 < label1 {
                (
                    builder.new_inner(node_id, child0, child1),
                    label0,
                    next_node_id,
                )
            } else {
                (
                    builder.new_inner(node_id, child1, child0),
                    label1,
                    next_node_id,
                )
            }
        }
    }
}
