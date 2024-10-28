use std::{borrow::Cow, iter, str::Chars};

use unicode_xid::UnicodeXID;

use crate::source::{SourceFile, Span};

#[derive(Debug)]
pub struct Token<'src> {
    pub span: Span,
    pub inner: TokenInner<'src>,
}

impl Token<'_> {
    fn is_whitespace(&self) -> bool {
        matches!(self.inner, TokenInner::Whitespace)
    }
}

#[derive(Debug)]
pub enum TokenInner<'src> {
    StringLiteral {
        contents: Cow<'src, str>,
        unclosed: bool,
    },
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
    Unrecognized,
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

impl From<ParseIntError> for TokenInner<'_> {
    fn from(e: ParseIntError) -> Self {
        Self::IntLiteral(Err(e))
    }
}

struct Cursor<'src> {
    /// unconsumed characters
    len_remaining: usize,
    /// position of the start of the current token
    pos: u32,
    chars: Chars<'src>,
}

impl<'src> Cursor<'src> {
    fn new(input: &'src SourceFile) -> Self {
        let pos = input.start_pos;

        Self {
            pos,
            len_remaining: input.src.len(),
            chars: input.src.chars(),
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
        self.pos += self.token_length() as u32;
        self.len_remaining = self.as_str().len();
    }

    fn next_token(&mut self) -> Option<Token<'src>> {
        let source = self.as_str();
        let first = self.bump()?;

        let inner = match first {
            c if c.is_whitespace() => {
                self.eat_while(|c| c.is_whitespace());
                TokenInner::Whitespace
            }
            c if is_id_start(c) => {
                self.eat_while(is_id_continue);
                let len = self.token_length();
                TokenInner::Identifier(&source[..len])
            }
            c @ '0'..='9' => TokenInner::IntLiteral(self.number(source, c)),
            '"' => self.string_literal(),
            '/' => match self.peek() {
                '/' => {
                    self.eat_while(|c| c != '\n');
                    let len = self.token_length();
                    TokenInner::Comment(&source[2..len])
                }
                _ => TokenInner::Operator(Operator::Div),
            },
            '|' => TokenInner::Pipe,
            ',' => TokenInner::Comma,
            ':' => TokenInner::Colon,
            ';' => TokenInner::Semicolon,
            '(' => TokenInner::OpenDelim(DelimKind::Paren),
            '[' => TokenInner::OpenDelim(DelimKind::Bracket),
            '{' => TokenInner::OpenDelim(DelimKind::Brace),
            ')' => TokenInner::CloseDelim(DelimKind::Paren),
            ']' => TokenInner::CloseDelim(DelimKind::Bracket),
            '}' => TokenInner::CloseDelim(DelimKind::Brace),
            '+' => TokenInner::Operator(Operator::Plus),
            '-' => match self.peek() {
                '>' => {
                    self.bump();
                    TokenInner::Operator(Operator::Arrow)
                }
                _ => TokenInner::Operator(Operator::Minus),
            },
            '*' => TokenInner::Operator(Operator::Mul),
            '=' => match self.peek() {
                '>' => {
                    self.bump();
                    TokenInner::Operator(Operator::FatArrow)
                }
                _ => TokenInner::Operator(Operator::Equals),
            },
            '.' => TokenInner::Operator(Operator::Dot),
            '%' => TokenInner::Operator(Operator::Percent),
            _ => {
                self.eat_while(is_unknown);
                TokenInner::Unrecognized
            }
        };
        let span = Span::new(self.pos, self.token_length() as u32);
        self.reset_token();
        Some(Token { inner, span })
    }

    // TODO: this still feels like it could be a lot cleaner :sob:
    // TODO: need to detect and error on empty int literals
    fn number(&mut self, source: &'src str, first_char: char) -> Result<i64, ParseIntError> {
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
                return Err(ParseIntError::InvalidDigit(c, base));
            } else {
                break;
            }
        }

        let mut val = first_char.to_digit(base).unwrap() as i64;

        let len = self.token_length();
        // valid digits are all ascii so we can loop over bytes instead of characters
        for digit in source[lo..len].bytes() {
            if digit == b'_' {
                continue;
            }
            val = val
                .checked_mul(base as i64)
                .and_then(|v| {
                    v.checked_add(match digit {
                        b'a'..=b'f' => digit + 0xA - b'a',
                        b'A'..=b'F' => digit + 0xA - b'A',
                        _ => digit - b'0',
                    } as i64)
                })
                .ok_or(ParseIntError::Overflow)?;
        }

        Ok(val)
    }

    fn string_literal(&mut self) -> TokenInner<'src> {
        let source = self.as_str();

        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    let len = self.token_length() - 2;
                    return TokenInner::StringLiteral {
                        contents: unescape(&source[..len]),
                        unclosed: false,
                    };
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
        TokenInner::StringLiteral {
            contents: source[..len].into(),
            unclosed: true,
        }
    }
}

/// returns an iterator over the tokens in the input. the second value in the tuple indicates if
/// the token was preceeded by whitespace.
pub fn tokenize(input: &SourceFile) -> impl Iterator<Item = (Token<'_>, bool)> {
    let mut cursor = Cursor::new(input);
    iter::from_fn(move || {
        cursor.next_token().and_then(|t| {
            Some(if t.is_whitespace() {
                (cursor.next_token()?, true)
            } else {
                (t, false)
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
