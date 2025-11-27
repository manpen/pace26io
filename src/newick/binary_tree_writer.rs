use super::{super::binary_tree::*, *};
use std::io::Write;

impl<B: TopDownCursor> NewickWriter for B {
    fn write_newick_inner(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self.visit() {
            NodeType::Inner(left, right) => {
                write!(writer, "(")?;
                left.write_newick_inner(writer)?;
                write!(writer, ",")?;
                right.write_newick_inner(writer)?;
                write!(writer, ")")
            }
            NodeType::Leaf(Label(label)) => {
                write!(writer, "{label}")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn to_string(tree: BinTree) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        tree.top_down().write_newick(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn leaf() {
        let tree = BinTreeBuilder::default().new_leaf(Label(1234));
        assert_eq!(to_string(tree), "1234;");
    }

    #[test]
    fn pair() {
        let mut build = BinTreeBuilder::default();

        let l1 = build.new_leaf(Label(1234));
        let l2 = build.new_leaf(Label(5678));
        let tree = build.new_inner(NodeIdx::new(0), l1, l2);

        assert_eq!(to_string(tree), "(1234,5678);");
    }
}
