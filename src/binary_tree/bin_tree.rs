use super::*;

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
pub struct BinTreeBuilder();

impl TreeBuilder for BinTreeBuilder {
    type Node = BinTree;

    fn new_inner(&mut self, _id: NodeIdx, left: Self::Node, right: Self::Node) -> Self::Node {
        BinTree::Node(Box::new((left, right)))
    }

    fn new_leaf(&mut self, label: Label) -> Self::Node {
        BinTree::Leaf(label)
    }
}
