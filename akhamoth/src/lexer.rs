use std::{borrow::Cow, iter, str::Chars};

use unicode_xid::UnicodeXID;

#[derive(Debug)]
pub enum Token<'src> {
    StringLiteral(Cow<'src, str>),
    IntLiteral(Result<i64, ParseIntError>),
    Identifier(&'src str),
    OpenDelim(DelimKind),
    CloseDelim(DelimKind),
    Pipe,
    Comma,
    Colon,
    Semicolon,
    Whitespace,
    Operator(Operator),
    Comment(&'src str),
    Unrecognized(&'src str),
}

#[derive(Debug)]
pub enum DelimKind {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug)]
pub enum Operator {
    Dot,
    Arrow,
    FatArrow,
    Equals,
    Plus,
    Minus,
    Div,
    Mul,
    Percent,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseIntError {
    #[error("empty int literal")]
    Empty,
    #[error("int literal out of range")]
    Overflow,
    #[error("invalid digit '{0}' for base {1}")]
    InvalidDigit(char, u32),
}

impl From<ParseIntError> for Token<'_> {
    fn from(e: ParseIntError) -> Self {
        Self::IntLiteral(Err(e))
    }
}

struct Cursor<'src> {
    /// unconsumed characters
    len_remaining: usize,
    chars: Chars<'src>,
}

impl<'src> Cursor<'src> {
    fn new(input: &'src str) -> Self {
        Self {
            len_remaining: input.len(),
            chars: input.chars(),
        }
    }

    fn as_str(&self) -> &'src str {
        self.chars.as_str()
    }

    fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn is_eof(&self) -> bool {
        self.as_str().is_empty()
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.is_eof() {
            self.bump();
        }
    }

    /// length of the current token being lexed
    fn token_length(&self) -> usize {
        self.len_remaining - self.as_str().len()
    }

    /// reset to begin lexing another token
    fn reset_token(&mut self) {
        self.len_remaining = self.as_str().len()
    }

    fn next_token(&mut self) -> Option<Token<'src>> {
        let source = self.as_str();
        let first = self.bump()?;

        let token = match first {
            c if c.is_whitespace() => {
                self.eat_while(|c| c.is_whitespace());
                Token::Whitespace
            }
            c if is_id_start(c) => {
                self.eat_while(is_id_continue);
                let len = self.token_length();
                Token::Identifier(&source[..len])
            }
            c @ '0'..='9' => self.number(source, c),
            '"' => self.string_literal(),
            '/' => match self.peek() {
                '/' => {
                    self.eat_while(|c| c != '\n');
                    let len = self.token_length();
                    Token::Comment(&source[2..len])
                }
                _ => Token::Operator(Operator::Div),
            },
            '|' => Token::Pipe,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '(' => Token::OpenDelim(DelimKind::Paren),
            '[' => Token::OpenDelim(DelimKind::Bracket),
            '{' => Token::OpenDelim(DelimKind::Brace),
            ')' => Token::CloseDelim(DelimKind::Paren),
            ']' => Token::CloseDelim(DelimKind::Bracket),
            '}' => Token::CloseDelim(DelimKind::Brace),
            '+' => Token::Operator(Operator::Plus),
            '-' => match self.peek() {
                '>' => {
                    self.bump();
                    Token::Operator(Operator::Arrow)
                }
                _ => Token::Operator(Operator::Minus),
            },
            '*' => Token::Operator(Operator::Mul),
            '=' => match self.peek() {
                '>' => {
                    self.bump();
                    Token::Operator(Operator::FatArrow)
                }
                _ => Token::Operator(Operator::Equals),
            },
            '.' => Token::Operator(Operator::Dot),
            '%' => Token::Operator(Operator::Percent),
            _ => {
                self.eat_while(is_unknown);
                let len = self.token_length();
                Token::Unrecognized(&source[..len])
            }
        };
        self.reset_token();
        Some(token)
    }

    fn number(&mut self, source: &'src str, first_char: char) -> Token<'static> {
        let (lo, base) = if first_char == '0' {
            match self.peek() {
                'b' => (2, 2),
                'o' => (2, 8),
                'x' => (2, 16),
                _ => (0, 10),
            }
        } else {
            (0, 10)
        };

        if lo != 0 {
            self.bump();
        }

        loop {
            let c = self.peek();
            if c == '_' || c.is_digit(base) {
                self.bump();
            } else if is_id_continue(c) {
                self.eat_while(is_id_continue);
                return ParseIntError::InvalidDigit(c, base).into();
            } else {
                break;
            }
        }

        let mut val = first_char.to_digit(base).unwrap() as i64;

        let len = self.token_length();
        let digits = &source[lo..len];
        // valid digits are all ascii so we can loop over bytes instead of characters
        for digit in digits.bytes() {
            if digit == b'_' {
                continue;
            }
            val *= base as i64;
            val += match digit {
                b'a'..=b'f' => digit + 0xA - b'a',
                b'A'..=b'F' => digit + 0xA - b'A',
                _ => digit - b'0',
            } as i64;
        }

        Token::IntLiteral(Ok(val))
    }

    fn string_literal(&mut self) -> Token<'src> {
        let source = self.as_str();

        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    let len = self.token_length() - 2;
                    return Token::StringLiteral(unescape(&source[..len]));
                }
                '\\' if self.peek() == '\\' || self.peek() == '"' => {
                    self.bump();
                }
                _ => (),
            }
        }

        // some basic error recovery, if we have an unclosed string then start lexing again from
        // the next line
        self.chars = source.chars();
        self.eat_while(|c| c != '\n');
        let len = self.token_length();
        Token::StringLiteral(source[..len].into())
    }
}

/// returns an iterator over the tokens in the input. the second value in the tuple indicates if
/// the token was preceeded by whitespace.
pub fn tokenize(input: &str) -> impl Iterator<Item = (Token<'_>, bool)> {
    let mut cursor = Cursor::new(input);
    iter::from_fn(move || {
        cursor.next_token().and_then(|t| {
            Some(match t {
                Token::Whitespace => (cursor.next_token()?, true),
                t => (t, false),
            })
        })
    })
}

fn is_id_start(c: char) -> bool {
    c == '_' || c.is_xid_start()
}

fn is_id_continue(c: char) -> bool {
    c.is_xid_continue()
}

fn is_unknown(c: char) -> bool {
    !(c.is_whitespace()
        || c.is_xid_continue()
        || matches!(
            c,
            '"' | '/'
                | '|'
                | ':'
                | ';'
                | '('
                | '['
                | '{'
                | ')'
                | ']'
                | '}'
                | '+'
                | '-'
                | '*'
                | '='
                | '.'
                | '%'
                | '<'
                | '>'
        ))
}

// TODO: actually unescape string lol
fn unescape(s: &str) -> Cow<'_, str> {
    s.into()
}
