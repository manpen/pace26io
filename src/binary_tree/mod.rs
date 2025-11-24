pub mod bin_tree;
pub use bin_tree::*;
pub mod indexed_bin_tree;
pub use indexed_bin_tree::*;

pub mod depth_first_search;
pub use depth_first_search::DepthFirstSearch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NodeIdx(pub u32);

impl NodeIdx {
    pub fn new(v: u32) -> Self {
        NodeIdx(v)
    }

    pub fn incremented(self) -> Self {
        NodeIdx(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub u32);

impl From<Label> for NodeIdx {
    fn from(value: Label) -> Self {
        NodeIdx(value.0)
    }
}

pub trait TreeBuilder {
    type Node;

    /// Creates a new inner node with the two children provided.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let l1 = builder.new_leaf(Label(1));
    /// let l2 = builder.new_leaf(Label(2));
    /// let root = builder.new_inner(NodeIdx::new(0), l1,l2);
    ///
    /// assert!( root.top_down().is_inner());
    /// assert!(!root.top_down().is_leaf());
    /// ```
    fn new_inner(&mut self, id: NodeIdx, left: Self::Node, right: Self::Node) -> Self::Node;

    /// Creates a new leaf node with the label provided.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let leaf = builder.new_leaf(Label(42));
    /// assert!( leaf.top_down().is_leaf());
    /// assert!(!leaf.top_down().is_inner());
    /// ```    
    fn new_leaf(&mut self, label: Label) -> Self::Node;

    /// Declares a node a root. Depending on the tree
    /// implementation this may be a no-op, or may trigger
    /// the computation of meta information.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let l1 = builder.new_leaf(Label(1));
    /// let l2 = builder.new_leaf(Label(2));
    /// let root = builder.new_inner(NodeIdx::new(0),l1,l2);
    /// let root = builder.make_root(root);
    ///
    /// assert!( root.top_down().is_inner());
    /// assert!(!root.top_down().is_leaf());
    /// ```
    fn make_root(&mut self, root: Self::Node) -> Self::Node {
        root
    }
}

pub trait TopDownCursor: Sized {
    /// Returns the children iff self is an inner node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let l1 = builder.new_leaf(Label(1));
    /// assert!(l1.top_down().children().is_none());
    ///
    /// let l2 = builder.new_leaf(Label(2));
    /// let root = builder.new_inner(NodeIdx::new(0), l1, l2);
    ///
    /// assert!(root.top_down().children().is_some());
    /// assert!(root.top_down().children().unwrap().0.is_leaf());
    /// ```
    fn children(&self) -> Option<(Self, Self)>;

    /// Returns the left child iff self is an inner node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let left_leaf = builder.new_leaf(Label(3141));
    /// let right_leaf = builder.new_leaf(Label(1234));
    /// let root = builder.new_inner(NodeIdx::new(0), left_leaf, right_leaf);
    ///
    /// assert_eq!(root.top_down().left_child().unwrap().leaf_label(), Some(Label(3141)));
    /// ```
    fn left_child(&self) -> Option<Self> {
        self.children().map(|(l, _)| l)
    }

    /// Returns the right child iff self is an inner node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let left_leaf = builder.new_leaf(Label(3141));
    /// let right_leaf = builder.new_leaf(Label(1234));
    /// let root = builder.new_inner(NodeIdx::new(0), left_leaf, right_leaf);
    ///
    /// assert_eq!(root.top_down().right_child().unwrap().leaf_label(), Some(Label(1234)));
    /// ```
    fn right_child(&self) -> Option<Self> {
        self.children().map(|(_, r)| r)
    }

    /// Returns the label iff self is a leaf node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let leaf = builder.new_leaf(Label(1337));
    /// let root = builder.new_inner(NodeIdx::new(0), leaf.clone(), leaf.clone());
    ///
    /// assert_eq!(leaf.top_down().leaf_label().unwrap(), Label(1337));
    /// assert!(   root.top_down().leaf_label().is_none());
    ///
    fn leaf_label(&self) -> Option<Label>;

    /// Returns true iff self is an inner node
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let leaf = builder.new_leaf(Label(1));
    /// let root = builder.new_inner(NodeIdx::new(0), leaf.clone(), leaf.clone());
    ///
    /// assert!( root.top_down().is_inner());
    /// assert!(!leaf.top_down().is_inner());
    /// ```
    fn is_inner(&self) -> bool {
        !self.is_leaf()
    }

    /// Returns true iff self is a leaf
    ///
    /// # Example
    /// ```
    /// use pace26io::binary_tree::*;
    ///
    /// let mut builder = BinTreeBuilder::default();
    /// let leaf = builder.new_leaf(Label(1));
    /// let root = builder.new_inner(NodeIdx::new(0), leaf.clone(), leaf.clone());
    ///
    /// assert!(!root.top_down().is_leaf());
    /// assert!( leaf.top_down().is_leaf());
    /// ```
    fn is_leaf(&self) -> bool {
        self.leaf_label().is_some()
    }
}

/// Tree with indexed inner nodes
pub trait TreeWithNodeIdx {
    /// Returns the index of the node. If the node is a leaf,
    /// the leaf label is converted into a node index.
    fn node_idx(&self) -> NodeIdx;
}
