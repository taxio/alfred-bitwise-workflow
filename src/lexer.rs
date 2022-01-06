use crate::error::BitwiseError;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    EOL,
    Symbol(Symbol),
    Value(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Symbol {
    LPAREN, // (
    RPAREN, // )

    LSHIFT, // <<
    RSHIFT, // >>
    And,    // &
    Xor,    // ^
    Or,     // |
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
}

pub struct Lexer {
    csr: Cursor,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer {
            csr: Cursor::new(src.to_string()),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, BitwiseError> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let c = self.csr.get();
            let mut is_break = false;

            match c {
                EOL_CHAR => {
                    is_break = true;
                    tokens.push(Token {
                        kind: TokenKind::EOL,
                    });
                }
                ' ' | '\t' => continue,
                '0'..='9' => {
                    let value = match self.read_value(c) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };
                    tokens.push(Token {
                        kind: TokenKind::Value(value),
                    })
                }
                '&' => tokens.push(Token {
                    kind: TokenKind::Symbol(Symbol::And),
                }),
                '|' => tokens.push(Token {
                    kind: TokenKind::Symbol(Symbol::Or),
                }),
                '^' => tokens.push(Token {
                    kind: TokenKind::Symbol(Symbol::Xor),
                }),
                '(' => tokens.push(Token {
                    kind: TokenKind::Symbol(Symbol::LPAREN),
                }),
                ')' => tokens.push(Token {
                    kind: TokenKind::Symbol(Symbol::RPAREN),
                }),
                '<' => match self.read_shift(c) {
                    Ok(_) => tokens.push(Token {
                        kind: TokenKind::Symbol(Symbol::LSHIFT),
                    }),
                    Err(e) => return Err(e),
                },
                '>' => match self.read_shift(c) {
                    Ok(_) => tokens.push(Token {
                        kind: TokenKind::Symbol(Symbol::RSHIFT),
                    }),
                    Err(e) => return Err(e),
                },
                _ => {
                    return Err(BitwiseError::InvalidToken(c.to_string()));
                }
            }

            if is_break {
                break;
            }
        }
        Ok(tokens)
    }

    fn read_value(&mut self, c0: char) -> Result<String, BitwiseError> {
        let mut cs: Vec<char> = vec![c0];

        loop {
            let c = self.csr.get();
            if c == EOL_CHAR {
                break;
            }

            // TODO; support bin, oct and hex
            if !c.is_digit(10) {
                match self.csr.unget() {
                    Ok(_) => {
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }

            cs.push(c);
        }

        Ok(cs.into_iter().collect())
    }

    fn read_shift(&mut self, c0: char) -> Result<(), BitwiseError> {
        let c1 = self.csr.get();

        if c0 != c1 {
            return Err(BitwiseError::InvalidToken(format!("{}{}", c0, c1)));
        }

        Ok(())
    }
}

const EOL_CHAR: char = '\0';

struct Cursor {
    idx: usize,
    s: String,
}

impl Cursor {
    pub fn new(s: String) -> Cursor {
        Cursor { idx: 0, s }
    }

    pub fn get(&mut self) -> char {
        if self.is_eol() {
            return EOL_CHAR;
        }

        let c = self.s.chars().nth(self.idx).unwrap_or(EOL_CHAR);
        self.idx += 1;

        c
    }

    pub fn unget(&mut self) -> Result<(), BitwiseError> {
        if self.idx == 0 {
            return Err(BitwiseError::CursorPosition(
                "cursor cannot go back ahead of the leader".to_string(),
            ));
        }

        self.idx -= 1;

        Ok(())
    }

    fn is_eol(&self) -> bool {
        self.idx == self.s.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        src: String,
        tokens: Vec<Token>,
    }

    #[test]
    fn it_works() {
        let tests = [
            TestCase {
                src: "123".to_string(),
                tokens: vec![
                    Token {
                        kind: TokenKind::Value("123".to_string()),
                    },
                    Token {
                        kind: TokenKind::EOL,
                    },
                ],
            },
            TestCase {
                src: "(123 & 456) >> 2".to_string(),
                tokens: vec![
                    Token {
                        kind: TokenKind::Symbol(Symbol::LPAREN),
                    },
                    Token {
                        kind: TokenKind::Value("123".to_string()),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::And),
                    },
                    Token {
                        kind: TokenKind::Value("456".to_string()),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::RPAREN),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::RSHIFT),
                    },
                    Token {
                        kind: TokenKind::Value("2".to_string()),
                    },
                    Token {
                        kind: TokenKind::EOL,
                    },
                ],
            },
        ];

        for t in tests {
            let mut lex = Lexer::new(&t.src);
            assert_eq!(lex.tokenize().unwrap(), t.tokens, "Failed in the {:?}", t);
        }
    }
}
