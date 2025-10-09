use std::io::Write;

pub trait NewickWriter {
    /// Produces minimal Newick representation of a binary without any whitespace characters
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::*;
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
    /// use pace26io::newick::*;
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
