use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read, Write},
    str::Chars,
};

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    MoveLeft,  // <
    MoveRight, // >
    Increment, // +
    Decrement, // -
    Print,     // .
    Input,     // ,
    Jmp,       // [
    Pmj,       // ]
}

#[derive(Debug)]
struct Token {
    pub kind: TokenKind,
    pos: usize,
}

impl Token {
    pub fn from(kind: TokenKind, pos: usize) -> Self {
        Self { kind, pos }
    }
}

struct Lexer<'a> {
    buffer: Chars<'a>,
}

impl Lexer<'_> {
    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos: usize = 1;
        loop {
            match self.buffer.next() {
                None => break,
                Some(c) => match Self::identify(c) {
                    Some(token) => tokens.push(Token::from(token, pos)),
                    None => (),
                },
            }
            pos += 1;
        }
        tokens
    }

    fn identify(c: char) -> Option<TokenKind> {
        use TokenKind::*;
        match c {
            '<' => Some(MoveLeft),
            '>' => Some(MoveRight),
            '+' => Some(Increment),
            '-' => Some(Decrement),
            '.' => Some(Print),
            ',' => Some(Input),
            '[' => Some(Jmp),
            ']' => Some(Pmj),
            _ => None,
        }
    }
}

impl<'a> From<&'a str> for Lexer<'a> {
    fn from(buffer: &'a str) -> Self {
        let buffer = buffer.chars();
        Self { buffer }
    }
}

struct Interpreter {
    tokens: Vec<Token>,
    program_cursor: usize,
    mem_cursor: usize,
    mem: Vec<u8>,
    jmp_table: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn from(mut lexer: Lexer) -> Self {
        Self {
            tokens: lexer.lex(),
            program_cursor: 0,
            mem_cursor: 0,
            mem: vec![0],
            jmp_table: HashMap::new(),
        }
    }

    pub fn interpret(&mut self) {
        use TokenKind as K;
        self.load_jump_table();
        loop {
            match self.tokens.get(self.program_cursor) {
                Some(tok) => match tok.kind {
                    K::Print => print!("{}", self.mem[self.mem_cursor] as char),
                    K::Input => {
                        let mut input = [0];
                        let _ = io::stdout().flush();
                        io::stdin().read_exact(&mut input).expect(
                            format!("Couldn't fetch user input: failed on , at pos {}", tok.pos)
                                .as_str(),
                        );
                        self.mem[self.mem_cursor] = input[0];
                    }
                    K::MoveRight => {
                        self.mem_cursor += 1;
                        if self.mem_cursor >= self.mem.len() {
                            self.mem.push(0);
                        }
                    }
                    K::MoveLeft => {
                        if self.mem_cursor == 0 {
                            self.mem.insert(0, 0);
                        }
                        self.mem_cursor -= 1;
                    }
                    K::Increment => {
                        self.mem[self.mem_cursor] = self.mem[self.mem_cursor].wrapping_add(1)
                    }
                    K::Decrement => {
                        self.mem[self.mem_cursor] = self.mem[self.mem_cursor].wrapping_sub(1)
                    }
                    K::Jmp => {
                        if self.mem[self.mem_cursor] == 0 {
                            self.program_cursor =
                                *self.jmp_table.get(&self.program_cursor).unwrap();
                        }
                    }
                    K::Pmj => {
                        if self.mem[self.mem_cursor] != 0 {
                            self.program_cursor =
                                *self.jmp_table.get(&self.program_cursor).unwrap();
                        }
                    }
                },
                None => break,
            }
            self.program_cursor += 1;
        }
    }

    fn load_jump_table(&mut self) {
        let mut stack: Vec<usize> = vec![];
        let mut check = 0usize;
        for (i, v) in self.tokens.iter().enumerate() {
            if v.kind == TokenKind::Jmp {
                check += 1;
                stack.push(i);
            }
            if v.kind == TokenKind::Pmj {
                match stack.pop() {
                    Some(index) => {
                        self.jmp_table.insert(index, i);
                        self.jmp_table.insert(i, index);
                    }
                    None => unreachable!(""),
                };
                check += 1;
            }
        }
        if check % 2 != 0 {
            panic!("No matching ]");
        }
    }
}

fn launch_repl() {
    // todo!()
}

fn main() {
    let mut args = env::args();
    let alen = args.len();
    match alen {
        1 => launch_repl(),
        2 => {
            let source = fs::read_to_string(args.nth(1).unwrap()).expect("Failed to read file");
            let lexer = Lexer::from(source.as_str());
            Interpreter::from(lexer).interpret();
        }
        _ => eprintln!("Too many arguments"),
    };
}
