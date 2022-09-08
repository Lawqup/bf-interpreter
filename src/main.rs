use std::boxed::Box;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};

struct BFInterpreter<'a, R: Read, W: Write> {
    raw: Vec<u8>,
    istream: &'a mut R,
    ostream: &'a mut W,
}

#[derive(PartialEq, Debug)]
enum Tokens {
    INCM, // '>', increase memory pointer
    DECM, // '<', decrease memory pointer
    INCV, // '+', increase memory value
    DECV, // '-', decrease memory value
    LSRT, // '[', start of BF loop
    LEND, // ']', end of BF loop
    IN,   // ',', get character from stdin into memory
    OUT,  // '.', put character to stdout from memory
}

impl<'a, R: Read, W: Write + Debug> BFInterpreter<'a, R, W> {
    pub fn new(raw: Vec<u8>, istream: &'a mut R, ostream: &'a mut W) -> Self {
        Self {
            raw,
            istream,
            ostream,
        }
    }

    fn parse(&self) -> Vec<Tokens> {
        use Tokens::*;

        let mut res = Vec::new();

        for c in self.raw.iter() {
            let token = match c {
                b'>' => Some(INCM),
                b'<' => Some(DECM),
                b'+' => Some(INCV),
                b'-' => Some(DECV),
                b'[' => Some(LSRT),
                b']' => Some(LEND),
                b',' => Some(IN),
                b'.' => Some(OUT),
                _ => None,
            };

            if let Some(t) = token {
                res.push(t);
            }
        }

        res
    }

    pub fn run(&mut self) {
        let mut memory = [0_u8; 30_000];
        let mut memptr = 0;

        let code = self.parse();
        let mut instr_ptr = 0;
        let mut loopstack = Vec::<usize>::new();

        loop {
            let token = code.get(instr_ptr);

            instr_ptr += 1;

            if token.is_none() {
                break;
            }

            let token = token.unwrap();

            use Tokens::*;

            match token {
                INCM => {
                    if memptr < memory.len() - 1 {
                        memptr += 1;
                    }
                }
                DECM => {
                    if memptr > 0 {
                        memptr -= 1;
                    }
                }
                INCV => {
                    memory[memptr] = memory[memptr].wrapping_add(1);
                }
                DECV => {
                    memory[memptr] = memory[memptr].wrapping_sub(1);
                }
                LSRT => {
                    // skip loop
                    if memory[memptr] == 0 {
                        while code[instr_ptr] != LEND {
                            instr_ptr += 1;
                        }
                        // skip over LEND
                        instr_ptr += 1;
                    } else {
                        // point to the instruction after LSRT
                        loopstack.push(instr_ptr);
                    }
                }
                LEND => {
                    if memory[memptr] == 0 {
                        loopstack.pop();
                    } else {
                        instr_ptr = *loopstack.last().unwrap();
                    }
                }
                IN => {
                    let buf = &mut [0; 1]; // assume ASCII
                    self.istream.read_exact(buf).unwrap();
                    memory[memptr] = buf[0];
                }
                OUT => {
                    self.ostream.write_all(&[memory[memptr]]).unwrap();
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        panic!("Incorrect number of arguments");
    }

    let mut file = File::open(args.remove(1))?;
    let mut raw = Vec::new();

    file.read_to_end(&mut raw)?;

    let mut stdin = Box::new(std::io::stdin());
    let mut stdout = Box::new(std::io::stdout());
    BFInterpreter::new(raw, &mut stdin, &mut stdout).run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interpret<'a>(code: &'a str, input: &'a str, output: &'a mut Vec<u8>) {
        let mut inbuf = input.as_bytes();

        let raw = code.as_bytes().to_vec();

        BFInterpreter::new(raw, &mut inbuf, output).run();
    }

    #[test]
    fn reprints() {
        let mut output = Vec::new();

        interpret(",.", "a", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output, "a".as_bytes());
    }

    #[test]
    fn add() {
        let mut output = Vec::new();

        interpret("+.", "", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 1);
    }

    #[test]
    fn with_comments() {
        let mut output = Vec::new();

        let code = "add one: + print value: .";

        interpret(code, "", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 1);
    }

    #[test]
    fn subtract() {
        let mut output = Vec::new();

        interpret("+-.", "", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 0);
    }

    #[test]
    fn underflow() {
        let mut output = Vec::new();

        interpret("-.", "", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 255);
    }

    #[test]
    fn shift() {
        let mut output = Vec::new();

        interpret("+>++<.>.>.", "", &mut output);

        assert_eq!(output.len(), 3);
        assert_eq!(output, [1, 2, 0]);
    }

    #[test]
    fn skip_loop() {
        let mut output = Vec::new();

        interpret("[-].", "", &mut output);

        assert_eq!(output.len(), 1);
        assert_eq!(output, [0]);
    }

    #[test]
    fn add_loop() {
        let mut output = Vec::new();

        interpret("+++[>++<-].>.", "", &mut output);

        assert_eq!(output.len(), 2);
        assert_eq!(output, [0, 6]);
    }

    #[test]
    fn nested_loop() {
        let mut output = Vec::new();

        let code = "++[>+++[>+++++<-]<-].>.>.";

        interpret(code, "", &mut output);

        assert_eq!(output.len(), 3);
        assert_eq!(output, [0, 0, 30]);
    }

    #[test]
    fn looped_input() {
        let mut output = Vec::new();

        let code = "++++[>,.<-]";

        interpret(code, "bruh", &mut output);

        assert_eq!(output.len(), 4);
        assert_eq!(output, "bruh".as_bytes());
    }
}
