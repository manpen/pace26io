use super::*;

pub trait DepthFirstSearch {
    fn dfs(self) -> impl Iterator<Item = Self>;
}

pub struct DFSImpl<C> {
    stack: Vec<C>,
}

impl<C: TopDownCursor> DepthFirstSearch for C {
    fn dfs(self) -> impl Iterator<Item = Self> {
        DFSImpl { stack: vec![self] }
    }
}

impl<C: TopDownCursor> Iterator for DFSImpl<C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stack.pop()?;

        if let Some((left, right)) = item.children() {
            self.stack.push(right);
            self.stack.push(left);
        }

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newick::BinaryTreeParser;

    #[test]
    fn dfs() {
        let tree = BinTreeBuilder::default()
            .parse_newick_from_str("((3,1),2);", NodeIdx::new(0))
            .unwrap();
        let mut trav = tree.dfs();

        assert!(trav.next().unwrap().is_inner());
        assert!(trav.next().unwrap().is_inner());
        assert_eq!(trav.next().unwrap().leaf_label(), Some(Label(3)));
        assert_eq!(trav.next().unwrap().leaf_label(), Some(Label(1)));
        assert_eq!(trav.next().unwrap().leaf_label(), Some(Label(2)));
        assert!(trav.next().is_none());
        assert!(trav.next().is_none());
    }
}
