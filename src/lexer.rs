pub struct Lexer {
    pub content: Vec<u8>,
    pub cursor: usize,
}

impl Lexer {
    pub fn new(input_file: &String) -> Result<Self, String> {
        match std::fs::read(input_file) {
            Ok(content) => Ok(Self { content, cursor: 0 }),
            Err(err) => Err(format!("Could not read file `{input_file}`: {err}")),
        }
    }

    fn is_instr(&self, code: u8) -> bool {
        code == b'>'
            || code == b'<'
            || code == b'+'
            || code == b'-'
            || code == b'.'
            || code == b','
            || code == b'['
            || code == b']'
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
