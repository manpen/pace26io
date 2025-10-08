/// A lexer for tokenizing Newick format strings.
///
/// The `Lexer` struct takes an input string and produces a stream of tokens representing
/// the syntactic elements of the Newick format, such as parentheses, semicolons, and numbers.
/// It supports optional whitespace skipping and provides error handling for unexpected characters.
///
/// # Errors
///
/// Returns a [`LexerError`] if an unexpected character is encountered in the input.
use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    ParOpen,
    ParClose,
    Comma,
    Semicolon,
    Number(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub offset: usize,
    pub token_type: TokenType,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LexerError {
    #[error("unexpected character {character} at {offset}")]
    UnexpectedChar { character: char, offset: usize },
}

pub struct Lexer<'a> {
    input: Peekable<Enumerate<Chars<'a>>>,
    allow_whitespace: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().enumerate().peekable(),
            allow_whitespace: false,
        }
    }

    pub fn allow_whitespaces(&mut self) {
        self.allow_whitespace = true;
    }

    fn try_parse_number(&mut self) -> Option<(usize, u32)> {
        if self.input.peek().is_none_or(|(_, c)| !c.is_ascii_digit()) {
            return None;
        }

        let (offset, first_char) = self.input.next().unwrap();
        let mut number = first_char.to_digit(10).unwrap();

        while let Some((_, c)) = self.input.next_if(|(_, c)| c.is_ascii_digit()) {
            number = number * 10 + c.to_digit(10).unwrap();
        }

        Some((offset, number))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        // attempt to read a number
        if let Some((offset, number)) = self.try_parse_number() {
            return Some(Ok(Token {
                token_type: TokenType::Number(number),
                offset,
            }));
        }

        // otherwise try to match dedicated chars
        let (offset, next_char) = self.input.next()?;
        let token_type = match next_char {
            '(' => TokenType::ParOpen,
            ')' => TokenType::ParClose,
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            _ if self.allow_whitespace && next_char.is_whitespace() => {
                return self.next();
            }
            _ => {
                return Some(Err(LexerError::UnexpectedChar {
                    character: next_char,
                    offset,
                }));
            }
        };

        Some(Ok(Token { token_type, offset }))
    }
}

#[cfg(test)]
mod test {
    use rand::{Rng, SeedableRng};
    use rand_pcg::Pcg64Mcg;

    use super::*;

    macro_rules! token_at {
        ($offset:expr, $token:expr) => {
            Some(Ok(Token {
                offset: $offset,
                token_type: $token,
            }))
        };
    }

    #[test]
    fn strict_correct() {
        let mut lexer = Lexer::new(")(10(;23,");
        assert_eq!(lexer.next(), token_at!(0, TokenType::ParClose));
        assert_eq!(lexer.next(), token_at!(1, TokenType::ParOpen));
        assert_eq!(lexer.next(), token_at!(2, TokenType::Number(10)));
        assert_eq!(lexer.next(), token_at!(4, TokenType::ParOpen));
        assert_eq!(lexer.next(), token_at!(5, TokenType::Semicolon));
        assert_eq!(lexer.next(), token_at!(6, TokenType::Number(23)));
        assert_eq!(lexer.next(), token_at!(8, TokenType::Comma));
    }

    #[test]
    fn strict_with_spaces() {
        let mut lexer = Lexer::new(")( 10(;23");
        assert_eq!(lexer.next(), token_at!(0, TokenType::ParClose));
        assert_eq!(lexer.next(), token_at!(1, TokenType::ParOpen));
        assert!(lexer.next().unwrap().is_err());
    }

    #[test]
    fn nonstrict_with_spaces() {
        let mut lexer = Lexer::new(")( 10(;23");
        lexer.allow_whitespaces();
        assert_eq!(lexer.next(), token_at!(0, TokenType::ParClose));
        assert_eq!(lexer.next(), token_at!(1, TokenType::ParOpen));
        assert_eq!(lexer.next(), token_at!(3, TokenType::Number(10)));
        assert_eq!(lexer.next(), token_at!(5, TokenType::ParOpen));
        assert_eq!(lexer.next(), token_at!(6, TokenType::Semicolon));
        assert_eq!(lexer.next(), token_at!(7, TokenType::Number(23)));
    }

    #[test]
    fn random_number() {
        const ITERATIONS: usize = 10_000;
        let mut rng = Pcg64Mcg::seed_from_u64(0x1234678);
        for _ in 0..ITERATIONS {
            let mut text = String::with_capacity(20);
            let mut expected = Vec::with_capacity(3);

            if rng.random_bool(0.5) {
                expected.push(Token {
                    offset: text.len(),
                    token_type: TokenType::ParOpen,
                });
                text.push('(');
            }

            let rand_num = rng.random_range(0..u32::MAX);
            expected.push(Token {
                offset: text.len(),
                token_type: TokenType::Number(rand_num),
            });
            text.push_str(format!("{rand_num}").as_str());

            if rng.random_bool(0.5) {
                expected.push(Token {
                    offset: text.len(),
                    token_type: TokenType::ParClose,
                });
                text.push(')');
            }
        }
    }
}
