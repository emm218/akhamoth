use std::{borrow::Cow, iter, str::Chars};

use unicode_xid::UnicodeXID;

#[derive(Debug)]
pub enum Token<'src> {
    StringLiteral(Cow<'src, str>),
    IntLiteral(i64),
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
    Unknown(char),
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
            '"' => self.string_literal(),
            '0' => {
                self.reset_token();
                match self.peek() {
                    'b' => {
                        self.bump();
                        self.reset_token();
                        self.number(&source[2..], 2)
                    }
                    'o' => {
                        self.bump();
                        self.reset_token();
                        self.number(&source[2..], 8)
                    }
                    'x' => {
                        self.bump();
                        self.reset_token();
                        self.number(&source[2..], 16)
                    }
                    '0'..='9' => self.number(source, 10),
                    _ => Token::IntLiteral(0),
                }
            }
            '1'..='9' => self.number(source, 10),
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
            _ => Token::Unknown(first),
        };
        self.reset_token();
        Some(token)
    }

    fn number(&mut self, source: &'src str, radix: u32) -> Token<'static> {
        self.eat_while(|c| c.is_digit(radix));

        let len = self.token_length();
        Token::IntLiteral(i64::from_str_radix(&source[..len], radix).unwrap())
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

// TODO: actually unescape string lol
fn unescape(s: &str) -> Cow<'_, str> {
    s.into()
}
