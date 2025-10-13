use std::{
    collections::HashMap, fmt::{self, Display, Formatter}, fs, io, path::PathBuf
};

use clap::Parser;
use crossterm::execute;
use log::*;

#[derive(Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    clang_log::init(Level::Trace, "snl");

    let src = fs::read_to_string(args.file)?;

    let mut vm = Vm::new(&src);
    vm.run()?;
    execute!(std::io::stdout(), crossterm::cursor::Show,).unwrap();
    Ok(())
}

pub struct Vm<'src> {
    ptr: usize,
    src: &'src str,
    data: Tape<u8>,
}

#[derive(Debug)]
pub enum Context {
    Loop(usize),
}

impl<'src> Vm<'src> {
    pub fn new(src: &'src str) -> Self {
        Vm {
            ptr: 0,
            src,
            data: Tape::default(),
        }
    }

    pub fn current_char(&self) -> Option<char> {
        self.src.chars().nth(self.ptr)
    }

    pub fn next_char(&mut self) -> Option<char> {
        let c = self.current_char();
        self.ptr += 1;
        c
    }

    pub fn seek_char(&mut self, i: usize) {
        self.ptr = i;
    }

    pub fn char_ptr(&self) -> usize {
        self.ptr
    }

    pub fn at_eof(&self) -> bool {
        self.current_char().is_none()
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut stack: Vec<Context> = Vec::new();

        while let Some(c) = self.next_char() {
            match c {
                '0'..='9' => {
                    self.data.write(c.to_digit(10).unwrap() as u8);
                }
                '>' => self.data.right(),
                '<' => self.data.left(),
                'c' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data.write(buf.trim().parse::<u8>()?);
                }
                'i' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data.write(buf.trim().parse::<char>()? as u8);
                }
                'n' => {
                    print!("{}", self.data.read());
                }
                'o' => {
                    print!("{}", self.data.read() as char);
                }
                '+' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left + right);
                }
                '-' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left - right);
                }
                '*' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left * right);
                }
                '/' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left / right);
                }
                '[' => {}
                ']' => match stack.pop() {
                    None => {}
                    Some(c) => match c {
                        Context::Loop(ptr) => {
                            if self.data.read() != 0 {
                                self.seek_char(ptr);
                                stack.push(c);
                            }
                        }
                    },
                },
                'z' => {
                    if self.current_char() != Some('[') {
                        error!("'z' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() != 0 {
                        stack.push(Context::Loop(self.ptr));
                    } else {
                        let mut stack_size = 0;
                        while let Some(c) = self.next_char() {
                            if c == ']' && stack_size == 0 {
                                break;
                            } else if c == ']' {
                                stack_size -= 1;
                            } else if c == '[' {
                                stack_size += 1;
                            }
                        }
                    }
                }
                _ => error!("Unknown character '{c}'! Skipping."),
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Tape<T>
where
    T: Copy + Default,
{
    data: HashMap<usize, T>,
    head: usize,
}

impl<T: Copy + Default> Tape<T> {
    pub fn right(&mut self) {
        self.head += 1;
    }

    pub fn left(&mut self) {
        self.head -= 1;
    }

    pub fn read(&self) -> T {
        self.data.get(&self.head).copied().unwrap_or_default()
    }

    pub fn write(&mut self, value: T) {
        self.data.insert(self.head, value);
    }

    pub fn new() -> Self {
        Tape {
            data: HashMap::new(),
            head: 0,
        }
    }
}

impl Display for Tape<u8> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut result = vec![];
        for i in &self.data {
            while result.len() <= *i.0 * 3 + 3 {
                result.push(' ');
            }
            if !(*i.1).is_ascii_control() {
                result[*i.0 * 3] = *i.1 as char;
                result[*i.0 * 3 + 2] = '|';
            } else {
                let formatted = format!("{:X}", *i.1);
                let mut chars = formatted.chars();
                result[*i.0 * 3 + 1] = chars.next().unwrap();
                result[*i.0 * 3] = chars.next().unwrap_or('0');
                result[*i.0 * 3 + 2] = '|';
            }
        }

        f.write_str(result.into_iter().collect::<String>().as_str())?;
        f.write_str("\n")?;
        f.write_str(("   ".repeat(self.head) + "^").as_str())
    }
}
