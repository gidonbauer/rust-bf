#[path = "./lexer.rs"]
mod lexer;
pub use lexer::*;

// > 	Increment the data pointer by one (to point to the next cell to the right).
// < 	Decrement the data pointer by one (to point to the next cell to the left).
// + 	Increment the byte at the data pointer by one.
// - 	Decrement the byte at the data pointer by one.
// . 	Output the byte at the data pointer.
// , 	Accept one byte of input, storing its value in the byte at the data pointer.
// [ 	If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
// ] 	If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.

#[derive(Debug, Clone)]
pub enum Token {
    IncPtr(usize),
    DecPtr(usize),
    IncByte(u8),
    DecByte(u8),
    Output,
    Input,
    JumpZero(usize),
    JumpNonZero(usize),
}
const INVALID_JUMP_ADDR: usize = usize::MAX;

pub struct Program {
    pub instr: Vec<Token>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            instr: Vec::<Token>::new(),
        }
    }

    fn count_instr(&self, next_token: &mut Option<u8>, lexer: &mut Lexer, instr: u8) -> usize {
        let mut count = 1;
        *next_token = lexer.next();
        while (next_token.is_some()) && (next_token.unwrap() == instr) {
            *next_token = lexer.next();
            count += 1;
        }
        count
    }

    pub fn tokenize(&mut self, lexer: &mut Lexer) -> Result<(), String> {
        let mut next_token = lexer.next();
        while next_token.is_some() {
            match next_token.unwrap() as char {
                '>' => {
                    let count = self.count_instr(&mut next_token, lexer, b'>');
                    self.instr.push(Token::IncPtr(count));
                }
                '<' => {
                    let count = self.count_instr(&mut next_token, lexer, b'<');
                    self.instr.push(Token::DecPtr(count));
                }
                '+' => {
                    let count = self.count_instr(&mut next_token, lexer, b'+');
                    self.instr.push(Token::IncByte(count as u8));
                }
                '-' => {
                    let count = self.count_instr(&mut next_token, lexer, b'-');
                    self.instr.push(Token::DecByte(count as u8));
                }
                '.' => {
                    self.instr.push(Token::Output);
                    next_token = lexer.next();
                }
                ',' => {
                    self.instr.push(Token::Input);
                    next_token = lexer.next();
                }
                '[' => {
                    self.instr.push(Token::JumpZero(INVALID_JUMP_ADDR));
                    next_token = lexer.next();
                }
                ']' => {
                    self.instr.push(Token::JumpNonZero(INVALID_JUMP_ADDR));
                    next_token = lexer.next();
                }
                _ => panic!("Unreachable."),
            }
        }

        self.backpatch_jump_addr()
    }

    fn backpatch_jump_addr(&mut self) -> Result<(), String> {
        let mut jump_zero_stack = Vec::<usize>::new();
        for (idx, token) in self.instr.clone().iter().enumerate() {
            match token {
                Token::JumpZero(_) => jump_zero_stack.push(idx),
                Token::JumpNonZero(_) => {
                    if let Some(jump_zero_addr) = jump_zero_stack.pop() {
                        self.instr[idx] = Token::JumpNonZero(jump_zero_addr);
                        self.instr[jump_zero_addr] = Token::JumpZero(idx);
                    } else {
                        return Err("Missing `[` for `]`".into());
                    }
                }
                _ => (),
            }
        }

        if !jump_zero_stack.is_empty() {
            return Err(format!("Missing `]` for {} `[`", jump_zero_stack.len()));
        }

        Ok(())
    }
}
