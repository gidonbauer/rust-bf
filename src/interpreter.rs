use std::io::Read;

use super::tokenizer::*;

fn read_char() -> u8 {
    let mut stdin = std::io::stdin();
    let mut buf = [0_u8; 1];
    match stdin.read_exact(&mut buf) {
        Ok(()) => buf[0],
        Err(err) => {
            panic!("Could not read char from stdin: {err}");
        }
    }
}

pub struct Interpreter {
    pub cells: Vec<u8>,
    pub data_ptr: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            cells: vec![0; 30000],
            data_ptr: 0,
        }
    }

    pub fn run(&mut self, prog: &Program) {
        let mut instr_ptr = 0;

        while instr_ptr < prog.instr.len() {
            match prog.instr[instr_ptr] {
                Token::IncPtr(count) => {
                    assert!(self.data_ptr + count < self.cells.len());
                    self.data_ptr += count;
                }
                Token::DecPtr(count) => {
                    assert!(self.data_ptr >= count);
                    self.data_ptr -= count;
                }
                Token::IncByte(count) => {
                    self.cells[self.data_ptr] = self.cells[self.data_ptr].wrapping_add(count)
                }
                Token::DecByte(count) => {
                    self.cells[self.data_ptr] = self.cells[self.data_ptr].wrapping_sub(count)
                }
                Token::Output => print!("{}", self.cells[self.data_ptr] as char),
                Token::Input => self.cells[self.data_ptr] = read_char(),
                Token::JumpZero(jump_addr) => {
                    if self.cells[self.data_ptr] == 0 {
                        instr_ptr = jump_addr;
                    }
                }
                Token::JumpNonZero(jump_addr) => {
                    if self.cells[self.data_ptr] != 0 {
                        instr_ptr = jump_addr;
                    }
                }
            }
            instr_ptr += 1;
        }
    }
}
