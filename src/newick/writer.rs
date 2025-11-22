use std::io::Write;

pub trait NewickWriter {
    /// Produces minimal Newick representation of a binary without any whitespace characters
    ///
    /// # Example
    /// ```
    /// use pace26io::{binary_tree::*, newick::*};
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let l1 = builder.new_leaf(Label(1));
    /// let l2 = builder.new_leaf(Label(2));
    /// let tree = builder.new_inner(NodeIdx::new(3), l1, l2);
    ///
    /// let mut buffer : Vec<u8> = Vec::new();
    /// tree.top_down().write_newick(&mut buffer).unwrap();
    /// assert_eq!(String::from_utf8(buffer).unwrap(), "(1,2);");
    /// ```
    fn write_newick(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.write_newick_inner(writer)?;
        write!(writer, ";")
    }

    /// Produces a Newick string representation of self by calling [NewickWriter::write_newick]
    ///
    /// # Example
    /// ```
    /// use pace26io::{binary_tree::*, newick::*};
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let l1 = builder.new_leaf(Label(2));
    /// let l2 = builder.new_leaf(Label(3));
    /// let tree = builder.new_inner(NodeIdx::new(0), l1, l2);
    ///
    /// assert_eq!(tree.top_down().to_newick_string(), "(2,3);");
    /// ```
    fn to_newick_string(&self) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_newick(&mut buffer)
            .expect("The writer should not fail");
        String::from_utf8(buffer).expect("The writer should not produce invalid strings")
    }

    /// Produces minimal Newick representation of a binary without any whitespace characters
    /// Same as [NewickWriter::write_newick], but omits the finishing semicolon.
    fn write_newick_inner(&self, writer: &mut impl Write) -> std::io::Result<()>;
}
