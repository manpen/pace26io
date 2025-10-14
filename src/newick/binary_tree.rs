use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub u32);

pub trait Buildable {
    type Node;
    fn new_inner(&mut self, left: Self::Node, right: Self::Node) -> Self::Node;
    fn new_leaf(&mut self, label: Label) -> Self::Node;
}

pub trait Constructable {
    /// Creates a new inner node with the two children provided.
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let root = BinTree::new_inner(
    ///     BinTree::new_leaf(Label(1)),
    ///     BinTree::new_leaf(Label(2))
    /// );
    ///
    /// assert!( root.top_down().is_inner());
    /// assert!(!root.top_down().is_leaf());
    /// ```
    fn new_inner(left: Self, right: Self) -> Self;

    /// Creates a new leaf node with the label provided.
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let leaf = BinTree::new_leaf(Label(42));
    /// assert!( leaf.top_down().is_leaf());
    /// assert!(!leaf.top_down().is_inner());
    /// ```
    fn new_leaf(label: Label) -> Self;
}

pub trait TopDownCursor: Sized {
    /// Returns the children iff self is an inner node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let leaf = BinTree::new_leaf(Label(1));
    /// let root = BinTree::new_inner(leaf.clone(), leaf.clone());
    ///
    /// assert!(leaf.top_down().children().is_none());
    /// assert!(root.top_down().children().is_some());
    /// assert!(root.top_down().children().unwrap().0.is_leaf());
    /// ```
    fn children(&self) -> Option<(Self, Self)>;

    /// Returns the left child iff self is an inner node and `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let left_leaf = BinTree::new_leaf(Label(3141));
    /// let right_leaf = BinTree::new_leaf(Label(1234));
    /// let root = BinTree::new_inner(left_leaf, right_leaf);
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
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let left_leaf = BinTree::new_leaf(Label(3141));
    /// let right_leaf = BinTree::new_leaf(Label(1234));
    /// let root = BinTree::new_inner(left_leaf, right_leaf);
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
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let leaf = BinTree::new_leaf(Label(1337));
    /// let root = BinTree::new_inner(leaf.clone(), leaf.clone());
    ///
    /// assert_eq!(leaf.top_down().leaf_label().unwrap(), Label(1337));
    /// assert!(   root.top_down().leaf_label().is_none());
    ///
    fn leaf_label(&self) -> Option<Label>;

    /// Returns true iff self is an inner node
    ///
    /// # Example
    /// ```
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let leaf = BinTree::new_leaf(Label(1));
    /// let root = BinTree::new_inner(leaf.clone(), leaf.clone());
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
    /// use pace26io::newick::binary_tree::*;
    ///
    /// let leaf = BinTree::new_leaf(Label(1));
    /// let root = BinTree::new_inner(leaf.clone(), leaf.clone());
    ///
    /// assert!(!root.top_down().is_leaf());
    /// assert!( leaf.top_down().is_leaf());
    /// ```
    fn is_leaf(&self) -> bool {
        self.leaf_label().is_some()
    }
}

/// Minimalistic implementation of a binary tree without any meta information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinTree {
    Node(Box<(BinTree, BinTree)>),
    Leaf(Label),
}

impl BinTree {
    pub fn top_down(&self) -> &Self {
        self
    }
}

impl Constructable for BinTree {
    fn new_inner(left: Self, right: Self) -> Self {
        BinTree::Node(Box::new((left, right)))
    }

    fn new_leaf(label: Label) -> Self {
        BinTree::Leaf(label)
    }
}

impl TopDownCursor for &BinTree {
    fn children(&self) -> Option<(Self, Self)> {
        match self {
            BinTree::Node(b) => Some((&b.as_ref().0, &b.as_ref().1)),
            BinTree::Leaf(_) => None,
        }
    }

    fn leaf_label(&self) -> Option<Label> {
        match self {
            BinTree::Leaf(l) => Some(*l),
            BinTree::Node(_) => None,
        }
    }
}

#[derive(Default)]
pub struct ConstrToBuilderAdapter<C: Constructable>(PhantomData<C>);

impl<C: Constructable> ConstrToBuilderAdapter<C> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<C: Constructable> Buildable for ConstrToBuilderAdapter<C> {
    type Node = C;

    fn new_inner(&mut self, left: Self::Node, right: Self::Node) -> Self::Node {
        C::new_inner(left, right)
    }

    fn new_leaf(&mut self, label: Label) -> Self::Node {
        C::new_leaf(label)
    }
}

pub type BinTreeBuilder = ConstrToBuilderAdapter<BinTree>;
