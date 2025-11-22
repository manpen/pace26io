use super::*;

/// Minimalistic implementation of a binary tree without any meta information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndexedBinTree {
    Node(Box<(NodeIdx, IndexedBinTree, IndexedBinTree)>),
    Leaf(Label),
}

impl IndexedBinTree {
    pub fn top_down(&self) -> &Self {
        self
    }
}

impl TopDownCursor for &IndexedBinTree {
    fn children(&self) -> Option<(Self, Self)> {
        match self {
            IndexedBinTree::Node(b) => Some((&b.as_ref().1, &b.as_ref().2)),
            IndexedBinTree::Leaf(_) => None,
        }
    }

    fn leaf_label(&self) -> Option<Label> {
        match self {
            IndexedBinTree::Leaf(l) => Some(*l),
            IndexedBinTree::Node(_) => None,
        }
    }
}

impl TreeWithNodeIdx for IndexedBinTree {
    fn node_idx(&self) -> NodeIdx {
        match self {
            IndexedBinTree::Node(b) => b.0,
            IndexedBinTree::Leaf(label) => (*label).into(),
        }
    }
}

#[derive(Default)]
pub struct IndexedBinTreeBuilder();

impl TreeBuilder for IndexedBinTreeBuilder {
    type Node = IndexedBinTree;

    fn new_inner(&mut self, idx: NodeIdx, left: Self::Node, right: Self::Node) -> Self::Node {
        IndexedBinTree::Node(Box::new((idx, left, right)))
    }

    fn new_leaf(&mut self, label: Label) -> Self::Node {
        IndexedBinTree::Leaf(label)
    }
}
