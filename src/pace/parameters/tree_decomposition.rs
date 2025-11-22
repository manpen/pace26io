use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

type Node = u32;
type NumNodes = Node;

/// Container to store the `treedecomp` parameter.
/// Recall that all indices are 1-index, i.e. the edge `(u,v)` refers to `bags[u-1]` and `bags[v-1]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDecomposition {
    pub treewidth: NumNodes,
    pub bags: Vec<Vec<Node>>,
    pub edges: Vec<(Node, Node)>,
}

impl<'de> Deserialize<'de> for TreeDecomposition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TreeDecompositionVisitor;

        impl<'de> Visitor<'de> for TreeDecompositionVisitor {
            type Value = TreeDecomposition;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of three elements: treewidth, bags, edges")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<TreeDecomposition, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let treewidth: NumNodes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let bags: Vec<Vec<Node>> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let edges: Vec<(Node, Node)> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                Ok(TreeDecomposition {
                    treewidth,
                    bags,
                    edges,
                })
            }
        }

        deserializer.deserialize_seq(TreeDecompositionVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::pace::parameters::tree_decomposition::TreeDecomposition;

    #[test]
    fn json_to_tree_decomp() {
        const JSON: &str = "[2,[[8,16],[8,11,16],[1,11,15],[2,11,16],[7,8,11],[8,10,16],[3,10,13],[4,10,16],[8,9],[5,9,14],[6,9,12]],[[1,2],[1,6],[1,9],[2,3],[2,4],[2,5],[6,7],[6,8],[9,10],[9,11]]]";
        let td: TreeDecomposition = serde_json::from_str(JSON).unwrap();

        assert_eq!(td.treewidth, 2);
        assert_eq!(td.bags.len(), 11);
        assert_eq!(td.edges.len(), 10);
    }
}
