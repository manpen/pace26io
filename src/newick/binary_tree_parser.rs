use thiserror::Error;

use super::{lexer::*, *};

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

pub trait BinaryTreeParser: Buildable + Sized {
    fn parse_newick_from_lexer(&mut self, lexer: &mut Lexer) -> Result<Self::Node, ParserError>;

    fn parse_newick_from_str(&mut self, text: &str) -> Result<Self::Node, ParserError> {
        let mut lexer = Lexer::new(text);
        self.parse_newick_from_lexer(&mut lexer)
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

fn parse_inner<B: Buildable>(builder: &mut B, lexer: &mut Lexer) -> Result<B::Node, ParserError> {
    let token = lexer.next().ok_or(ParserError::UnexpectedEnd)??;

    match token.token_type {
        TokenType::ParOpen => {
            let left_child = parse_inner(builder, lexer)?;

            assert_next_token_else(lexer, TokenType::Comma, |token| {
                ParserError::ExpectedComma { token }
            })?;

            let right_child = parse_inner(builder, lexer)?;

            assert_next_token_else(lexer, TokenType::ParClose, |token| {
                ParserError::ExpectedClosing { token }
            })?;

            Ok(builder.new_inner(left_child, right_child))
        }

        TokenType::Number(x) => Ok(builder.new_leaf(Label(x))),
        _ => Err(ParserError::ExpectedNodeBegin { token }),
    }
}

impl<B: Buildable> BinaryTreeParser for B {
    fn parse_newick_from_lexer(&mut self, lexer: &mut Lexer) -> Result<Self::Node, ParserError> {
        let tree = parse_inner(self, lexer)?;

        assert_next_token_else(lexer, TokenType::Semicolon, |token| {
            ParserError::ExpectedEnd { token }
        })?;

        Ok(tree)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn leaf() {
        let tree = BinTreeBuilder::new().parse_newick_from_str("132;").unwrap();
        assert_eq!(tree.leaf_label(), Some(Label(132)));
    }

    macro_rules! parser_error_test {
        ($ident:ident, $text:expr, $expect:pat) => {
            #[test]
            fn $ident() {
                let result = BinTreeBuilder::new()
                    .parse_newick_from_str($text)
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
        let tree = BinTreeBuilder::new()
            .parse_newick_from_lexer(&mut lexer)
            .expect("A valid binary tree");
        let lc = tree.left_child().unwrap();

        assert_eq!(lc.left_child().unwrap().leaf_label().unwrap(), Label(0));
        assert_eq!(lc.right_child().unwrap().leaf_label().unwrap(), Label(1));
        assert_eq!(tree.right_child().unwrap().leaf_label().unwrap(), Label(2));
    }

    #[test]
    fn parser_writer_roundtrip() {
        fn test_string(text: &str) {
            let tree = BinTreeBuilder::new().parse_newick_from_str(text).unwrap();
            assert_eq!(text, tree.to_newick_string());
        }

        test_string("1;");
        test_string("(1,2);");
        test_string("(1,(5,91234));");
        test_string("(((4,2),(7,1)),8);");
    }
}
