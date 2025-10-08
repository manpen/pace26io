use super::binary_tree::*;
use std::io::Write;

pub trait BinaryTreeWriter {
    /// Produces minimal Newick representation of a binary without any whitespace characters
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::{binary_tree::*, binary_tree_writer::*};
    ///
    /// let tree = BinTree::new_inner(
    ///         BinTree::new_leaf(Label(1)),
    ///         BinTree::new_leaf(Label(2))
    /// );
    ///
    /// let mut buffer : Vec<u8> = Vec::new();
    /// tree.write_newick(&mut buffer).unwrap();
    /// assert_eq!(String::from_utf8(buffer).unwrap(), "(1,2);");
    /// ```
    fn write_newick(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.write_newick_inner(writer)?;
        write!(writer, ";")
    }

    /// Produces a Newick string representation of self by calling [BinaryTreeWriter::write_newick]
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::{binary_tree::*, binary_tree_writer::*};
    ///
    /// let tree = BinTree::new_inner(
    ///         BinTree::new_leaf(Label(2)),
    ///         BinTree::new_leaf(Label(3))
    /// );
    ///
    /// assert_eq!(tree.to_newick_string(), "(2,3);");
    /// ```
    fn to_newick_string(&self) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_newick(&mut buffer)
            .expect("The writer should not fail");
        String::from_utf8(buffer).expect("The writer should not produce invalid strings")
    }

    /// Produces minimal Newick representation of a binary without any whitespace characters
    /// Same as [BinaryTreeWriter::write_newick], but omits the finishing semicolon.
    fn write_newick_inner(&self, writer: &mut impl Write) -> std::io::Result<()>;
}

impl<B: BinaryTreeNode> BinaryTreeWriter for B {
    fn write_newick_inner(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((left, right)) = self.children() {
            write!(writer, "(")?;
            left.write_newick_inner(writer)?;
            write!(writer, ",")?;
            right.write_newick_inner(writer)?;
            write!(writer, ")")
        } else if let Some(Label(label)) = self.leaf_label() {
            write!(writer, "{}", label)
        } else {
            unreachable!("Nodes must either have children or a label; this one has neither");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::newick::{
        binary_tree::{BinTree, BinaryTreeNode, Label},
        binary_tree_writer::BinaryTreeWriter,
    };

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
