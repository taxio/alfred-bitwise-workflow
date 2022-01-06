use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitwiseError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Invalid cursor position: {0}")]
    CursorPosition(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let ans = match calculate(query) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    println!("Answer: {}", ans);

    // TODO: Build for Alfred
}

fn calculate(query: &str) -> Result<u64, BitwiseError> {
    println!("query: {}", query);

    let mut lex = Lexer::new(query);
    let tokens = lex.tokenize().unwrap();
    for t in tokens.iter() {
        println!("{:?}", t);
    }

    println!("-------------------------------");

    let tokens = reverse_polish_notation(tokens).unwrap();
    for t in tokens.iter() {
        println!("{:?}", t);
    }

    println!("-------------------------------");

    // 計算する (オーバーフローの検出もする)
    let mut calc_stack: Vec<u64> = Vec::new();
    for t in tokens {
        match t.kind {
            TokenKind::Value(v) => {
                calc_stack.push(v.parse::<u64>().unwrap());
            }
            TokenKind::Symbol(s) => {
                let v1 = calc_stack.pop().unwrap();
                let v2 = calc_stack.pop().unwrap();
                println!("{} {:?} {}", v2, s, v1);
                let v: u64 = match s {
                    Symbol::And => v2 & v1,
                    Symbol::Xor => v2 ^ v1,
                    Symbol::Or => v2 | v1,
                    Symbol::LSHIFT => v2 << v1,
                    Symbol::RSHIFT => v2 >> v1,
                    _ => {
                        return Err(BitwiseError::RuntimeError(format!(
                            "Unsupported not yet: {:?}",
                            s
                        )));
                    }
                };
                calc_stack.push(v);
            }
            TokenKind::EOL => {
                break;
            }
        }
    }

    if calc_stack.len() != 1 {
        return Err(BitwiseError::RuntimeError(format!(
            "calc_stack size is not 1, got {}",
            calc_stack.len()
        )));
    }

    let ans = calc_stack.pop().unwrap();

    Ok(ans)
}

// TOOD: Support PARENs
fn reverse_polish_notation(tokens: Vec<Token>) -> Result<Vec<Token>, BitwiseError> {
    let mut rets: Vec<Token> = Vec::new();
    let mut stack: Vec<Symbol> = Vec::new();
    for token in tokens {
        match token.kind {
            TokenKind::Value(_) => {
                rets.push(token);
            }
            TokenKind::Symbol(s) => match stack.pop() {
                Some(prev_symbol) => {
                    if prev_symbol < s {
                        rets.push(Token {
                            kind: TokenKind::Symbol(prev_symbol),
                        });
                        stack.push(s);
                    } else {
                        stack.push(prev_symbol);
                        stack.push(s);
                    }
                }
                None => {
                    stack.push(s);
                }
            },
            TokenKind::EOL => {
                break;
            }
        }
    }

    stack.reverse();
    for s in stack {
        rets.push(Token {
            kind: TokenKind::Symbol(s),
        });
    }

    Ok(rets)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCaseForCalculate {
        src: String,
        ans: u64,
    }

    #[test]
    fn test_calculate() {
        let tests = [
            TestCaseForCalculate {
                src: "123 & 456".to_string(),
                ans: 72,
            },
            TestCaseForCalculate {
                src: "123 & 456".to_string(),
                ans: 72,
            },
        ];

        for t in tests {
            let got = match calculate(&t.src) {
                Ok(v) => v,
                Err(e) => panic!("Error: {}", e),
            };
            assert_eq!(got, t.ans);
        }
    }

    #[derive(Debug)]
    struct TestCaseForTokenize {
        src: String,
        tokens: Vec<Token>,
    }

    #[test]
    fn test_tokenize() {
        let tests = [
            TestCaseForTokenize {
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
            TestCaseForTokenize {
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
