use std::{
    io::{self, Read, Write},
    str::from_utf8,
};

use crate::parser::{Cmd, Op};

const MEM_SIZE: usize = 30_000;
pub struct Interpreter {
    mem: [u8; MEM_SIZE],
}

impl Interpreter {
    pub fn new() -> Self {
        Self { mem: [0; MEM_SIZE] }
    }

    pub fn run_all(&mut self, cmds: Vec<Cmd>) {
        let mut mem_ptr = 0;
        let mut instr_ptr = 0;

        while instr_ptr < cmds.len() {
            let cell = &mut self.mem[mem_ptr];
            let cmd = &cmds[instr_ptr];

            match cmd.operator {
                Op::Add => *cell = cell.wrapping_add(cmd.operand as u8),
                Op::Sub => *cell = cell.wrapping_sub(cmd.operand as u8),
                Op::Left => {
                    mem_ptr = (mem_ptr as i128 - cmd.operand as i128).rem_euclid(MEM_SIZE as i128)
                        as usize
                }
                Op::Right => mem_ptr = (mem_ptr + cmd.operand).rem_euclid(MEM_SIZE),
                Op::Out => {
                    if !cell.is_ascii() {
                        panic!("Runtime error: tried to output invalid ascii");
                    }

                    let output: Vec<_> = (0..cmd.operand).map(|_| *cell).collect();
                    print!("{}", from_utf8(&output).expect("Is valid ascii"));
                    io::stdout().flush().expect("Could not flush to stdout");
                }
                Op::In => {
                    let mut buf = vec![0; cmd.operand];
                    let res = io::stdin().read_exact(&mut buf);

                    // Only the last byte stays
                    *cell = res.map(|_| buf[cmd.operand - 1]).unwrap_or(0);
                }
                Op::JmpZero => {
                    if *cell == 0 {
                        instr_ptr = cmd.operand;
                        continue;
                    }
                }
                Op::JmpNonZero => {
                    if *cell != 0 {
                        instr_ptr = cmd.operand;
                        continue;
                    }
                }
            };

            instr_ptr += 1;
        }
    }
}
