use crate::{
    binary_tree::{NodeIdx, TreeBuilder},
    newick::{BinaryTreeParser, ParserError},
    pace::{
        parameters::tree_decomposition::TreeDecomposition,
        reader::{Action, InstanceReader, InstanceVisitor, ReaderError},
    },
};
use std::io::BufRead;

use thiserror::Error;

/// Simplified interface to read PACE26 instances and extract all information
/// relevant for solvers. The interface is generic in [`TreeBuilder`]
/// allowing the calling code to specify its own binary tree implementation.
/// The only explicitly implemented function is [`Instance::try_read`]; if successful,
/// access retrieved data directly from the struct.
/// Future versions of this crate may add more instance parameters.
#[derive(Debug, Clone)]
pub struct Instance<B: TreeBuilder> {
    pub num_leaves: usize,
    pub trees: Vec<B::Node>,
    pub tree_decomposition: Option<TreeDecomposition>,
}

impl<B: TreeBuilder> Instance<B> {
    pub fn try_read(
        reader: impl BufRead,
        tree_builder: &mut B,
    ) -> Result<Self, SimplifiedReaderError> {
        let mut instance = Instance {
            num_leaves: 0,
            trees: Vec::with_capacity(2),
            tree_decomposition: None,
        };

        let mut visitor = Visitor {
            builder: tree_builder,
            instance: &mut instance,
            num_leaves: None,
            error: None,
        };

        let mut instance_reader = InstanceReader::new(&mut visitor);
        instance_reader.read(reader)?;

        if let Some(err) = visitor.error {
            return Err(err);
        }

        Ok(instance)
    }
}

struct Visitor<'a, B: TreeBuilder> {
    builder: &'a mut B,
    instance: &'a mut Instance<B>,
    num_leaves: Option<usize>,
    error: Option<SimplifiedReaderError>,
}

impl<'a, B: TreeBuilder> InstanceVisitor for Visitor<'a, B> {
    fn visit_header(
        &mut self,
        _lineno: usize,
        _num_trees: usize,
        num_leaves: usize,
    ) -> super::reader::Action {
        if self.num_leaves.is_some() {
            self.error = Some(SimplifiedReaderError::MultipleHeaders);
            return Action::Terminate;
        }

        if num_leaves == 0 {
            self.error = Some(SimplifiedReaderError::NoLeaves);
            return Action::Terminate;
        }

        self.num_leaves = Some(num_leaves);
        self.instance.num_leaves = num_leaves;
        Action::Continue
    }

    fn visit_tree(&mut self, _lineno: usize, line: &str) -> super::reader::Action {
        let num_leaves = match self.num_leaves {
            Some(x) => x,
            None => {
                self.error = Some(SimplifiedReaderError::NoHeader);
                return Action::Terminate;
            }
        };

        let root_id = (self.instance.trees.len() + 1) * (num_leaves - 1) + 2;

        let tree = match self
            .builder
            .parse_newick_from_str(line, NodeIdx(root_id as u32))
        {
            Ok(t) => t,
            Err(e) => {
                self.error = Some(SimplifiedReaderError::NewickError(e));
                return Action::Terminate;
            }
        };

        self.instance.trees.push(tree);

        super::reader::Action::Continue
    }

    const VISIT_PARAM_TREE_DECOMPOSITION: bool = true;
    fn visit_param_tree_decomposition(
        &mut self,
        _lineno: usize,
        td: TreeDecomposition,
    ) -> super::reader::Action {
        self.instance.tree_decomposition = Some(td);
        super::reader::Action::Continue
    }
}

#[derive(Debug, Error)]
pub enum SimplifiedReaderError {
    #[error(transparent)]
    ReaderError(#[from] ReaderError),

    #[error(transparent)]
    NewickError(#[from] ParserError),

    #[error(transparent)]
    JSONError(#[from] serde_json::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Multiple headers found")]
    MultipleHeaders,

    #[error("Header indicates no leaves")]
    NoLeaves,

    #[error("No header before first tree")]
    NoHeader,
}

#[cfg(test)]
mod test {
    use crate::binary_tree::IndexedBinTreeBuilder;

    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn read_tiny() {
        let mut input = BufReader::new(File::open("examples/tiny01.nw").unwrap());

        // Parse instance
        let mut tree_builder = IndexedBinTreeBuilder::default();
        let instance =
            Instance::try_read(&mut input, &mut tree_builder).expect("Valid PACE26 Instance");

        assert_eq!(instance.num_leaves, 6);
        assert_eq!(instance.trees.len(), 2);
        assert_eq!(instance.tree_decomposition.unwrap().treewidth, 2);
    }
}
