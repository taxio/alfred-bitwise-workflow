use std::env;

pub mod error;
pub mod lexer;

use crate::error::BitwiseError;

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let ans = match calculate(query) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return
        }
    };

    println!("Answer: {}", ans);

    // TODO: Build for Alfred
}

fn calculate(query: &String) -> Result<u64, BitwiseError> {
    println!("query: {}", query);

    let mut lex = lexer::Lexer::new(query);
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
            lexer::TokenKind::Value(v) => {
                calc_stack.push(v.parse::<u64>().unwrap());
            }
            lexer::TokenKind::Symbol(s) => {
                let v1 = calc_stack.pop().unwrap();
                let v2 = calc_stack.pop().unwrap();
                println!("{} {:?} {}", v2, s, v1);
                let v: u64 = match s {
                    lexer::Symbol::And => v2 & v1,
                    lexer::Symbol::Xor => v2 ^ v1,
                    lexer::Symbol::Or => v2 | v1,
                    lexer::Symbol::LSHIFT => v2 << v1,
                    lexer::Symbol::RSHIFT => v2 >> v1,
                    _ => {
                        return Err(BitwiseError::RuntimeError(format!(
                            "Unsupported not yet: {:?}",
                            s
                        )));
                    }
                };
                calc_stack.push(v);
            }
            lexer::TokenKind::EOL => {
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
fn reverse_polish_notation(tokens: Vec<lexer::Token>) -> Result<Vec<lexer::Token>, BitwiseError> {
    let mut rets: Vec<lexer::Token> = Vec::new();
    let mut stack: Vec<lexer::Symbol> = Vec::new();
    for token in tokens {
        match token.kind {
            lexer::TokenKind::Value(_) => {
                rets.push(token);
            }
            lexer::TokenKind::Symbol(s) => match stack.pop() {
                Some(prev_symbol) => {
                    if prev_symbol < s {
                        rets.push(lexer::Token {
                            kind: lexer::TokenKind::Symbol(prev_symbol),
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
            lexer::TokenKind::EOL => {
                break;
            }
        }
    }

    stack.reverse();
    for s in stack {
        rets.push(lexer::Token {
            kind: lexer::TokenKind::Symbol(s),
        });
    }

    Ok(rets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        src: String,
        ans: u64,
    }

    #[test]
    fn it_works() {
        let tests = [
            TestCase {
                src: "123 & 456".to_string(),
                ans: 72,
            },
            TestCase {
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
}
