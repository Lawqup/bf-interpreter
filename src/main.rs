use std::{io, path};

use interpreter::Interpreter;
use parser::Parser;

mod interpreter;
mod jit;
mod parser;

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let input = &args[1];

    let cmds = Parser::from_file(path::Path::new(input))?
        .parse_all()
        .unwrap();

    let mut interpreter = Interpreter::new();

    interpreter.run_all(cmds);

    Ok(())
}
