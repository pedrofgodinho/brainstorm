use std::fmt::Display;
use std::io::{BufRead, BufReader, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("] has no matching [ ")]
    MissingOpen,
    #[error("[ has no matching ]")]
    MissingClose,
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    Increment(u8),
    Move(isize),
    JumpZero(usize),
    JumpNotZero(usize),
    Input,
    Output,
    PrintState,
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Increment(value) => {
                let value = *value as i8;
                if value > 0 {
                    write!(f, "+{}", value as u8)
                } else {
                    write!(f, "-{}", -value as u8)
                }
            }
            Self::Move(value) => {
                if *value > 0 {
                    write!(f, ">{value}")
                } else {
                    write!(f, "<{}", -value)
                }
            }
            Self::JumpZero(_) => {
                write!(f, "[")
            }
            Self::JumpNotZero(_) => {
                write!(f, "]")
            }
            Self::Input => {
                write!(f, ",")
            }
            Self::Output => {
                write!(f, ".")
            }
            Self::PrintState => {
                write!(f, "#")
            }
            Self::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
pub struct Unit {
    pub description: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct Program {
    pub units: Vec<Unit>,
    pub tokens: Vec<Token>,
}

impl Program {
    pub fn parse<T: Read>(input: BufReader<T>, parse_print: bool) -> Result<Program, ParserError> {
        let mut tokens = Vec::new();
        let mut next_token = None;
        let mut jump_stack = Vec::new();
        let mut units: Vec<Unit> = Vec::new();

        for line in input.lines() {
            let line = line?;

            let line = line.trim();
            if let Some(line) = line.strip_prefix(";") {
                Self::push_token(&mut tokens, &mut next_token);
                
                if units.is_empty() && !tokens.is_empty() {
                    units.push(Unit {
                        description: "No Unit Name".to_string(),
                        start: 0,
                        end: tokens.len(),
                    });
                }

                if let Some(last) = units.last_mut() {
                    last.end = tokens.len();
                }

                units.push(Unit {
                    description: line.trim().to_owned(),
                    start: tokens.len(),
                    end: 0,
                });
            }

            for char in line.chars() {
                match char {
                    '+' | '-' => {
                        let initial_value = if char == '+' { 1 } else { 255 };

                        match next_token {
                            Some(Token::Increment(value)) => {
                                next_token = Some(Token::Increment(value.wrapping_add(initial_value)));
                            }
                            _ => {
                                Self::push_token(&mut tokens, &mut next_token);
                                next_token = Some(Token::Increment(initial_value));
                            }
                        }
                    }
                    '>' | '<' => {
                        let initial_value = if char == '>' { 1 } else { -1 };

                        match next_token {
                            Some(Token::Move(value)) => {
                                next_token = Some(Token::Move(value.wrapping_add(initial_value)));
                            }
                            _ => {
                                Self::push_token(&mut tokens, &mut next_token);
                                next_token = Some(Token::Move(initial_value));
                            }
                        }
                    }
                    '.' => {
                        Self::push_token(&mut tokens, &mut next_token);
                        tokens.push(Token::Output);
                    }
                    ',' => {
                        Self::push_token(&mut tokens, &mut next_token);
                        next_token = Some(Token::Input);
                    }
                    '[' => {
                        Self::push_token(&mut tokens, &mut next_token);
                        tokens.push(Token::JumpZero(0)); // Value is set when the matching ']' is found
                        jump_stack.push(tokens.len());
                    }
                    ']' => {
                        Self::push_token(&mut tokens, &mut next_token);
                        let start = jump_stack.pop().ok_or(ParserError::MissingOpen)?;
                        tokens[start - 1] = Token::JumpZero(tokens.len() + 1);
                        tokens.push(Token::JumpNotZero(start));
                    }
                    '#' => {
                        if !parse_print {
                            continue;
                        }
                        Self::push_token(&mut tokens, &mut next_token);
                        tokens.push(Token::PrintState);
                    }
                    _ => continue,
                }
            }
        }

        if let Some(token) = next_token.take() {
            tokens.push(token);
        }
        tokens.push(Token::Eof);

        if !jump_stack.is_empty() {
            return Err(ParserError::MissingClose);
        }

        // If no units, create a default unit
        if units.is_empty() {
            units.push(Unit {
                description: "No Unit Information".to_string(),
                start: 0,
                end: tokens.len(),
            })
        }

        // Update last unit
        units.last_mut().unwrap().end = tokens.len();

        Ok(Program { units, tokens })
    }

    fn push_token(tokens: &mut Vec<Token>, token: &mut Option<Token>) {
        if let Some(token) = token.take() {
            match token {
                Token::Increment(0) | Token::Move(0) => (),
                _ => tokens.push(token),
            }
        }
    }
}