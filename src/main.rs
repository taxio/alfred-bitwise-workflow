use serde::{Deserialize, Serialize};
use std::env;
use std::num::ParseIntError;
use std::process;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitwiseError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("{0}")]
    JsonSerializeError(#[from] serde_json::Error),
}

fn main() {
    match run() {
        Ok(_) => process::exit(0),
        Err(e) => {
            output_error(format!("{}", e));
            process::exit(1);
        }
    }
}

fn run() -> Result<(), BitwiseError> {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    if query.is_empty() {
        return Ok(());
    }

    let ans = calculate(query)?;

    println!("{}", build_json_for_alfred(ans)?);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct AlfredScriptFilterIcon {
    path: String,
}

// See: https://www.alfredapp.com/help/workflows/inputs/script-filter/json/
#[derive(Serialize, Deserialize, Debug)]
struct AlfredScriptFilterItem {
    title: String,
    subtitle: String,
    arg: String,
    icon: AlfredScriptFilterIcon,
}

#[derive(Serialize, Deserialize, Debug)]
struct AlfredScriptFilter {
    items: Vec<AlfredScriptFilterItem>,
}

fn build_json_for_alfred(ans: u64) -> Result<String, BitwiseError> {
    let items: Vec<AlfredScriptFilterItem> = vec![
        AlfredScriptFilterItem {
            title: format!("{:x}", ans),
            subtitle: "Hexadecimal".to_string(),
            arg: format!("{:x}", ans),
            icon: AlfredScriptFilterIcon {
                path: "".to_string(),
            },
        },
        AlfredScriptFilterItem {
            title: format!("{}", ans),
            subtitle: "Decimal".to_string(),
            arg: format!("{}", ans),
            icon: AlfredScriptFilterIcon {
                path: "".to_string(),
            },
        },
        AlfredScriptFilterItem {
            title: format!("{:o}", ans),
            subtitle: "Octal".to_string(),
            arg: format!("{:o}", ans),
            icon: AlfredScriptFilterIcon {
                path: "".to_string(),
            },
        },
        AlfredScriptFilterItem {
            title: format!("{:b}", ans),
            subtitle: "Binary".to_string(),
            arg: format!("{:b}", ans),
            icon: AlfredScriptFilterIcon {
                path: "".to_string(),
            },
        },
    ];

    let serialized = serde_json::to_string(&AlfredScriptFilter { items })?;

    Ok(serialized)
}

fn output_error(msg: String) {
    eprintln!("Error: {}", msg);

    let json: AlfredScriptFilter = AlfredScriptFilter {
        items: vec![AlfredScriptFilterItem {
            title: msg.clone(),
            subtitle: "Error".to_string(),
            arg: msg,
            icon: AlfredScriptFilterIcon {
                path: "".to_string(),
            },
        }],
    };
    println!("{}", serde_json::to_string(&json).unwrap());
}

fn calculate(query: &str) -> Result<u64, BitwiseError> {
    let mut lex = Lexer::new(query);
    let tokens = lex.tokenize()?;
    let tokens = reverse_polish_notation(tokens)?;

    let mut calc_stack: Vec<u64> = Vec::new();
    for t in tokens {
        match t.kind {
            TokenKind::Value(v) => calc_stack.push(v.u64()?),
            TokenKind::Symbol(s) => match s {
                Symbol::LPAREN | Symbol::RPAREN => {}
                _ => {
                    let v1 = calc_stack.pop().unwrap();
                    let v2 = calc_stack.pop().unwrap();
                    let v: u64 = match s {
                        Symbol::And => v2 & v1,
                        Symbol::Xor => v2 ^ v1,
                        Symbol::Or => v2 | v1,
                        // TODO: Check overflow
                        Symbol::LSHIFT => v2 << v1,
                        Symbol::RSHIFT => v2 >> v1,
                        _ => {
                            return Err(BitwiseError::InvalidQuery(format!(
                                "unsupported symbol: {:?}",
                                s
                            )));
                        }
                    };
                    calc_stack.push(v);
                }
            },
            TokenKind::EOL => {
                break;
            }
        }
    }

    if calc_stack.len() != 1 {
        panic!("calc_stack size is not 1, got {}", calc_stack.len());
    }

    Ok(calc_stack.pop().unwrap())
}

fn reverse_polish_notation(tokens: Vec<Token>) -> Result<Vec<Token>, BitwiseError> {
    let mut rets: Vec<Token> = Vec::new();
    let mut stack: Vec<Symbol> = Vec::new();
    for token in tokens {
        match token.kind {
            TokenKind::Value(_) => rets.push(token),
            TokenKind::Symbol(s) => match s {
                Symbol::LPAREN => stack.push(s),
                Symbol::RPAREN => loop {
                    match stack.pop() {
                        Some(prev_symbol) => match prev_symbol {
                            Symbol::LPAREN => break,
                            _ => rets.push(Token {
                                kind: TokenKind::Symbol(prev_symbol),
                            }),
                        },
                        None => {
                            return Err(BitwiseError::InvalidQuery(
                                "incorrect pair of parentheses".to_string(),
                            ))
                        }
                    }
                },
                _ => match stack.pop() {
                    Some(prev_symbol) => {
                        if prev_symbol == Symbol::LPAREN || prev_symbol == Symbol::RPAREN {
                            stack.push(prev_symbol);
                            stack.push(s);
                        } else if prev_symbol < s {
                            rets.push(Token {
                                kind: TokenKind::Symbol(prev_symbol),
                            });
                            stack.push(s);
                        } else {
                            stack.push(prev_symbol);
                            stack.push(s);
                        }
                    }
                    None => stack.push(s),
                },
            },
            TokenKind::EOL => break,
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
            panic!("cursor cannot go back ahead of the leader");
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
    Value(Value),
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
pub enum Value {
    Hex(String),
    Dec(String),
    Oct(String),
    Bin(String),
}

impl Value {
    pub fn u64(&self) -> Result<u64, BitwiseError> {
        let radix = match self {
            Value::Hex(_) => 16,
            Value::Dec(_) => 10,
            Value::Oct(_) => 8,
            Value::Bin(_) => 2,
        };

        let v = match self {
            Value::Hex(s) | Value::Dec(s) | Value::Oct(s) | Value::Bin(s) => {
                u64::from_str_radix(s, radix)?
            }
        };

        Ok(v)
    }
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
                '0' => tokens.push(Token {
                    kind: TokenKind::Value(self.read_prefixed_value()?),
                }),
                '1'..='9' => tokens.push(Token {
                    kind: TokenKind::Value(self.read_dec(c)?),
                }),
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
                '<' => {
                    self.read_shift(c)?;
                    tokens.push(Token {
                        kind: TokenKind::Symbol(Symbol::LSHIFT),
                    });
                }
                '>' => {
                    self.read_shift(c)?;
                    tokens.push(Token {
                        kind: TokenKind::Symbol(Symbol::RSHIFT),
                    });
                }
                _ => {
                    return Err(BitwiseError::InvalidQuery(format!(
                        "unexpected character: {}",
                        c
                    )));
                }
            }

            if is_break {
                break;
            }
        }
        Ok(tokens)
    }

    fn read_dec(&mut self, c0: char) -> Result<Value, BitwiseError> {
        Ok(Value::Dec(self.read_value(c0, 10)?))
    }

    fn read_prefixed_value(&mut self) -> Result<Value, BitwiseError> {
        let prefix = self.csr.get();

        // Oct
        if prefix.is_digit(8) {
            return Ok(Value::Oct(self.read_value(prefix, 8)?));
        }

        if prefix != 'x' && prefix != 'd' && prefix != 'b' {
            return Err(BitwiseError::InvalidQuery(format!(
                "\"0{}\" is not supported",
                prefix
            )));
        }

        let mut cs: Vec<char> = Vec::new();
        loop {
            let c = self.csr.get();
            if c == EOL_CHAR {
                break;
            }

            let is_break;
            match prefix {
                'x' => is_break = !c.is_digit(16),
                'd' => is_break = !c.is_digit(10),
                'b' => is_break = !c.is_digit(2),
                _ => {
                    return Err(BitwiseError::InvalidQuery(format!(
                        "\"0{}\" is not supported",
                        prefix
                    )))
                }
            }
            if is_break {
                self.csr.unget()?;
                break;
            }

            cs.push(c);
        }

        if cs.is_empty() {
            return Err(BitwiseError::InvalidQuery("empty value".to_string()));
        }

        match prefix {
            'x' => Ok(Value::Hex(cs.into_iter().collect())),
            'd' => Ok(Value::Dec(cs.into_iter().collect())),
            'b' => Ok(Value::Bin(cs.into_iter().collect())),
            _ => {
                return Err(BitwiseError::InvalidQuery(format!(
                    "\"0{}\" is not supported",
                    prefix
                )))
            }
        }
    }

    fn read_value(&mut self, c0: char, radix: u32) -> Result<String, BitwiseError> {
        let mut cs: Vec<char> = vec![c0];

        loop {
            let c = self.csr.get();

            if c == EOL_CHAR {
                break;
            }
            if !c.is_digit(radix) {
                self.csr.unget()?;
                break;
            }

            cs.push(c);
        }

        Ok(cs.into_iter().collect())
    }

    fn read_shift(&mut self, c0: char) -> Result<(), BitwiseError> {
        let c1 = self.csr.get();

        if c0 != c1 {
            return Err(BitwiseError::InvalidQuery(format!(
                "unexpected token: {}{}",
                c0, c1
            )));
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
                src: "123 & 456 >> 2".to_string(),
                ans: 114,
            },
            TestCaseForCalculate {
                src: "(123 & 456) >> 2".to_string(),
                ans: 18,
            },
            TestCaseForCalculate {
                src: "(0xab & 123) >> 2 | 0b11001010 & 0456 ^ 0d789".to_string(),
                ans: 799,
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
                        kind: TokenKind::Value(Value::Dec("123".to_string())),
                    },
                    Token {
                        kind: TokenKind::EOL,
                    },
                ],
            },
            TestCaseForTokenize {
                src: "0x1ac".to_string(),
                tokens: vec![
                    Token {
                        kind: TokenKind::Value(Value::Hex("1ac".to_string())),
                    },
                    Token {
                        kind: TokenKind::EOL,
                    },
                ],
            },
            TestCaseForTokenize {
                src: "0456".to_string(),
                tokens: vec![
                    Token {
                        kind: TokenKind::Value(Value::Oct("456".to_string())),
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
                        kind: TokenKind::Value(Value::Dec("123".to_string())),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::And),
                    },
                    Token {
                        kind: TokenKind::Value(Value::Dec("456".to_string())),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::RPAREN),
                    },
                    Token {
                        kind: TokenKind::Symbol(Symbol::RSHIFT),
                    },
                    Token {
                        kind: TokenKind::Value(Value::Dec("2".to_string())),
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

    #[derive(Debug)]
    struct TestCaseForValueU64 {
        value: Value,
        expected: u64,
    }

    #[test]
    fn test_value_u64() {
        let tests = [TestCaseForValueU64 {
            value: Value::Hex("ab".to_string()),
            expected: 171,
        }];

        for t in tests {
            assert_eq!(t.value.u64().unwrap(), t.expected);
        }
    }
}
