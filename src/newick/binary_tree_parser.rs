use thiserror::Error;

use super::{super::binary_tree::*, lexer::*};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParserError {
    #[error("unexpected end of token stream")]
    UnexpectedEnd,

    #[error("Expected begin of node definition, i.e. label or opening parenthesis. Got: {token:?}")]
    ExpectedNodeBegin { token: Token },

    #[error("Expected comma. Got: {token:?}")]
    ExpectedComma { token: Token },

    #[error("Expected closing parenthesis. Got {token:?}")]
    ExpectedClosing { token: Token },

    #[error("Expected end of expression, i.e. ';'. Got: {token:?}")]
    ExpectedEnd { token: Token },

    #[error(transparent)]
    Lexer(#[from] LexerError),
}

pub trait BinaryTreeParser: TreeBuilder + Sized {
    fn parse_newick_from_lexer(
        &mut self,
        lexer: &mut Lexer,
        root_id: NodeIdx,
    ) -> Result<Self::Node, ParserError>;

    fn parse_newick_from_str(
        &mut self,
        text: &str,
        root_id: NodeIdx,
    ) -> Result<Self::Node, ParserError> {
        let mut lexer = Lexer::new(text);
        self.parse_newick_from_lexer(&mut lexer, root_id)
    }
}

fn assert_next_token_else(
    lexer: &mut Lexer,
    expected: TokenType,
    error: impl FnOnce(Token) -> ParserError,
) -> Result<(), ParserError> {
    let token = lexer.next().ok_or(ParserError::UnexpectedEnd)??;
    if token.token_type == expected {
        Ok(())
    } else {
        Err(error(token))
    }
}

fn parse_inner<B: TreeBuilder>(
    builder: &mut B,
    lexer: &mut Lexer,
    own_id: NodeIdx,
) -> Result<(B::Node, NodeIdx), ParserError> {
    let token = lexer.next().ok_or(ParserError::UnexpectedEnd)??;

    match token.token_type {
        TokenType::ParOpen => {
            let (left_child, next_id) = parse_inner(builder, lexer, own_id.incremented())?;

            assert_next_token_else(lexer, TokenType::Comma, |token| {
                ParserError::ExpectedComma { token }
            })?;

            let (right_child, next_id) = parse_inner(builder, lexer, next_id)?;

            assert_next_token_else(lexer, TokenType::ParClose, |token| {
                ParserError::ExpectedClosing { token }
            })?;

            Ok((builder.new_inner(own_id, left_child, right_child), next_id))
        }

        TokenType::Number(x) => Ok((builder.new_leaf(Label(x)), own_id)),
        _ => Err(ParserError::ExpectedNodeBegin { token }),
    }
}

impl<B: TreeBuilder> BinaryTreeParser for B {
    fn parse_newick_from_lexer(
        &mut self,
        lexer: &mut Lexer,
        root_id: NodeIdx,
    ) -> Result<Self::Node, ParserError> {
        let (tree, _) = parse_inner(self, lexer, root_id)?;

        assert_next_token_else(lexer, TokenType::Semicolon, |token| {
            ParserError::ExpectedEnd { token }
        })?;

        Ok(self.make_root(tree))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::newick::*;

    fn navigate<T: TopDownCursor>(mut cursor: T, path: &str) -> Option<T> {
        for x in path.chars() {
            match x {
                'l' => {
                    cursor = cursor.left_child()?;
                }

                'r' => {
                    cursor = cursor.right_child()?;
                }

                _ => panic!("Unknown char"),
            }
        }

        Some(cursor)
    }

    #[test]
    fn leaf() {
        let tree = BinTreeBuilder::default()
            .parse_newick_from_str("132;", NodeIdx::new(0))
            .unwrap();
        assert_eq!(tree.top_down().leaf_label(), Some(Label(132)));
    }

    macro_rules! parser_error_test {
        ($ident:ident, $text:expr, $expect:pat) => {
            #[test]
            fn $ident() {
                let result = BinTreeBuilder::default()
                    .parse_newick_from_str($text, NodeIdx(0))
                    .unwrap_err();
                assert!(matches!(result, $expect), "Got: {result:?}");
            }
        };
    }

    parser_error_test!(unexpected_end, "123", ParserError::UnexpectedEnd { .. });
    parser_error_test!(expected_end, "123,", ParserError::ExpectedEnd { .. });
    parser_error_test!(expected_comma, "(123)", ParserError::ExpectedComma { .. });
    parser_error_test!(
        expected_node_begin,
        "(123,)",
        ParserError::ExpectedNodeBegin { .. }
    );
    parser_error_test!(
        expected_closing,
        "(123,123,23)",
        ParserError::ExpectedClosing { .. }
    );

    #[test]
    fn binary() {
        let mut lexer = Lexer::new(" ( ( 0 , 1 ) , 2 ) ;");
        lexer.allow_whitespaces();
        let tree = BinTreeBuilder::default()
            .parse_newick_from_lexer(&mut lexer, NodeIdx::new(0))
            .expect("A valid binary tree");
        let lc = tree.top_down().left_child().unwrap();

        assert_eq!(lc.left_child().unwrap().leaf_label().unwrap(), Label(0));
        assert_eq!(lc.right_child().unwrap().leaf_label().unwrap(), Label(1));
        assert_eq!(
            tree.top_down().right_child().unwrap().leaf_label().unwrap(),
            Label(2)
        );
    }

    #[test]
    fn parser_writer_roundtrip() {
        fn test_string(text: &str) {
            let tree = BinTreeBuilder::default()
                .parse_newick_from_str(text, NodeIdx::new(0))
                .unwrap();
            assert_eq!(text, tree.top_down().to_newick_string());
        }

        test_string("1;");
        test_string("(1,2);");
        test_string("(1,(5,91234));");
        test_string("(((4,2),(7,1)),8);");
    }

    #[test]
    fn parser_indexed_bintree() {
        let tree = IndexedBinTreeBuilder::default()
            .parse_newick_from_str("((1,2),(3,(5,4)));", NodeIdx::new(6))
            .unwrap();

        assert_eq!(tree.node_idx(), NodeIdx::new(6));

        let td = tree.top_down();
        assert_eq!(navigate(td, "l").unwrap().node_idx(), NodeIdx::new(7));
        assert_eq!(navigate(td, "ll").unwrap().node_idx(), NodeIdx::new(1));
        assert_eq!(navigate(td, "lr").unwrap().node_idx(), NodeIdx::new(2));
        assert_eq!(navigate(td, "r").unwrap().node_idx(), NodeIdx::new(8));
        assert_eq!(navigate(td, "rl").unwrap().node_idx(), NodeIdx::new(3));
        assert_eq!(navigate(td, "rr").unwrap().node_idx(), NodeIdx::new(9));
        assert_eq!(navigate(td, "rrl").unwrap().node_idx(), NodeIdx::new(5));
        assert_eq!(navigate(td, "rrr").unwrap().node_idx(), NodeIdx::new(4));
    }
}
