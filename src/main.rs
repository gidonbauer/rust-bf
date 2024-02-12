use std::io::Read;

// > 	Increment the data pointer by one (to point to the next cell to the right).
// < 	Decrement the data pointer by one (to point to the next cell to the left).
// + 	Increment the byte at the data pointer by one.
// - 	Decrement the byte at the data pointer by one.
// . 	Output the byte at the data pointer.
// , 	Accept one byte of input, storing its value in the byte at the data pointer.
// [ 	If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
// ] 	If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.

#[derive(Debug, Clone)]
enum Token {
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

struct Lexer {
    content: Vec<u8>,
    cursor: usize,
}

impl Lexer {
    fn new(input_file: &String) -> Result<Self, String> {
        match std::fs::read(input_file) {
            Ok(content) => Ok(Self { content, cursor: 0 }),
            Err(err) => Err(format!("Could not read file `{input_file}`: {err}")),
        }
    }

    fn is_instr(&self, code: u8) -> bool {
        code == '>' as u8
            || code == '<' as u8
            || code == '+' as u8
            || code == '-' as u8
            || code == '.' as u8
            || code == ',' as u8
            || code == '[' as u8
            || code == ']' as u8
    }
}

impl Iterator for Lexer {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.content.len() && !self.is_instr(self.content[self.cursor]) {
            self.cursor += 1;
        }

        if self.cursor >= self.content.len() {
            return None;
        }

        let res = Some(self.content[self.cursor]);
        self.cursor += 1;
        res
    }
}

struct Program {
    instr: Vec<Token>,
}

impl Program {
    fn new() -> Self {
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

    fn tokenize(&mut self, lexer: &mut Lexer) {
        let mut next_token = lexer.next();
        while next_token.is_some() {
            match next_token.unwrap() as char {
                '>' => {
                    let count = self.count_instr(&mut next_token, lexer, '>' as u8);
                    self.instr.push(Token::IncPtr(count));
                }
                '<' => {
                    let count = self.count_instr(&mut next_token, lexer, '<' as u8);
                    self.instr.push(Token::DecPtr(count));
                }
                '+' => {
                    let count = self.count_instr(&mut next_token, lexer, '+' as u8);
                    self.instr.push(Token::IncByte(count as u8));
                }
                '-' => {
                    let count = self.count_instr(&mut next_token, lexer, '-' as u8);
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

        if jump_zero_stack.len() > 0 {
            return Err(format!("Missing `]` for {} `[`", jump_zero_stack.len()));
        }

        Ok(())
    }
}

fn read_char() -> u8 {
    let mut stdin = std::io::stdin();
    let mut buf = [0 as u8; 1];
    match stdin.read_exact(&mut buf) {
        Ok(()) => buf[0],
        Err(err) => {
            panic!("Could not read char from stdin: {err}");
        }
    }
}

struct Interpreter {
    cells: Vec<u8>,
    data_ptr: usize,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            cells: vec![0; 30000],
            data_ptr: 0,
        }
    }

    fn run(&mut self, prog: &Program) {
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
                        continue;
                    }
                }
                Token::JumpNonZero(jump_addr) => {
                    if self.cells[self.data_ptr] != 0 {
                        instr_ptr = jump_addr;
                        continue;
                    }
                }
            }
            instr_ptr += 1;
        }
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(format!("Usage: {} <input>: No input provided.", args[0]));
    }

    let input_file = &args[1];
    println!("Input file: {}", input_file);

    let mut lexer = Lexer::new(input_file)?;

    let mut prog = Program::new();
    prog.tokenize(&mut lexer);
    prog.backpatch_jump_addr()?;

    let mut inter = Interpreter::new();
    inter.run(&prog);

    Ok(())
}
