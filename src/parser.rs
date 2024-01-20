use std::{
    fs::File,
    io::{self, BufReader, Bytes, Read},
    path::Path,
};

struct Lexer {
    raw: Bytes<BufReader<File>>,
}

impl Lexer {
    fn from_file(path: &Path) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;

        Ok(Self {
            raw: BufReader::new(file).bytes(),
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Left,
    Right,
    Out,
    In,
    JmpZero,
    JmpNonZero,
}

impl Iterator for Lexer {
    type Item = Op;

    fn next(&mut self) -> Option<Self::Item> {
        for byte in self.raw.by_ref() {
            let byte = byte.ok()?;
            if !b"+-<>.,[]".contains(&byte) {
                continue;
            }

            let token = match byte {
                b'+' => Op::Add,
                b'-' => Op::Sub,
                b'<' => Op::Left,
                b'>' => Op::Right,
                b'.' => Op::Out,
                b',' => Op::In,
                b'[' => Op::JmpZero,
                b']' => Op::JmpNonZero,
                _ => unreachable!(),
            };

            return Some(token);
        }
        None
    }
}

pub struct Parser {
    token_stream: Lexer,
}

#[derive(Debug)]
pub struct Cmd {
    pub operator: Op,
    pub operand: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnclosedBracket(usize),
    UnopenedBracket(usize),
}

impl Parser {
    pub fn from_file(path: &Path) -> io::Result<Self> {
        Ok(Self {
            token_stream: Lexer::from_file(path)?,
        })
    }
    pub fn parse_all(mut self) -> Result<Vec<Cmd>, ParseError> {
        let mut cmds = Vec::new();

        let mut jmp_stack = Vec::new();

        let mut curr = self.token_stream.next();
        while let Some(operator) = curr {
            match operator {
                Op::Add | Op::Sub | Op::Left | Op::Right | Op::Out | Op::In => {
                    let mut next = Some(operator);
                    let mut operand = 0;
                    while next.is_some_and(|next| next == operator) {
                        operand += 1;
                        next = self.token_stream.next();
                    }

                    curr = next;
                    cmds.push(Cmd { operator, operand });
                }
                Op::JmpZero => {
                    curr = self.token_stream.next();

                    jmp_stack.push(cmds.len());
                    cmds.push(Cmd {
                        operator: Op::JmpZero,
                        operand: 0,
                    });
                }
                Op::JmpNonZero => {
                    curr = self.token_stream.next();

                    let Some(close) = jmp_stack.pop() else {
                        return Err(ParseError::UnopenedBracket(cmds.len()));
                    };

                    cmds.push(Cmd {
                        operator: Op::JmpNonZero,
                        operand: close + 1,
                    });

                    cmds[close].operand = cmds.len();
                }
            }
        }

        if let Some(close) = jmp_stack.pop() {
            return Err(ParseError::UnclosedBracket(close));
        }

        Ok(cmds)
    }
}
