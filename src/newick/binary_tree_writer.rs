use super::{binary_tree::*, *};
use std::io::Write;

impl<B: BinaryTreeTopDown> NewickWriter for B {
    fn write_newick_inner(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((left, right)) = self.children() {
            write!(writer, "(")?;
            left.write_newick_inner(writer)?;
            write!(writer, ",")?;
            right.write_newick_inner(writer)?;
            write!(writer, ")")
        } else if let Some(Label(label)) = self.leaf_label() {
            write!(writer, "{label}")
        } else {
            unreachable!("Nodes must either have children or a label; this one has neither");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::newick::{binary_tree::*, binary_tree_writer::NewickWriter};

    fn to_string(tree: BinTree) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        tree.write_newick(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn leaf() {
        assert_eq!(to_string(BinTree::new_leaf(Label(1234))), "1234;");
    }

    #[test]
    fn pair() {
        assert_eq!(
            to_string(BinTree::new_inner(
                BinTree::new_leaf(Label(1234)),
                BinTree::new_leaf(Label(5678))
            )),
            "(1234,5678);"
        );
    }
}
